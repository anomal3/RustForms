use crate::{Align, Color, ControlBase, Control, ControlImpl, EventCx, Key, Modifiers, Painter, Point};

#[derive(Control)]
pub struct Button {
    base: ControlBase,
    text: String,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(90.0, 28.0, true), text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl ControlImpl for Button {
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
            _ if state.pressed => Color::FACE_PRESSED,
            _ if state.hovered => Color::FACE_HOVER,
            _ => Color::FACE,
        };
        let (border, width) = if state.focused {
            (Color::ACCENT, 2.0)
        } else {
            (Color::BORDER, 1.0)
        };
        p.fill_round_rect(area, 4.0, border);
        p.fill_round_rect(area.inset(width), 4.0 - width / 2.0, face);
        let ink = if state.enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        p.text(&self.text, area.inset(4.0), ink, Align::Center, 0.0);
    }

    fn on_mouse_up(&mut self, _at: Point, inside: bool, cx: &mut EventCx) {
        if inside {
            cx.click();
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        if matches!(key, Key::Enter | Key::Space) {
            cx.click();
        }
    }
}
