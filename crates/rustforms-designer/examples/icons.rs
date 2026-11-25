//! Рисует значки Toolbox в `crates/rustforms-designer/icons`. Одни и те же файлы берут
//! дизайнер и оба плагина. Запуск: `cargo run -p rustforms-designer --example icons`.

use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

/// Значок 16×16 рисуется вдвое крупнее — для экранов с масштабом.
const SCALE: f32 = 2.0;
// Цвета читаются и на тёмной, и на светлой теме.
const LINE: (u8, u8, u8) = (150, 156, 166);
const ACCENT: (u8, u8, u8) = (53, 132, 228);

struct Canvas(Pixmap);

impl Canvas {
    fn paint(color: (u8, u8, u8)) -> Paint<'static> {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.0, color.1, color.2, 255));
        paint.anti_alias = true;
        paint
    }

    fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, color: (u8, u8, u8)) {
        let rect = Rect::from_xywh(x, y, w, h).expect("размер значка");
        let transform = Transform::from_scale(SCALE, SCALE);
        self.0.fill_rect(rect, &Self::paint(color), transform, None);
    }

    fn stroke(&mut self, points: &[(f32, f32)], close: bool, color: (u8, u8, u8), width: f32) {
        let mut path = PathBuilder::new();
        path.move_to(points[0].0, points[0].1);
        for p in &points[1..] {
            path.line_to(p.0, p.1);
        }
        if close {
            path.close();
        }
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        let transform = Transform::from_scale(SCALE, SCALE);
        let path = path.finish().expect("контур значка");
        self.0.stroke_path(&path, &Self::paint(color), &stroke, transform, None);
    }

    fn frame(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.stroke(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], true, LINE, 1.2);
    }

    fn circle(&mut self, radius: f32, color: (u8, u8, u8), filled: bool) {
        let path = PathBuilder::from_circle(8.0, 8.0, radius).expect("круг");
        let transform = Transform::from_scale(SCALE, SCALE);
        if filled {
            self.0.fill_path(&path, &Self::paint(color), FillRule::Winding, transform, None);
        } else {
            let stroke = Stroke { width: 1.3, ..Stroke::default() };
            self.0.stroke_path(&path, &Self::paint(color), &stroke, transform, None);
        }
    }
}

fn draw(name: &str, c: &mut Canvas) {
    match name {
        "Button" => {
            c.frame(1.5, 4.5, 13.0, 7.0);
            c.fill(4.5, 7.25, 7.0, 1.5, ACCENT);
        }
        "Label" => {
            c.stroke(&[(3.0, 13.0), (8.0, 3.0), (13.0, 13.0)], false, ACCENT, 1.8);
            c.stroke(&[(5.2, 9.5), (10.8, 9.5)], false, ACCENT, 1.6);
        }
        "TextBox" => {
            c.frame(1.5, 4.0, 13.0, 8.0);
            c.fill(4.0, 6.0, 1.4, 4.0, ACCENT);
            c.fill(7.0, 8.6, 5.0, 1.2, LINE);
        }
        "CheckBox" => {
            c.frame(2.5, 2.5, 11.0, 11.0);
            c.stroke(&[(5.0, 8.2), (7.2, 10.5), (11.2, 5.6)], false, ACCENT, 1.8);
        }
        "RadioButton" => {
            c.circle(6.0, LINE, false);
            c.circle(2.8, ACCENT, true);
        }
        "ComboBox" => {
            c.frame(1.5, 4.0, 13.0, 8.0);
            c.fill(3.5, 7.4, 4.0, 1.2, LINE);
            c.stroke(&[(9.5, 7.0), (11.2, 9.0), (12.9, 7.0)], false, ACCENT, 1.4);
        }
        "ListBox" => {
            c.frame(2.0, 2.0, 12.0, 12.0);
            c.fill(4.0, 4.2, 8.0, 1.3, LINE);
            c.fill(3.0, 6.8, 10.0, 2.4, ACCENT);
            c.fill(4.0, 10.6, 8.0, 1.3, LINE);
        }
        "ProgressBar" => {
            c.frame(1.5, 5.5, 13.0, 5.0);
            c.fill(2.5, 6.5, 7.0, 3.0, ACCENT);
        }
        "MenuStrip" => {
            c.frame(1.5, 2.5, 13.0, 11.0);
            c.fill(2.0, 3.0, 12.0, 3.0, ACCENT);
            c.fill(3.5, 8.0, 6.0, 1.2, LINE);
            c.fill(3.5, 10.6, 8.0, 1.2, LINE);
        }
        "Panel" => {
            // Пунктирная рамка.
            for i in 0..4 {
                let at = 2.0 + i as f32 * 3.4;
                c.fill(at, 2.0, 2.0, 1.2, LINE);
                c.fill(at, 12.8, 2.0, 1.2, LINE);
                c.fill(2.0, at, 1.2, 2.0, LINE);
                c.fill(12.8, at, 1.2, 2.0, LINE);
            }
            c.fill(5.5, 5.5, 5.0, 5.0, ACCENT);
        }
        _ => unreachable!("нет значка для {name}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    std::fs::create_dir_all(&dir)?;
    let mut sheet = Vec::new();
    for kind in rustforms_model::Kind::ALL {
        let size = (16.0 * SCALE) as u32;
        let mut canvas = Canvas(Pixmap::new(size, size).ok_or("значок")?);
        draw(kind.name(), &mut canvas);
        canvas.0.save_png(dir.join(format!("{}.png", kind.name())))?;
        sheet.push(canvas.0);
    }
    // Общий лист для просмотра глазами, не используется программой.
    if let Some(path) = std::env::args().nth(1) {
        let mut all = Pixmap::new(40 * sheet.len() as u32, 40).ok_or("лист")?;
        all.fill(Color::from_rgba8(30, 31, 34, 255));
        for (i, icon) in sheet.iter().enumerate() {
            let paint = tiny_skia::PixmapPaint::default();
            let x = 40 * i as i32 + 4;
            all.draw_pixmap(x, 4, icon.as_ref(), &paint, Transform::identity(), None);
        }
        all.save_png(path)?;
    }
    Ok(())
}
