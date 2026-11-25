use super::items::{Items, visible_rows};
use crate::{Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect};

const MAX_ROWS: usize = 8;

/// Выпадающий список (без свободного ввода).
#[derive(Control)]
pub struct ComboBox {
    base: ControlBase,
    items: Items,
    open: bool,
    /// Подсвеченная строка открытого списка.
    hot: Option<usize>,
}

impl ComboBox {
    pub fn new() -> Self {
        Self { base: ControlBase::new(160.0, 28.0, true), items: Items::default(), open: false, hot: None }
    }

    pub fn items(&self) -> &[String] {
        &self.items.list
    }

    pub fn set_items<S: Into<String>>(&mut self, items: impl IntoIterator<Item = S>) {
        self.items.set(items.into_iter().map(Into::into).collect());
    }

    pub fn add_item(&mut self, item: impl Into<String>) {
        self.items.list.push(item.into());
    }

    pub fn clear(&mut self) {
        self.items.set(Vec::new());
        self.open = false;
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.items.selected
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.items.select(index);
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.items.selected_text()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn rows(&self) -> usize {
        self.items.list.len().min(MAX_ROWS)
    }

    fn drop_down(&mut self) {
        if !self.items.list.is_empty() {
            self.open = true;
            self.hot = self.items.selected;
            self.items.reveal(self.hot.unwrap_or(0), self.rows());
        }
    }

    fn choose(&mut self, index: Option<usize>, cx: &mut EventCx) {
        if index.is_some() && index != self.items.selected {
            self.items.select(index);
            let text = self.selected_text().unwrap_or_default().to_owned();
            cx.selected_changed(self.items.selected, text);
        }
    }
}

impl Default for ComboBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for ComboBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        let face = match () {
            _ if !state.enabled => Color::FORM,
            _ if state.hovered || self.open => Color::FACE_HOVER,
            _ => Color::FACE,
        };
        let border = if state.focused { Color::ACCENT } else { Color::BORDER };
        p.fill_round_rect(area, 4.0, border);
        p.fill_round_rect(area.inset(1.0), 3.0, face);
        let ink = p.fore_color(Color::TEXT);
        let text = Rect::new(8.0, 0.0, (area.width - 30.0).max(0.0), area.height);
        p.text(self.items.selected_text().unwrap_or(""), text, ink, Align::Left, 0.0);
        // Стрелка из полосок: не зависит от того, есть ли значок в шрифте.
        let (x, y) = (area.width - 16.0, area.height / 2.0 - 2.0);
        for i in 0..4 {
            let i = i as f32;
            p.fill_rect(Rect::new(x - 4.0 + i, y + i, 8.0 - i * 2.0, 1.0), ink);
        }
    }

    fn on_mouse_down(&mut self, _at: Point, _mods: Modifiers, _cx: &mut EventCx) {
        if self.open {
            self.open = false;
        } else {
            self.drop_down();
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        match key {
            Key::Escape => self.open = false,
            Key::Enter | Key::Space if self.open => {
                self.open = false;
                self.choose(self.hot, cx);
            }
            Key::Enter | Key::Space => self.drop_down(),
            _ if self.open => {
                self.hot = self.items.step(self.hot, key);
                if let Some(hot) = self.hot {
                    self.items.reveal(hot, self.rows());
                }
            }
            _ => self.choose(self.items.step(self.items.selected, key), cx),
        }
    }

    fn popup_size(&self) -> Option<(f32, f32)> {
        let height = self.rows() as f32 * self.items.row_height() + 2.0;
        self.open.then_some((self.base.bounds.width, height))
    }

    fn close_popup(&mut self) {
        self.open = false;
    }

    fn on_paint_popup(&mut self, p: &mut Painter) {
        let hot = self.hot;
        self.items.paint(p, hot);
    }

    fn on_popup_mouse_move(&mut self, at: Point, _cx: &mut EventCx) {
        self.hot = self.items.row_at(at.y).or(self.hot);
    }

    fn on_popup_mouse_down(&mut self, at: Point, cx: &mut EventCx) {
        if let Some(index) = self.items.row_at(at.y) {
            self.open = false;
            self.choose(Some(index), cx);
        }
    }

    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        if self.open {
            let visible = visible_rows(cx.size().1);
            self.items.scroll(lines, visible);
        }
    }
}
