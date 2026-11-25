use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};
use tiny_skia::{FillRule, Mask, Paint, PathBuilder, Pixmap, Transform};

use crate::{Color, Rect};

pub const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 20.0;

#[cfg(windows)]
const FAMILY: Family<'static> = Family::Name("Segoe UI");
#[cfg(not(windows))]
const FAMILY: Family<'static> = Family::SansSerif;

/// Шрифты и раскладка текста. Все размеры снаружи — в логических пикселях.
pub struct TextSystem {
    fonts: FontSystem,
    cache: SwashCache,
}

impl TextSystem {
    pub(crate) fn new() -> Self {
        Self { fonts: FontSystem::new(), cache: SwashCache::new() }
    }

    fn layout(&mut self, text: &str, scale: f32) -> Buffer {
        let metrics = Metrics::new(FONT_SIZE * scale, LINE_HEIGHT * scale);
        let mut buffer = Buffer::new(&mut self.fonts, metrics);
        buffer.set_wrap(Wrap::None);
        buffer.set_size(None, None);
        buffer.set_text(text, &Attrs::new().family(FAMILY), Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        buffer
    }

    /// Горизонтальные позиции курсора: пары (байтовый индекс, x) по всем границам символов.
    /// Только для одной строки с письмом слева направо.
    pub fn caret_positions(&mut self, text: &str) -> Vec<(usize, f32)> {
        let buffer = self.layout(text, 1.0);
        let mut positions = vec![(0, 0.0)];
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let cluster = &text[glyph.start..glyph.end];
                let count = cluster.chars().count().max(1) as f32;
                for (n, (offset, ch)) in cluster.char_indices().enumerate() {
                    let end = glyph.start + offset + ch.len_utf8();
                    positions.push((end, glyph.x + glyph.w * (n as f32 + 1.0) / count));
                }
            }
        }
        positions.sort_by_key(|p| p.0);
        positions.dedup_by_key(|p| p.0);
        positions
    }

    pub fn width(&mut self, text: &str) -> f32 {
        self.layout(text, 1.0)
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0, f32::max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualState {
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub enabled: bool,
}

/// Рисование внутри одного контрола. Начало координат — его левый верхний угол.
pub struct Painter<'a> {
    pub(crate) pixmap: &'a mut Pixmap,
    pub(crate) text: &'a mut TextSystem,
    pub(crate) scale: f32,
    /// Положение контрола в окне.
    pub(crate) frame: Rect,
    /// Видимая часть в окне.
    pub(crate) clip: Rect,
    pub(crate) state: VisualState,
}

impl Painter<'_> {
    pub fn size(&self) -> (f32, f32) {
        (self.frame.width, self.frame.height)
    }

    /// Прямоугольник всего контрола в его координатах.
    pub fn area(&self) -> Rect {
        Rect::new(0.0, 0.0, self.frame.width, self.frame.height)
    }

    pub fn state(&self) -> VisualState {
        self.state
    }

    pub fn text_system(&mut self) -> &mut TextSystem {
        self.text
    }

    fn to_window(&self, rect: Rect) -> Rect {
        Rect::new(rect.x + self.frame.x, rect.y + self.frame.y, rect.width, rect.height)
    }

    fn physical(&self, rect: Rect) -> Option<tiny_skia::Rect> {
        let s = self.scale;
        let left = (rect.x * s).round();
        let top = (rect.y * s).round();
        let right = ((rect.x + rect.width) * s).round();
        let bottom = ((rect.y + rect.height) * s).round();
        tiny_skia::Rect::from_ltrb(left, top, right, bottom)
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let visible = self.to_window(rect).intersect(&self.clip);
        if visible.is_empty() {
            return;
        }
        if let Some(r) = self.physical(visible) {
            self.pixmap.fill_rect(r, &solid(color), Transform::identity(), None);
        }
    }

    /// Рамка внутри прямоугольника.
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        let Rect { x, y, width: w, height: h } = rect;
        self.fill_rect(Rect::new(x, y, w, width), color);
        self.fill_rect(Rect::new(x, y + h - width, w, width), color);
        self.fill_rect(Rect::new(x, y + width, width, h - width * 2.0), color);
        self.fill_rect(Rect::new(x + w - width, y + width, width, h - width * 2.0), color);
    }

    pub fn fill_round_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        let target = self.to_window(rect);
        let r = radius.min(target.width / 2.0).min(target.height / 2.0).max(0.0);
        let (x, y, right, bottom) =
            (target.x, target.y, target.x + target.width, target.y + target.height);
        // Доля радиуса для опорных точек, при которой кривая Безье повторяет дугу.
        let k = r * (1.0 - 0.552_284_8);
        let mut path = PathBuilder::new();
        path.move_to(x + r, y);
        path.line_to(right - r, y);
        path.cubic_to(right - k, y, right, y + k, right, y + r);
        path.line_to(right, bottom - r);
        path.cubic_to(right, bottom - k, right - k, bottom, right - r, bottom);
        path.line_to(x + r, bottom);
        path.cubic_to(x + k, bottom, x, bottom - k, x, bottom - r);
        path.line_to(x, y + r);
        path.cubic_to(x, y + k, x + k, y, x + r, y);
        path.close();
        let Some(path) = path.finish() else { return };
        let transform = Transform::from_scale(self.scale, self.scale);

        // Маска дорогая, поэтому строится только когда фигура выходит за видимую часть.
        let mask = if target.intersect(&self.clip) == target {
            None
        } else {
            let Some(clip) = self.physical(self.clip) else { return };
            let Some(mut mask) = Mask::new(self.pixmap.width(), self.pixmap.height()) else {
                return;
            };
            let clip = PathBuilder::from_rect(clip);
            mask.fill_path(&clip, FillRule::Winding, false, Transform::identity());
            Some(mask)
        };
        self.pixmap
            .fill_path(&path, &solid(color), FillRule::Winding, transform, mask.as_ref());
    }

    /// Одна строка текста внутри `rect`, по вертикали по центру; не выходит за его границы.
    /// `scroll` сдвигает текст влево.
    pub fn text(&mut self, text: &str, rect: Rect, color: Color, align: Align, scroll: f32) {
        let area = self.to_window(rect);
        let clip = area.intersect(&self.clip);
        if text.is_empty() || clip.is_empty() {
            return;
        }
        let Some(clip) = self.physical(clip) else { return };
        let buffer = self.text.layout(text, self.scale);
        let width = buffer.layout_runs().map(|r| r.line_w).fold(0.0, f32::max);
        let s = self.scale;
        let dx = match align {
            Align::Left => 0.0,
            Align::Center => ((area.width * s - width) / 2.0).max(0.0),
        };
        let origin_x = ((area.x - scroll) * s + dx).round() as i32;
        let origin_y = ((area.y + (area.height - LINE_HEIGHT) / 2.0) * s).round() as i32;

        let (stride, height) = (self.pixmap.width() as i32, self.pixmap.height() as i32);
        let left = (clip.left() as i32).max(0);
        let top = (clip.top() as i32).max(0);
        let right = (clip.right() as i32).min(stride);
        let bottom = (clip.bottom() as i32).min(height);
        let data = self.pixmap.data_mut();
        let ink = cosmic_text::Color::rgba(color.r, color.g, color.b, color.a);

        let mut buffer = buffer;
        buffer.draw(&mut self.text.fonts, &mut self.text.cache, ink, |x, y, w, h, c| {
            let alpha = c.a() as u32;
            if alpha == 0 {
                return;
            }
            let (x, y) = (x + origin_x, y + origin_y);
            for py in y.max(top)..(y + h as i32).min(bottom) {
                for px in x.max(left)..(x + w as i32).min(right) {
                    let i = ((py * stride + px) * 4) as usize;
                    for (k, src) in [c.r(), c.g(), c.b()].into_iter().enumerate() {
                        let mixed = src as u32 * alpha + data[i + k] as u32 * (255 - alpha);
                        data[i + k] = (mixed / 255) as u8;
                    }
                }
            }
        });
    }
}

fn solid(color: Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    paint.anti_alias = true;
    paint
}
