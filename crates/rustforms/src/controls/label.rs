use crate::{Align, Color, ControlBase, Control, ControlImpl, EventCx, Painter, Point};

#[derive(Control)]
pub struct Label {
    base: ControlBase,
    text: String,
    pub fore_color: Color,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: ControlBase::new(120.0, 22.0, false),
            text: text.into(),
            fore_color: Color::TEXT,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl ControlImpl for Label {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let ink = if p.state().enabled { self.fore_color } else { Color::TEXT_DISABLED };
        let area = p.area();
        p.text(&self.text, area, ink, Align::Left, 0.0);
    }

    fn on_mouse_up(&mut self, _at: Point, inside: bool, cx: &mut EventCx) {
        if inside {
            cx.click();
        }
    }
}
