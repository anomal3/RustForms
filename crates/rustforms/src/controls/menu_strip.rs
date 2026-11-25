//! Главное меню: строка заголовков, выпадающие пункты и подменю.

use crate::{Align, Color, Control, ControlBase, ControlImpl, EventCx, Modifiers, Painter, Point, Rect};

const PAD: f32 = 10.0;
const ROW: f32 = 24.0;
const SEPARATOR: f32 = 9.0;
/// Насколько подменю наезжает на родительскую колонку.
const OVERLAP: f32 = 2.0;

#[derive(Default)]
struct Node {
    title: String,
    children: Vec<Node>,
}

impl Node {
    fn height(&self) -> f32 {
        if self.title == "-" { SEPARATOR } else { ROW }
    }

    fn insert(list: &mut Vec<Node>, parts: &[&str]) {
        let Some((first, rest)) = parts.split_first() else { return };
        // Разделители не объединяются: их может быть несколько.
        let found = list.iter().position(|n| n.title == *first && *first != "-");
        let index = found.unwrap_or_else(|| {
            list.push(Node { title: (*first).to_owned(), children: Vec::new() });
            list.len() - 1
        });
        Node::insert(&mut list[index].children, rest);
    }
}

struct Open {
    /// Номер заголовка и дальше — номера пунктов с раскрытыми подменю.
    path: Vec<usize>,
    x: f32,
    /// Подсвеченный пункт: колонка и номер.
    hot: Option<(usize, usize)>,
}

#[derive(Control)]
pub struct MenuStrip {
    base: ControlBase,
    items: Vec<String>,
    menus: Vec<Node>,
    /// Левые края заголовков; измеряются при отрисовке.
    edges: Vec<f32>,
    /// Ширина колонки для каждого раскрытого уровня.
    widths: Vec<f32>,
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
            widths: Vec::new(),
            open: None,
            hover: None,
        }
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Пункты в виде `"Меню/Пункт"` или `"Меню/Подменю/Пункт"`; `"Меню/-"` — разделитель,
    /// `"Меню"` без косой черты — заголовок без подпунктов.
    pub fn set_items<S: Into<String>>(&mut self, items: impl IntoIterator<Item = S>) {
        self.items = items.into_iter().map(Into::into).collect();
        self.menus.clear();
        for item in &self.items {
            let parts: Vec<&str> = item.split('/').map(str::trim).collect();
            Node::insert(&mut self.menus, &parts);
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

    /// Узел, в котором лежат пункты колонки `level`.
    fn column(&self, path: &[usize], level: usize) -> &Node {
        let mut node = &self.menus[path[0]];
        for &index in &path[1..=level] {
            node = &node.children[index];
        }
        node
    }

    /// Пересчитывает ширины раскрытых колонок.
    fn fit(&mut self, text: &mut crate::TextSystem) {
        let Some(open) = &self.open else { return };
        let path = open.path.clone();
        self.widths = (0..path.len())
            .map(|level| {
                self.column(&path, level)
                    .children
                    .iter()
                    .map(|n| text.width(&n.title).ceil() + PAD * 5.0)
                    .fold(140.0, f32::max)
            })
            .collect();
    }

    /// Прямоугольники раскрытых колонок внутри окошка.
    fn columns(&self) -> Vec<Rect> {
        let Some(open) = &self.open else { return Vec::new() };
        let mut rects: Vec<Rect> = Vec::new();
        for level in 0..open.path.len() {
            let node = self.column(&open.path, level);
            let height: f32 = node.children.iter().map(Node::height).sum::<f32>() + 6.0;
            let (x, y) = match rects.last() {
                None => (0.0, 0.0),
                Some(previous) => {
                    let parent = self.column(&open.path, level - 1);
                    let above: f32 =
                        parent.children[..open.path[level]].iter().map(Node::height).sum();
                    (previous.x + previous.width - OVERLAP, previous.y + above)
                }
            };
            rects.push(Rect::new(x, y, self.widths.get(level).copied().unwrap_or(140.0), height));
        }
        rects
    }

    /// Пункт под точкой окошка: колонка и номер; разделители не считаются.
    fn entry_at(&self, at: Point) -> Option<(usize, usize)> {
        let open = self.open.as_ref()?;
        let columns = self.columns();
        // Подменю лежит поверх, поэтому поиск с последней колонки.
        let level = columns.iter().rposition(|r| r.contains(at))?;
        let mut top = columns[level].y + 3.0;
        for (i, node) in self.column(&open.path, level).children.iter().enumerate() {
            let bottom = top + node.height();
            if at.y >= top && at.y < bottom {
                return (node.title != "-").then_some((level, i));
            }
            top = bottom;
        }
        None
    }

    fn show(&mut self, index: usize, cx: &mut EventCx) {
        self.measure(cx.text_system());
        if self.menus[index].children.is_empty() {
            self.open = None;
            cx.item_clicked(self.menus[index].title.clone());
            return;
        }
        self.open = Some(Open { path: vec![index], x: self.edges[index], hot: None });
        self.fit(cx.text_system());
    }

    /// Раскрывает подменю пункта или сворачивает более глубокие.
    fn expand(&mut self, level: usize, index: usize, cx: &mut EventCx) {
        let Some(open) = &self.open else { return };
        let branch = !self.column(&open.path, level).children[index].children.is_empty();
        let Some(open) = &mut self.open else { return };
        open.hot = Some((level, index));
        open.path.truncate(level + 1);
        if branch {
            open.path.push(index);
        }
        self.fit(cx.text_system());
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
        let open = self.open.as_ref().map(|o| o.path[0]);
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
            Some(i) if self.open.as_ref().is_some_and(|o| o.path[0] == i) => self.open = None,
            Some(i) => self.show(i, cx),
            None => self.open = None,
        }
    }

    fn on_mouse_move(&mut self, at: Point, cx: &mut EventCx) {
        self.hover = self.title_at(at.x);
        // Пока меню открыто, наведение на соседний заголовок переключает его.
        if let (Some(open), Some(i)) = (&self.open, self.hover)
            && open.path[0] != i
            && !self.menus[i].children.is_empty()
        {
            self.show(i, cx);
        }
    }

    fn popup_size(&self) -> Option<(f32, f32)> {
        self.open.as_ref()?;
        // Общая рамка всех колонок.
        let (width, height) = self.columns().iter().fold((0.0f32, 0.0f32), |(w, h), r| {
            (w.max(r.x + r.width), h.max(r.y + r.height))
        });
        Some((width, height))
    }

    fn popup_offset(&self) -> f32 {
        self.open.as_ref().map_or(0.0, |o| o.x)
    }

    fn close_popup(&mut self) {
        self.open = None;
    }

    fn on_paint_popup(&mut self, p: &mut Painter) {
        let columns = self.columns();
        let Some(open) = &self.open else { return };
        for (level, column) in columns.iter().enumerate() {
            p.fill_rect(*column, Color::WHITE);
            let mut top = column.y + 3.0;
            for (i, node) in self.column(&open.path, level).children.iter().enumerate() {
                let height = node.height();
                if node.title == "-" {
                    let line = Rect::new(column.x + 8.0, top + height / 2.0, column.width - 16.0, 1.0);
                    p.fill_rect(line, Color::rgb(215, 215, 215));
                } else {
                    let expanded = open.path.get(level + 1) == Some(&i);
                    if open.hot == Some((level, i)) || expanded {
                        let row = Rect::new(column.x + 2.0, top, column.width - 4.0, height);
                        p.fill_rect(row, Color::SELECTION);
                    }
                    let text = Rect::new(column.x + PAD * 2.0, top, column.width - PAD * 3.0, height);
                    p.text(&node.title, text, Color::TEXT, Align::Left, 0.0);
                    if !node.children.is_empty() {
                        // Треугольник «есть подменю» из полосок.
                        let (x, y) = (column.x + column.width - 14.0, top + height / 2.0);
                        for k in 0..4 {
                            let k = k as f32;
                            p.fill_rect(Rect::new(x + k, y - 4.0 + k, 1.0, 8.0 - k * 2.0), Color::TEXT);
                        }
                    }
                }
                top += height;
            }
            p.stroke_rect(*column, Color::BORDER, 1.0);
        }
    }

    fn on_popup_mouse_move(&mut self, at: Point, cx: &mut EventCx) {
        match self.entry_at(at) {
            Some((level, index)) => self.expand(level, index, cx),
            None => {
                if let Some(open) = &mut self.open {
                    open.hot = None;
                }
            }
        }
    }

    fn on_popup_mouse_down(&mut self, at: Point, cx: &mut EventCx) {
        let Some((level, index)) = self.entry_at(at) else {
            // Мимо колонок — меню закрывается.
            if !self.columns().iter().any(|r| r.contains(at)) {
                self.open = None;
            }
            return;
        };
        let Some(open) = &self.open else { return };
        let node = &self.column(&open.path, level).children[index];
        if !node.children.is_empty() {
            self.expand(level, index, cx);
            return;
        }
        let mut names = vec![self.menus[open.path[0]].title.as_str()];
        for depth in 1..=level {
            names.push(&self.column(&open.path, depth).title);
        }
        names.push(&node.title);
        let path = names.join("/");
        self.open = None;
        cx.item_clicked(path);
    }
}
