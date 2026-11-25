use crate::{
    Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

/// Надпись-ссылка: подчёркнута и вызывает `click`. Что открывать, решает обработчик.
#[derive(Control)]
pub struct LinkLabel {
    base: ControlBase,
    text: String,
}

impl LinkLabel {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(120.0, 22.0, true), text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl ControlImpl for LinkLabel {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        if let Some(back) = p.back {
            p.fill_rect(area, back);
        }
        let ink = p.fore_color(if state.pressed { Color::rgb(0, 60, 120) } else { Color::ACCENT });
        let align = p.align(Align::Left);
        p.text(&self.text, area, ink, align, 0.0);
        let width = p.text_system().width(&self.text).min(area.width);
        let x = match align {
            Align::Left => 0.0,
            Align::Center => (area.width - width) / 2.0,
            Align::Right => area.width - width,
        };
        let y = (area.height + p.text_system().line_height()) / 2.0 - 3.0;
        let thickness = if state.hovered || state.focused { 2.0 } else { 1.0 };
        p.fill_rect(Rect::new(x, y, width, thickness), ink);
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
