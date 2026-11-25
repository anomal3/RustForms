use crate::{Align, Color, ControlBase, Control, ControlImpl, EventCx, Painter, Point};

#[derive(Control)]
pub struct Label {
    base: ControlBase,
    text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(120.0, 22.0, false), text: text.into() }
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
        let area = p.area();
        if let Some(back) = p.back {
            p.fill_rect(area, back);
        }
        let (ink, align) = (p.fore_color(Color::TEXT), p.align(Align::Left));
        p.text(&self.text, area, ink, align, 0.0);
    }

    fn on_mouse_up(&mut self, _at: Point, inside: bool, cx: &mut EventCx) {
        if inside {
            cx.click();
        }
    }
}
