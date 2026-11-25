use crate::{Align, Color, Control, ControlBase, ControlImpl, Painter, Rect};

/// Рамка с подписью; контейнер, как [`Panel`](crate::Panel).
#[derive(Control)]
pub struct GroupBox {
    base: ControlBase,
    text: String,
}

impl GroupBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(200.0, 100.0, false), text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl ControlImpl for GroupBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let back = p.back_color(Color::FORM);
        p.fill_rect(area, back);
        let half = (p.text_system().line_height() / 2.0).round();
        p.stroke_rect(Rect::new(0.0, half, area.width, area.height - half), Color::BORDER, 1.0);
        if !self.text.is_empty() {
            let width = p.text_system().width(&self.text).ceil() + 8.0;
            let title = Rect::new(8.0, 0.0, width, half * 2.0);
            // Подпись разрывает рамку.
            p.fill_rect(title, back);
            let ink = p.fore_color(Color::TEXT);
            p.text(&self.text, title, ink, Align::Center, 0.0);
        }
    }
}
