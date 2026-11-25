use super::items::{Items, visible_rows};
use crate::{Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point};

#[derive(Control)]
pub struct ListBox {
    base: ControlBase,
    items: Items,
}

impl ListBox {
    pub fn new() -> Self {
        Self { base: ControlBase::new(160.0, 112.0, true), items: Items::default() }
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
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.items.selected
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.items.select(index);
        if let Some(index) = self.items.selected {
            self.items.reveal(index, visible_rows(self.base.bounds.height));
        }
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.items.selected_text()
    }

    fn choose(&mut self, index: Option<usize>, cx: &mut EventCx) {
        if index.is_some() && index != self.items.selected {
            self.set_selected_index(index);
            let text = self.selected_text().unwrap_or_default().to_owned();
            cx.selected_changed(self.items.selected, text);
        }
    }
}

impl Default for ListBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for ListBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let selected = self.items.selected;
        self.items.paint(p, selected);
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        self.choose(self.items.row_at(at.y), cx);
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        self.choose(self.items.step(self.items.selected, key), cx);
    }

    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        let visible = visible_rows(cx.size().1);
        self.items.scroll(lines, visible);
    }
}
