use crate::{
    Align, Color, Control, ControlBase, ControlImpl, EventCx, Modifiers, Painter, Point, Rect,
    TextSystem,
};

const PAD: f32 = 10.0;
const GAP: f32 = 9.0;

/// Панель инструментов: ряд кнопок-надписей, `-` — разделитель. Щелчок даёт `item_clicked`.
#[derive(Control)]
pub struct ToolStrip {
    base: ControlBase,
    items: Vec<String>,
    edges: Vec<f32>,
    hover: Option<usize>,
    pressed: Option<usize>,
}

impl ToolStrip {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(300.0, 30.0, false),
            items: Vec::new(),
            edges: Vec::new(),
            hover: None,
            pressed: None,
        }
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    pub fn set_items<S: Into<String>>(&mut self, items: impl IntoIterator<Item = S>) {
        self.items = items.into_iter().map(Into::into).collect();
        self.edges.clear();
    }

    fn measure(&mut self, text: &mut TextSystem) {
        let mut x = 4.0;
        self.edges = vec![x];
        for item in &self.items {
            x += if item == "-" { GAP } else { text.width(item).ceil() + PAD * 2.0 };
            self.edges.push(x);
        }
    }

    fn item_at(&self, x: f32) -> Option<usize> {
        let index = self.edges.windows(2).position(|e| x >= e[0] && x < e[1])?;
        (self.items[index] != "-").then_some(index)
    }
}

impl Default for ToolStrip {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for ToolStrip {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let back = p.back_color(Color::rgb(245, 245, 245));
        p.fill_rect(area, back);
        p.fill_rect(Rect::new(0.0, area.height - 1.0, area.width, 1.0), Color::rgb(215, 215, 215));
        self.measure(p.text_system());
        let state = p.state();
        let ink = p.fore_color(Color::TEXT);
        for (i, item) in self.items.iter().enumerate() {
            let x = self.edges[i];
            let width = self.edges[i + 1] - x;
            if item == "-" {
                p.fill_rect(Rect::new(x + GAP / 2.0, 5.0, 1.0, area.height - 10.0), Color::BORDER);
                continue;
            }
            let button = Rect::new(x, 3.0, width, area.height - 7.0);
            if state.hovered && self.hover == Some(i) {
                let down = state.pressed && self.pressed == Some(i);
                p.fill_round_rect(button, 3.0, if down { Color::FACE_PRESSED } else { Color::FACE_HOVER });
            }
            p.text(item, button, ink, Align::Center, 0.0);
        }
    }

    fn on_mouse_move(&mut self, at: Point, _cx: &mut EventCx) {
        self.hover = self.item_at(at.x);
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        self.measure(cx.text_system());
        self.pressed = self.item_at(at.x);
        self.hover = self.pressed;
    }

    fn on_mouse_up(&mut self, at: Point, inside: bool, cx: &mut EventCx) {
        let pressed = self.pressed.take();
        if inside
            && let Some(index) = pressed
            && self.item_at(at.x) == pressed
        {
            cx.item_clicked(self.items[index].clone());
        }
    }
}
