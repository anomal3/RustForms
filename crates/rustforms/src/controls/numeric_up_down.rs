use crate::{
    Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

const BUTTONS: f32 = 20.0;

/// Число со стрелками: щелчки, колесо, клавиши вверх/вниз и набор с клавиатуры.
#[derive(Control)]
pub struct NumericUpDown {
    base: ControlBase,
    value: i32,
    minimum: i32,
    maximum: i32,
    step: i32,
    /// Набираемый текст; `None` — показывается значение.
    typed: Option<String>,
}

impl NumericUpDown {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(100.0, 28.0, true),
            value: 0,
            minimum: 0,
            maximum: 100,
            step: 1,
            typed: None,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(self.minimum, self.maximum);
        self.typed = None;
    }

    pub fn set_range(&mut self, minimum: i32, maximum: i32) {
        self.minimum = minimum;
        self.maximum = maximum.max(minimum);
        self.set_value(self.value);
    }

    pub fn set_step(&mut self, step: i32) {
        self.step = step.max(1);
    }

    fn change(&mut self, value: i32, cx: &mut EventCx) {
        let value = value.clamp(self.minimum, self.maximum);
        if value != self.value {
            self.value = value;
            cx.value_changed(value);
        }
    }

    fn step_by(&mut self, steps: i32, cx: &mut EventCx) {
        self.typed = None;
        self.change(self.value.saturating_add(steps.saturating_mul(self.step)), cx);
    }

    /// Набранное применяется, как только становится числом из диапазона.
    fn typing(&mut self, text: String, cx: &mut EventCx) {
        if let Ok(value) = text.parse::<i32>()
            && (self.minimum..=self.maximum).contains(&value)
        {
            self.change(value, cx);
        }
        self.typed = Some(text);
    }
}

impl Default for NumericUpDown {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for NumericUpDown {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        if !state.focused {
            self.typed = None;
        }
        let back = p.back_color(if state.enabled { Color::WHITE } else { Color::FORM });
        p.fill_rect(area, back);
        let ink = p.fore_color(Color::TEXT);
        let shown = self.typed.clone().unwrap_or_else(|| self.value.to_string());
        let text = Rect::new(6.0, 0.0, (area.width - BUTTONS - 8.0).max(0.0), area.height);
        p.text(&shown, text, ink, Align::Left, 0.0);
        let x = area.width - BUTTONS;
        p.fill_rect(Rect::new(x, 1.0, BUTTONS - 1.0, area.height - 2.0), Color::FACE);
        p.fill_rect(Rect::new(x, 1.0, 1.0, area.height - 2.0), Color::BORDER);
        p.fill_rect(Rect::new(x, area.height / 2.0, BUTTONS, 1.0), Color::BORDER);
        // Треугольники из полосок.
        let center = x + BUTTONS / 2.0;
        for i in 0..4 {
            let i = i as f32;
            let up = area.height / 4.0 - 2.0 + i;
            let down = area.height * 0.75 + 1.0 - i;
            p.fill_rect(Rect::new(center - i, up, i * 2.0 + 1.0, 1.0), ink);
            p.fill_rect(Rect::new(center - i, down, i * 2.0 + 1.0, 1.0), ink);
        }
        p.stroke_rect(area, if state.focused { Color::ACCENT } else { Color::BORDER }, 1.0);
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        let (width, height) = cx.size();
        if at.x >= width - BUTTONS {
            self.step_by(if at.y < height / 2.0 { 1 } else { -1 }, cx);
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        match key {
            Key::Up => self.step_by(1, cx),
            Key::Down => self.step_by(-1, cx),
            Key::Enter => self.typed = None,
            Key::Backspace => {
                let mut text = self.typed.take().unwrap_or_else(|| self.value.to_string());
                text.pop();
                self.typing(text, cx);
            }
            _ => {}
        }
    }

    fn on_text_input(&mut self, input: &str, cx: &mut EventCx) {
        let mut text = self.typed.take().unwrap_or_default();
        for c in input.chars() {
            if c.is_ascii_digit() || (c == '-' && text.is_empty() && self.minimum < 0) {
                text.push(c);
            }
        }
        self.typing(text, cx);
    }

    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        if lines != 0.0 {
            self.step_by(if lines > 0.0 { 1 } else { -1 }, cx);
        }
    }
}
