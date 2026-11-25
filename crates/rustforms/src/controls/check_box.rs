use crate::{Align, Color, ControlBase, Control, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect};

#[derive(Control)]
pub struct CheckBox {
    base: ControlBase,
    text: String,
    checked: bool,
}

impl CheckBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(140.0, 22.0, true), text: text.into(), checked: false }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    fn toggle(&mut self, cx: &mut EventCx) {
        self.checked = !self.checked;
        cx.checked_changed(self.checked);
        cx.click();
    }
}

impl ControlImpl for CheckBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let (_, height) = p.size();
        let mark = Rect::new(1.0, (height - 16.0) / 2.0, 16.0, 16.0);
        let border = if state.focused || state.hovered { Color::ACCENT } else { Color::BORDER };
        p.fill_round_rect(mark, 3.0, border);
        if self.checked {
            p.fill_round_rect(mark.inset(1.0), 2.5, Color::WHITE);
            p.fill_round_rect(mark.inset(3.0), 2.0, Color::ACCENT);
        } else {
            let face = if state.enabled { Color::WHITE } else { Color::FORM };
            p.fill_round_rect(mark.inset(1.0), 2.5, face);
        }
        let ink = if state.enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        let (width, _) = p.size();
        p.text(&self.text, Rect::new(24.0, 0.0, width - 24.0, height), ink, Align::Left, 0.0);
    }

    fn on_mouse_up(&mut self, _at: Point, inside: bool, cx: &mut EventCx) {
        if inside {
            self.toggle(cx);
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        if key == Key::Space {
            self.toggle(cx);
        }
    }
}
