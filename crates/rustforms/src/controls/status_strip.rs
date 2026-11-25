use crate::{Align, Color, Control, ControlBase, ControlImpl, Painter, Rect};

/// Строка состояния; обычно прижата к низу (`Dock::Bottom`).
#[derive(Control)]
pub struct StatusStrip {
    base: ControlBase,
    text: String,
}

impl StatusStrip {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(300.0, 24.0, false), text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl ControlImpl for StatusStrip {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let back = p.back_color(Color::rgb(232, 232, 232));
        p.fill_rect(area, back);
        p.fill_rect(Rect::new(0.0, 0.0, area.width, 1.0), Color::rgb(210, 210, 210));
        let ink = p.fore_color(Color::TEXT);
        let align = p.align(Align::Left);
        p.text(&self.text, Rect::new(8.0, 0.0, area.width - 16.0, area.height), ink, align, 0.0);
    }
}
