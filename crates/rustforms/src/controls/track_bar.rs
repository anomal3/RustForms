use crate::{
    Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

const THUMB: f32 = 14.0;

/// Ползунок по горизонтали.
#[derive(Control)]
pub struct TrackBar {
    base: ControlBase,
    value: i32,
    minimum: i32,
    maximum: i32,
}

impl TrackBar {
    pub fn new() -> Self {
        Self { base: ControlBase::new(200.0, 28.0, true), value: 0, minimum: 0, maximum: 100 }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(self.minimum, self.maximum);
    }

    pub fn set_range(&mut self, minimum: i32, maximum: i32) {
        self.minimum = minimum;
        self.maximum = maximum.max(minimum);
        self.set_value(self.value);
    }

    fn change(&mut self, value: i32, cx: &mut EventCx) {
        let old = self.value;
        self.set_value(value);
        if self.value != old {
            cx.value_changed(self.value);
        }
    }

    fn span(&self) -> f64 {
        f64::from(self.maximum) - f64::from(self.minimum)
    }

    fn point(&mut self, x: f32, cx: &mut EventCx) {
        let track = (cx.size().0 - THUMB).max(1.0);
        let part = f64::from(((x - THUMB / 2.0) / track).clamp(0.0, 1.0));
        let value = f64::from(self.minimum) + (part * self.span()).round();
        self.change(value as i32, cx);
    }
}

impl Default for TrackBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for TrackBar {
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
        let part = if self.span() > 0.0 {
            ((f64::from(self.value) - f64::from(self.minimum)) / self.span()) as f32
        } else {
            0.0
        };
        let middle = area.height / 2.0;
        let x = part * (area.width - THUMB).max(0.0);
        let track = Rect::new(THUMB / 2.0, middle - 2.0, (area.width - THUMB).max(0.0), 4.0);
        p.fill_round_rect(track, 2.0, Color::BORDER);
        let accent = if state.enabled { Color::ACCENT } else { Color::TEXT_DISABLED };
        p.fill_round_rect(Rect::new(track.x, track.y, x, track.height), 2.0, accent);
        let thumb = Rect::new(x, middle - THUMB / 2.0, THUMB, THUMB);
        p.fill_round_rect(thumb, THUMB / 2.0, accent);
        if state.focused || state.hovered {
            p.fill_round_rect(thumb.inset(4.0), THUMB / 2.0 - 4.0, Color::WHITE);
        }
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        self.point(at.x, cx);
    }

    fn on_mouse_drag(&mut self, at: Point, cx: &mut EventCx) {
        self.point(at.x, cx);
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        match key {
            Key::Left | Key::Down => self.change(self.value.saturating_sub(1), cx),
            Key::Right | Key::Up => self.change(self.value.saturating_add(1), cx),
            Key::Home => self.change(self.minimum, cx),
            Key::End => self.change(self.maximum, cx),
            _ => {}
        }
    }

    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        if lines != 0.0 {
            self.change(self.value.saturating_add(if lines > 0.0 { 1 } else { -1 }), cx);
        }
    }
}
