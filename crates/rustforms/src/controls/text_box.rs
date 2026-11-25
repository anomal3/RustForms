use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Align, Color, ControlBase, Control, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

const PADDING: f32 = 6.0;

/// Однострочное поле ввода.
#[derive(Control)]
pub struct TextBox {
    base: ControlBase,
    text: String,
    /// Байтовые индексы; выделение — то, что между ними.
    cursor: usize,
    anchor: usize,
    scroll: f32,
}

impl TextBox {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(160.0, 28.0, true),
            text: String::new(),
            cursor: 0,
            anchor: 0,
            scroll: 0.0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = single_line(&text.into());
        self.cursor = self.text.len();
        self.anchor = self.cursor;
        self.scroll = 0.0;
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    fn selection(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    fn replace_selection(&mut self, with: &str, cx: &mut EventCx) {
        let range = self.selection();
        if range.is_empty() && with.is_empty() {
            return;
        }
        self.cursor = range.start + with.len();
        self.anchor = self.cursor;
        self.text.replace_range(range, with);
        cx.text_changed(self.text.clone());
    }

    fn previous(&self, from: usize) -> usize {
        self.text[..from].grapheme_indices(true).next_back().map_or(0, |(i, _)| i)
    }

    fn next(&self, from: usize) -> usize {
        self.text[from..]
            .graphemes(true)
            .next()
            .map_or(from, |g| from + g.len())
    }

    fn move_to(&mut self, index: usize, extend: bool) {
        self.cursor = index;
        if !extend {
            self.anchor = index;
        }
    }

    fn index_at(&self, x: f32, cx: &mut EventCx) -> usize {
        let target = x - PADDING + self.scroll;
        cx.text_system()
            .caret_positions(&self.text)
            .into_iter()
            .min_by(|a, b| (a.1 - target).abs().total_cmp(&(b.1 - target).abs()))
            .map_or(0, |p| p.0)
    }
}

impl Default for TextBox {
    fn default() -> Self {
        Self::new()
    }
}

fn single_line(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

impl ControlImpl for TextBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        let face = if state.enabled { Color::WHITE } else { Color::FORM };
        p.fill_rect(area, face);
        p.stroke_rect(area, Color::BORDER, 1.0);
        if state.focused {
            p.fill_rect(Rect::new(0.0, area.height - 2.0, area.width, 2.0), Color::ACCENT);
        }

        let inner = Rect::new(PADDING, 2.0, (area.width - PADDING * 2.0).max(0.0), area.height - 4.0);
        let positions = p.text_system().caret_positions(&self.text);
        let x_of = |index: usize| {
            positions.iter().find(|q| q.0 == index).map_or(0.0, |q| q.1)
        };
        // Прокрутка так, чтобы курсор оставался виден.
        let cursor_x = x_of(self.cursor);
        let end_x = positions.last().map_or(0.0, |q| q.1);
        self.scroll = self
            .scroll
            .min((end_x - inner.width + 1.0).max(0.0))
            .max(cursor_x - inner.width + 1.0)
            .min(cursor_x);

        let selection = self.selection();
        if state.focused && !selection.is_empty() {
            let from = x_of(selection.start) - self.scroll;
            let to = x_of(selection.end) - self.scroll;
            let band = Rect::new(inner.x + from, 4.0, to - from, area.height - 8.0);
            p.fill_rect(band.intersect(&inner), Color::SELECTION);
        }
        let ink = if state.enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        p.text(&self.text, inner, ink, Align::Left, self.scroll);
        if state.focused {
            let caret = Rect::new(inner.x + cursor_x - self.scroll, 5.0, 1.0, area.height - 10.0);
            p.fill_rect(caret, Color::TEXT);
        }
    }

    fn on_mouse_down(&mut self, at: Point, mods: Modifiers, cx: &mut EventCx) {
        let index = self.index_at(at.x, cx);
        self.move_to(index, mods.shift);
    }

    fn on_mouse_drag(&mut self, at: Point, cx: &mut EventCx) {
        let index = self.index_at(at.x, cx);
        self.move_to(index, true);
    }

    fn on_text_input(&mut self, text: &str, cx: &mut EventCx) {
        let text = single_line(text);
        if !text.is_empty() {
            self.replace_selection(&text, cx);
        }
    }

    fn on_key(&mut self, key: Key, mods: Modifiers, cx: &mut EventCx) {
        let selection = self.selection();
        match key {
            Key::Left if !mods.shift && !selection.is_empty() => {
                self.move_to(selection.start, false)
            }
            Key::Right if !mods.shift && !selection.is_empty() => {
                self.move_to(selection.end, false)
            }
            Key::Left => self.move_to(self.previous(self.cursor), mods.shift),
            Key::Right => self.move_to(self.next(self.cursor), mods.shift),
            Key::Home => self.move_to(0, mods.shift),
            Key::End => self.move_to(self.text.len(), mods.shift),
            Key::SelectAll => {
                self.anchor = 0;
                self.cursor = self.text.len();
            }
            Key::Backspace => {
                if selection.is_empty() {
                    self.anchor = self.previous(self.cursor);
                }
                self.replace_selection("", cx);
            }
            Key::Delete => {
                if selection.is_empty() {
                    self.anchor = self.next(self.cursor);
                }
                self.replace_selection("", cx);
            }
            Key::Copy | Key::Cut if !selection.is_empty() => {
                cx.clipboard_set(&self.text[selection]);
                if key == Key::Cut {
                    self.replace_selection("", cx);
                }
            }
            Key::Paste => {
                if let Some(text) = cx.clipboard_get() {
                    self.on_text_input(&text, cx);
                }
            }
            _ => {}
        }
    }
}
