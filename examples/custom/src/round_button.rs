//! Свой контрол на базе кнопки: другой вид, поведение родительское.

use rustforms::prelude::*;

#[derive(Control)]
pub struct RoundButton {
    #[base]
    base: Button,
    pub color: Color,
}

impl RoundButton {
    pub fn new(text: &str, color: Color) -> Self {
        Self { base: Button::new(text), color }
    }
}

impl ControlImpl for RoundButton {
    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        let radius = area.height / 2.0;
        let Color { r, g, b, .. } = self.color;
        let shade = if state.pressed {
            0.75
        } else if state.hovered {
            0.88
        } else {
            1.0
        };
        let dim = |c: u8| (c as f32 * shade) as u8;
        if state.focused {
            p.fill_round_rect(area, radius, Color::TEXT);
        }
        let inner = area.inset(2.0);
        p.fill_round_rect(inner, radius - 2.0, Color::rgb(dim(r), dim(g), dim(b)));
        p.text(self.base.text(), inner, Color::WHITE, Align::Center, 0.0);
    }
}
