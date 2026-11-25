use crate::{
    Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
    TextSystem,
};

/// Высота полосы с ярлычками вкладок.
pub const TAB_HEADER: f32 = 28.0;
const PAD: f32 = 12.0;

/// Вкладки. Страницы ([`TabPage`]) добавляются внутрь: `view.add_to(tabs, ...)`.
#[derive(Control)]
pub struct TabControl {
    base: ControlBase,
    titles: Vec<String>,
    selected: usize,
    /// Левые края ярлычков.
    edges: Vec<f32>,
}

impl TabControl {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(300.0, 180.0, true),
            titles: Vec::new(),
            selected: 0,
            edges: Vec::new(),
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn set_selected_index(&mut self, index: usize) {
        self.selected = index;
    }

    /// Левые края ярлычков и правый край последнего — после отрисовки.
    pub fn tab_edges(&self) -> &[f32] {
        &self.edges
    }

    fn measure(&mut self, text: &mut TextSystem) {
        let mut x = 0.0;
        self.edges = vec![x];
        for title in &self.titles {
            x += text.width(title).ceil() + PAD * 2.0;
            self.edges.push(x);
        }
    }

    fn choose(&mut self, index: usize, cx: &mut EventCx) {
        if index < self.titles.len() && index != self.selected {
            self.selected = index;
            cx.selected_changed(Some(index), self.titles[index].clone());
        }
    }
}

impl Default for TabControl {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for TabControl {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn sync_pages(&mut self, titles: Vec<String>) -> Option<Vec<(Rect, bool)>> {
        self.titles = titles;
        self.selected = self.selected.min(self.titles.len().saturating_sub(1));
        let bounds = self.base.bounds;
        let area = Rect::new(0.0, TAB_HEADER, bounds.width, (bounds.height - TAB_HEADER).max(0.0));
        Some((0..self.titles.len()).map(|i| (area, i == self.selected)).collect())
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let focused = p.state().focused;
        let back = p.back_color(Color::FORM);
        p.fill_rect(Rect::new(0.0, 0.0, area.width, TAB_HEADER), back);
        self.measure(p.text_system());
        let ink = p.fore_color(Color::TEXT);
        for (i, title) in self.titles.iter().enumerate() {
            let tab = Rect::new(self.edges[i], 2.0, self.edges[i + 1] - self.edges[i], TAB_HEADER - 2.0);
            let active = i == self.selected;
            p.fill_rect(tab, if active { Color::FACE } else { Color::rgb(229, 229, 229) });
            p.stroke_rect(tab, Color::BORDER, 1.0);
            if active {
                p.fill_rect(Rect::new(tab.x, tab.y, tab.width, 2.0), Color::ACCENT);
                if focused {
                    p.fill_rect(Rect::new(tab.x + 8.0, TAB_HEADER - 5.0, tab.width - 16.0, 1.0), ink);
                }
            }
            p.text(title, tab, ink, Align::Center, 0.0);
        }
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        if at.y < TAB_HEADER {
            self.measure(cx.text_system());
            if let Some(index) = self.edges.windows(2).position(|e| at.x >= e[0] && at.x < e[1]) {
                self.choose(index, cx);
            }
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        match key {
            Key::Left => self.choose(self.selected.saturating_sub(1), cx),
            Key::Right => self.choose(self.selected + 1, cx),
            _ => {}
        }
    }
}

/// Страница вкладок: контейнер, занимающий всё место под ярлычками.
#[derive(Control)]
pub struct TabPage {
    base: ControlBase,
    text: String,
}

impl TabPage {
    pub fn new(text: impl Into<String>) -> Self {
        Self { base: ControlBase::new(300.0, 152.0, false), text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl ControlImpl for TabPage {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn page_title(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let back = p.back_color(Color::FACE);
        p.fill_rect(area, back);
        p.stroke_rect(area, Color::BORDER, 1.0);
    }
}
