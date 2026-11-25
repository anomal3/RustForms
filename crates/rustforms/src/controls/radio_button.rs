use crate::{Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect};

/// Из радиокнопок одного родителя (формы или панели) отмеченной бывает одна.
#[derive(Control)]
pub struct RadioButton {
    base: ControlBase,
    text: String,
    checked: bool,
}

impl RadioButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(140.0, 22.0, true), text: text.into(), checked: false }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn checked(&self) -> bool {
        self.checked
    }

    /// Изменение из кода не снимает отметку с соседних кнопок.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    fn choose(&mut self, cx: &mut EventCx) {
        if !self.checked {
            self.checked = true;
            cx.checked_changed(true);
        }
        cx.click();
    }
}

impl ControlImpl for RadioButton {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let (width, height) = p.size();
        let mark = Rect::new(1.0, (height - 16.0) / 2.0, 16.0, 16.0);
        let border = if state.focused || state.hovered { Color::ACCENT } else { Color::BORDER };
        let face = if state.enabled { Color::WHITE } else { Color::FORM };
        if let Some(back) = p.back {
            let area = p.area();
            p.fill_rect(area, back);
        }
        p.fill_round_rect(mark, 8.0, border);
        p.fill_round_rect(mark.inset(1.0), 7.0, face);
        if self.checked {
            p.fill_round_rect(mark.inset(4.0), 4.0, Color::ACCENT);
        }
        let ink = p.fore_color(Color::TEXT);
        p.text(&self.text, Rect::new(24.0, 0.0, width - 24.0, height), ink, Align::Left, 0.0);
    }

    fn on_mouse_up(&mut self, _at: Point, inside: bool, cx: &mut EventCx) {
        if inside {
            self.choose(cx);
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        if key == Key::Space {
            self.choose(cx);
        }
    }

    fn exclusive(&self) -> bool {
        true
    }

    fn uncheck(&mut self) -> bool {
        std::mem::replace(&mut self.checked, false)
    }
}
