use std::collections::HashSet;

use crate::{
    Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

const INDENT: f32 = 16.0;

/// Дерево. Узлы задаются путями: `"Корень/Ветка/Лист"`.
#[derive(Control)]
pub struct TreeView {
    base: ControlBase,
    /// Все узлы в порядке показа — пути от корня.
    nodes: Vec<String>,
    expanded: HashSet<String>,
    selected: Option<String>,
    top: usize,
    wheel: f32,
    row_height: f32,
}

fn parent_of(path: &str) -> Option<&str> {
    path.rfind('/').map(|i| &path[..i])
}

fn depth(path: &str) -> usize {
    path.matches('/').count()
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(180.0, 150.0, true),
            nodes: Vec::new(),
            expanded: HashSet::new(),
            selected: None,
            top: 0,
            wheel: 0.0,
            row_height: 22.0,
        }
    }

    /// Недостающие родители добавляются сами; всё сначала свёрнуто.
    pub fn set_items<S: Into<String>>(&mut self, items: impl IntoIterator<Item = S>) {
        self.nodes.clear();
        for item in items {
            self.add_item(item);
        }
        self.expanded.clear();
        self.selected = None;
        self.top = 0;
    }

    pub fn add_item(&mut self, path: impl Into<String>) {
        let path: String = path.into();
        let mut prefix = String::new();
        for part in path.split('/').map(str::trim).filter(|p| !p.is_empty()) {
            let parent = prefix.clone();
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            if self.nodes.contains(&prefix) {
                continue;
            }
            // После последнего потомка родителя, чтобы ветка оставалась вместе.
            let at = if parent.is_empty() {
                self.nodes.len()
            } else {
                let inside = format!("{parent}/");
                self.nodes
                    .iter()
                    .rposition(|n| *n == parent || n.starts_with(&inside))
                    .map_or(self.nodes.len(), |i| i + 1)
            };
            self.nodes.insert(at, prefix.clone());
        }
    }

    pub fn selected_path(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn expand(&mut self, path: &str) {
        self.expanded.insert(path.to_owned());
    }

    pub fn collapse(&mut self, path: &str) {
        self.expanded.remove(path);
    }

    pub fn expand_all(&mut self) {
        self.expanded = self.nodes.iter().cloned().collect();
    }

    fn has_children(&self, path: &str) -> bool {
        self.nodes.iter().any(|n| parent_of(n) == Some(path))
    }

    /// Узлы, все предки которых раскрыты.
    fn shown(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .map(String::as_str)
            .filter(|node| {
                let mut at = parent_of(node);
                while let Some(parent) = at {
                    if !self.expanded.contains(parent) {
                        return false;
                    }
                    at = parent_of(parent);
                }
                true
            })
            .collect()
    }

    fn visible(&self, height: f32) -> usize {
        ((height - 2.0) / self.row_height).floor().max(0.0) as usize
    }

    fn choose(&mut self, path: String, cx: &mut EventCx) {
        let shown = self.shown();
        let Some(index) = shown.iter().position(|n| *n == path) else { return };
        let visible = self.visible(cx.size().1);
        if index < self.top {
            self.top = index;
        } else if visible > 0 && index >= self.top + visible {
            self.top = index + 1 - visible;
        }
        if self.selected.as_ref() != Some(&path) {
            self.selected = Some(path.clone());
            cx.selected_changed(Some(index), path);
        }
    }

    fn toggle(&mut self, path: &str) {
        if !self.expanded.remove(path) {
            self.expanded.insert(path.to_owned());
        }
    }
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for TreeView {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let enabled = p.state().enabled;
        let back = p.back_color(if enabled { Color::WHITE } else { Color::FORM });
        p.fill_rect(area, back);
        self.row_height = p.text_system().line_height() + 2.0;
        let height = self.row_height;
        let visible = self.visible(area.height);
        let ink = p.fore_color(Color::TEXT);
        let shown = self.shown();
        let top = self.top.min(shown.len().saturating_sub(visible));
        for (n, node) in shown.iter().skip(top).take(visible).enumerate() {
            let y = 1.0 + n as f32 * height;
            if self.selected.as_deref() == Some(*node) {
                p.fill_rect(Rect::new(1.0, y, area.width - 2.0, height), Color::SELECTION);
            }
            let x = 4.0 + depth(node) as f32 * INDENT;
            if self.has_children(node) {
                // «+» у свёрнутой ветки, «−» у раскрытой.
                let (cx, cy) = (x + 5.0, y + height / 2.0);
                p.stroke_rect(Rect::new(cx - 5.0, cy - 5.0, 11.0, 11.0), Color::BORDER, 1.0);
                p.fill_rect(Rect::new(cx - 2.0, cy, 5.0, 1.0), ink);
                if !self.expanded.contains(*node) {
                    p.fill_rect(Rect::new(cx, cy - 2.0, 1.0, 5.0), ink);
                }
            }
            let name = node.rsplit('/').next().unwrap_or(node);
            let text = Rect::new(x + INDENT, y, (area.width - x - INDENT).max(0.0), height);
            p.text(name, text, ink, Align::Left, 0.0);
        }
        let count = shown.len();
        if count > visible && visible > 0 {
            let track = area.height - 2.0;
            let size = (track * visible as f32 / count as f32).max(12.0);
            let at = (track - size) * top as f32 / (count - visible) as f32;
            p.fill_rect(Rect::new(area.width - 5.0, 1.0 + at, 4.0, size), Color::BORDER);
        }
        self.top = top;
        p.stroke_rect(area, Color::BORDER, 1.0);
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        let line = ((at.y - 1.0) / self.row_height).floor();
        if line < 0.0 {
            return;
        }
        let Some(node) = self.shown().get(self.top + line as usize).map(|n| (*n).to_owned()) else {
            return;
        };
        let x = 4.0 + depth(&node) as f32 * INDENT;
        if self.has_children(&node) && at.x >= x - 2.0 && at.x < x + INDENT {
            self.toggle(&node);
        } else {
            self.choose(node, cx);
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        let shown: Vec<String> = self.shown().into_iter().map(str::to_owned).collect();
        if shown.is_empty() {
            return;
        }
        let current = self.selected.clone();
        let index = current.as_ref().and_then(|s| shown.iter().position(|n| n == s));
        let to = match key {
            Key::Up => Some(index.map_or(0, |i| i.saturating_sub(1))),
            Key::Down => Some(index.map_or(0, |i| (i + 1).min(shown.len() - 1))),
            Key::Home => Some(0),
            Key::End => Some(shown.len() - 1),
            _ => None,
        };
        if let Some(to) = to {
            self.choose(shown[to].clone(), cx);
            return;
        }
        let Some(current) = current else { return };
        match key {
            Key::Right if self.has_children(&current) => self.expand(&current),
            Key::Left if self.expanded.contains(&current) => self.collapse(&current),
            Key::Left => {
                if let Some(parent) = parent_of(&current) {
                    self.choose(parent.to_owned(), cx);
                }
            }
            _ => {}
        }
    }

    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        self.wheel += lines;
        let whole = self.wheel.trunc();
        self.wheel -= whole;
        let max = self.shown().len().saturating_sub(self.visible(cx.size().1));
        self.top = (self.top as f32 - whole).clamp(0.0, max as f32) as usize;
    }
}
