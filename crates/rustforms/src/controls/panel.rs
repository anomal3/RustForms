use crate::{Color, ControlBase, Control, ControlImpl, Painter};

/// Контейнер: дочерние контролы добавляются через `ui.add_to(panel, ...)`.
#[derive(Control)]
pub struct Panel {
    base: ControlBase,
    pub back_color: Color,
    pub border: bool,
}

impl Panel {
    pub fn new() -> Self {
        Self { base: ControlBase::new(200.0, 100.0, false), back_color: Color::FORM, border: true }
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for Panel {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        p.fill_rect(area, self.back_color);
        if self.border {
            p.stroke_rect(area, Color::BORDER, 1.0);
        }
    }
}
