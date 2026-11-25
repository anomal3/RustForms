//! Общая часть списков: строки, выбор и прокрутка.

use crate::{Align, Color, Key, Painter, Rect};

pub(crate) const ROW: f32 = 22.0;

#[derive(Default)]
pub(crate) struct Items {
    pub list: Vec<String>,
    pub selected: Option<usize>,
    /// Первая видимая строка.
    pub top: usize,
    /// Прокрутка колесом меньше строки накапливается здесь.
    wheel: f32,
}

impl Items {
    pub fn set(&mut self, list: Vec<String>) {
        self.list = list;
        self.selected = self.selected.filter(|&i| i < self.list.len());
        self.top = 0;
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index.filter(|&i| i < self.list.len());
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selected.map(|i| self.list[i].as_str())
    }

    /// Куда переведёт выбор клавиша.
    pub fn step(&self, from: Option<usize>, key: Key) -> Option<usize> {
        let last = self.list.len().checked_sub(1)?;
        Some(match key {
            Key::Up => from.map_or(0, |i| i.saturating_sub(1)),
            Key::Down => from.map_or(0, |i| (i + 1).min(last)),
            Key::Home => 0,
            Key::End => last,
            _ => return from,
        })
    }

    pub fn scroll(&mut self, lines: f32, visible: usize) {
        self.wheel += lines;
        let whole = self.wheel.trunc();
        self.wheel -= whole;
        let max = self.list.len().saturating_sub(visible);
        self.top = (self.top as f32 - whole).clamp(0.0, max as f32) as usize;
    }

    pub fn reveal(&mut self, index: usize, visible: usize) {
        if index < self.top {
            self.top = index;
        } else if visible > 0 && index >= self.top + visible {
            self.top = index + 1 - visible;
        }
    }

    /// Строка по вертикальной координате внутри списка с рамкой в 1 пиксель.
    pub fn row_at(&self, y: f32) -> Option<usize> {
        let index = self.top + ((y - 1.0) / ROW).floor().max(0.0) as usize;
        (y >= 1.0 && index < self.list.len()).then_some(index)
    }

    pub fn paint(&mut self, p: &mut Painter, highlight: Option<usize>) {
        let area = p.area();
        let enabled = p.state().enabled;
        p.fill_rect(area, if enabled { Color::WHITE } else { Color::FORM });
        let visible = visible_rows(area.height);
        self.top = self.top.min(self.list.len().saturating_sub(visible));
        let ink = if enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        for (n, index) in (self.top..self.list.len()).take(visible).enumerate() {
            let row = Rect::new(1.0, 1.0 + n as f32 * ROW, area.width - 2.0, ROW);
            if highlight == Some(index) {
                p.fill_rect(row, Color::SELECTION);
            }
            let text = Rect::new(row.x + 5.0, row.y, row.width - 10.0, row.height);
            p.text(&self.list[index], text, ink, Align::Left, 0.0);
        }
        if self.list.len() > visible {
            let track = area.height - 2.0;
            let size = (track * visible as f32 / self.list.len() as f32).max(12.0);
            let at = (track - size) * self.top as f32 / (self.list.len() - visible) as f32;
            p.fill_rect(Rect::new(area.width - 5.0, 1.0 + at, 4.0, size), Color::BORDER);
        }
        p.stroke_rect(area, Color::BORDER, 1.0);
    }
}

pub(crate) fn visible_rows(height: f32) -> usize {
    ((height - 2.0) / ROW).floor().max(0.0) as usize
}
