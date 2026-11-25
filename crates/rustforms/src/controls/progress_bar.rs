use crate::{Color, Control, ControlBase, ControlImpl, Painter, Rect};

#[derive(Control)]
pub struct ProgressBar {
    base: ControlBase,
    value: i32,
    minimum: i32,
    maximum: i32,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self { base: ControlBase::new(200.0, 18.0, false), value: 0, minimum: 0, maximum: 100 }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    /// Значение вне диапазона прижимается к его краю.
    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(self.minimum, self.maximum);
    }

    pub fn set_range(&mut self, minimum: i32, maximum: i32) {
        self.minimum = minimum;
        self.maximum = maximum.max(minimum);
        self.set_value(self.value);
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for ProgressBar {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        p.fill_rect(area, Color::rgb(226, 226, 226));
        p.stroke_rect(area, Color::BORDER, 1.0);
        let span = i64::from(self.maximum) - i64::from(self.minimum);
        if span > 0 {
            let done = (i64::from(self.value) - i64::from(self.minimum)) as f32 / span as f32;
            let inner = area.inset(1.0);
            let filled = Rect::new(inner.x, inner.y, inner.width * done, inner.height);
            let color = if p.state().enabled { Color::ACCENT } else { Color::TEXT_DISABLED };
            p.fill_rect(filled, color);
        }
    }
}
