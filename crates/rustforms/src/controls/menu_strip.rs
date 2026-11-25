//! Главное меню: строка заголовков и выпадающие пункты (один уровень).

use crate::{Align, Color, Control, ControlBase, ControlImpl, EventCx, Modifiers, Painter, Point, Rect};

const PAD: f32 = 10.0;
const ROW: f32 = 24.0;
const SEPARATOR: f32 = 9.0;

struct Menu {
    title: String,
    /// `-` — разделитель.
    entries: Vec<String>,
}

impl Menu {
    fn height(entry: &str) -> f32 {
        if entry == "-" { SEPARATOR } else { ROW }
    }
}

struct Open {
    menu: usize,
    x: f32,
    width: f32,
    hot: Option<usize>,
}

#[derive(Control)]
pub struct MenuStrip {
    base: ControlBase,
    items: Vec<String>,
    menus: Vec<Menu>,
    /// Левые края заголовков; измеряются при отрисовке.
    edges: Vec<f32>,
    open: Option<Open>,
    hover: Option<usize>,
}

impl MenuStrip {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(300.0, 26.0, false),
            items: Vec::new(),
            menus: Vec::new(),
            edges: Vec::new(),
            open: None,
            hover: None,
        }
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Пункты в виде `"Меню/Пункт"`; `"Меню/-"` — разделитель, `"Меню"` без косой черты —
    /// заголовок без подпунктов.
    pub fn set_items<S: Into<String>>(&mut self, items: impl IntoIterator<Item = S>) {
        self.items = items.into_iter().map(Into::into).collect();
        self.menus.clear();
        for item in &self.items {
            let (title, entry) = match item.split_once('/') {
                Some((title, entry)) => (title.trim(), Some(entry.trim())),
                None => (item.trim(), None),
            };
            let index = match self.menus.iter().position(|m| m.title == title) {
                Some(index) => index,
                None => {
                    self.menus.push(Menu { title: title.to_owned(), entries: Vec::new() });
                    self.menus.len() - 1
                }
            };
            if let Some(entry) = entry {
                self.menus[index].entries.push(entry.to_owned());
            }
        }
        self.edges.clear();
        self.open = None;
    }

    fn title_at(&self, x: f32) -> Option<usize> {
        self.edges.windows(2).position(|e| x >= e[0] && x < e[1])
    }

    fn measure(&mut self, text: &mut crate::TextSystem) {
        let mut x = 2.0;
        self.edges = vec![x];
        for menu in &self.menus {
            x += text.width(&menu.title).ceil() + PAD * 2.0;
            self.edges.push(x);
        }
    }

    fn show(&mut self, index: usize, cx: &mut EventCx) {
        self.measure(cx.text_system());
        let entries = &self.menus[index].entries;
        if entries.is_empty() {
            self.open = None;
            cx.item_clicked(self.menus[index].title.clone());
            return;
        }
        let width = entries
            .iter()
            .map(|e| cx.text_system().width(e).ceil() + PAD * 4.0)
            .fold(140.0, f32::max);
        self.open = Some(Open { menu: index, x: self.edges[index], width, hot: None });
    }

    /// Номер пункта по вертикали внутри окошка.
    fn entry_at(&self, y: f32) -> Option<usize> {
        let menu = &self.menus[self.open.as_ref()?.menu];
        let mut top = 3.0;
        menu.entries.iter().position(|entry| {
            let bottom = top + Menu::height(entry);
            let inside = y >= top && y < bottom && entry != "-";
            top = bottom;
            inside
        })
    }
}

impl Default for MenuStrip {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for MenuStrip {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        p.fill_rect(area, Color::rgb(249, 249, 249));
        p.fill_rect(Rect::new(0.0, area.height - 1.0, area.width, 1.0), Color::rgb(220, 220, 220));
        self.measure(p.text_system());
        let hovered = p.state().hovered.then_some(self.hover).flatten();
        let open = self.open.as_ref().map(|o| o.menu);
        let ink = if p.state().enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        for (i, menu) in self.menus.iter().enumerate() {
            let rect = Rect::new(self.edges[i], 0.0, self.edges[i + 1] - self.edges[i], area.height - 1.0);
            if open == Some(i) {
                p.fill_rect(rect, Color::FACE_PRESSED);
            } else if hovered == Some(i) {
                p.fill_rect(rect, Color::FACE_HOVER);
            }
            p.text(&menu.title, rect, ink, Align::Center, 0.0);
        }
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        self.measure(cx.text_system());
        match self.title_at(at.x) {
            // Повторный щелчок по заголовку закрывает меню.
            Some(i) if self.open.as_ref().is_some_and(|o| o.menu == i) => self.open = None,
            Some(i) => self.show(i, cx),
            None => self.open = None,
        }
    }

    fn on_mouse_move(&mut self, at: Point, cx: &mut EventCx) {
        self.hover = self.title_at(at.x);
        // Пока меню открыто, наведение на соседний заголовок переключает его.
        if let (Some(open), Some(i)) = (&self.open, self.hover)
            && open.menu != i
            && !self.menus[i].entries.is_empty()
        {
            self.show(i, cx);
        }
    }

    fn popup_size(&self) -> Option<(f32, f32)> {
        let open = self.open.as_ref()?;
        let height: f32 = self.menus[open.menu].entries.iter().map(|e| Menu::height(e)).sum();
        Some((open.width, height + 6.0))
    }

    fn popup_offset(&self) -> f32 {
        self.open.as_ref().map_or(0.0, |o| o.x)
    }

    fn close_popup(&mut self) {
        self.open = None;
    }

    fn on_paint_popup(&mut self, p: &mut Painter) {
        let Some(open) = &self.open else { return };
        let area = p.area();
        p.fill_rect(area, Color::WHITE);
        let mut top = 3.0;
        for (i, entry) in self.menus[open.menu].entries.iter().enumerate() {
            let height = Menu::height(entry);
            if entry == "-" {
                let line = Rect::new(8.0, top + height / 2.0, area.width - 16.0, 1.0);
                p.fill_rect(line, Color::rgb(215, 215, 215));
            } else {
                if open.hot == Some(i) {
                    p.fill_rect(Rect::new(2.0, top, area.width - 4.0, height), Color::SELECTION);
                }
                let text = Rect::new(PAD * 2.0, top, area.width - PAD * 3.0, height);
                p.text(entry, text, Color::TEXT, Align::Left, 0.0);
            }
            top += height;
        }
        p.stroke_rect(area, Color::BORDER, 1.0);
    }

    fn on_popup_mouse_move(&mut self, at: Point) {
        let hot = self.entry_at(at.y);
        if let Some(open) = &mut self.open {
            open.hot = hot;
        }
    }

    fn on_popup_mouse_down(&mut self, at: Point, cx: &mut EventCx) {
        let Some(index) = self.entry_at(at.y) else { return };
        let Some(open) = self.open.take() else { return };
        let menu = &self.menus[open.menu];
        cx.item_clicked(format!("{}/{}", menu.title, menu.entries[index]));
    }
}
