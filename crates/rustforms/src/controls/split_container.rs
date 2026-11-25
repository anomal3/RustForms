use crate::{
    Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

/// Ширина разделителя.
pub const SPLITTER: f32 = 5.0;
const MIN: f32 = 20.0;

/// Две панели ([`SplitPanel`]) с разделителем, который можно двигать мышью.
#[derive(Control)]
pub struct SplitContainer {
    base: ControlBase,
    distance: f32,
    horizontal: bool,
    dragging: bool,
}

impl SplitContainer {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(300.0, 160.0, false),
            distance: 100.0,
            horizontal: false,
            dragging: false,
        }
    }

    /// Размер первой панели.
    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.max(0.0);
    }

    /// `true` — разделитель горизонтальный, панели друг над другом.
    pub fn set_horizontal(&mut self, horizontal: bool) {
        self.horizontal = horizontal;
    }

    fn length(&self) -> f32 {
        if self.horizontal { self.base.bounds.height } else { self.base.bounds.width }
    }

    fn effective(&self) -> f32 {
        let max = (self.length() - SPLITTER - MIN).max(MIN);
        self.distance.clamp(MIN.min(max), max)
    }
}

impl Default for SplitContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for SplitContainer {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn sync_pages(&mut self, titles: Vec<String>) -> Option<Vec<(Rect, bool)>> {
        let (w, h) = (self.base.bounds.width, self.base.bounds.height);
        let first = self.effective();
        let rest = (self.length() - first - SPLITTER).max(0.0);
        let areas = if self.horizontal {
            [Rect::new(0.0, 0.0, w, first), Rect::new(0.0, first + SPLITTER, w, rest)]
        } else {
            [Rect::new(0.0, 0.0, first, h), Rect::new(first + SPLITTER, 0.0, rest, h)]
        };
        Some((0..titles.len()).map(|i| (areas[i.min(1)], i < 2)).collect())
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let back = p.back_color(Color::FORM);
        p.fill_rect(area, back);
        let at = self.effective() + 1.0;
        let line = if self.horizontal {
            Rect::new(0.0, at, area.width, SPLITTER - 2.0)
        } else {
            Rect::new(at, 0.0, SPLITTER - 2.0, area.height)
        };
        let state = p.state();
        let color = if state.hovered || self.dragging { Color::ACCENT } else { Color::rgb(215, 215, 215) };
        p.fill_rect(line, color);
    }

    fn on_mouse_down(&mut self, _at: Point, _mods: Modifiers, _cx: &mut EventCx) {
        // Панели закрывают всё, кроме разделителя, так что щелчок пришёл по нему.
        self.dragging = true;
    }

    fn on_mouse_drag(&mut self, at: Point, _cx: &mut EventCx) {
        let to = if self.horizontal { at.y } else { at.x };
        self.distance = to - SPLITTER / 2.0;
        self.distance = self.effective();
    }

    fn on_mouse_up(&mut self, _at: Point, _inside: bool, _cx: &mut EventCx) {
        self.dragging = false;
    }

    fn on_key(&mut self, _key: Key, _mods: Modifiers, _cx: &mut EventCx) {}
}

/// Одна из панелей [`SplitContainer`].
#[derive(Control)]
pub struct SplitPanel {
    base: ControlBase,
}

impl SplitPanel {
    pub fn new() -> Self {
        Self { base: ControlBase::new(100.0, 160.0, false) }
    }
}

impl Default for SplitPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for SplitPanel {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn page_title(&self) -> Option<String> {
        Some(String::new())
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let back = p.back_color(Color::FORM);
        p.fill_rect(area, back);
        p.stroke_rect(area, Color::BORDER, 1.0);
    }
}
