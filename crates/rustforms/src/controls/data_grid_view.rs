use std::cmp::Ordering;

use crate::{
    Align, Color, Control, ControlBase, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

/// Насколько близко к границе столбца надо нажать, чтобы менять ширину.
const GRIP: f32 = 4.0;
const MIN_COLUMN: f32 = 24.0;

struct Edit {
    row: usize,
    column: usize,
    text: String,
}

/// Таблица: выбор строки, прокрутка, сортировка щелчком по заголовку, изменение ширины
/// столбцов и (если включено) правка ячеек.
///
/// Правка начинается повторным щелчком по выбранной ячейке, клавишей Enter или набором
/// текста; Enter или уход фокуса сохраняют, Escape отменяет.
#[derive(Control)]
pub struct DataGridView {
    base: ControlBase,
    columns: Vec<String>,
    /// Доли ширины столбцов.
    shares: Vec<f32>,
    rows: Vec<Vec<String>>,
    selected: Option<usize>,
    column: usize,
    /// Первая видимая строка.
    top: usize,
    wheel: f32,
    row_height: f32,
    editable: bool,
    sortable: bool,
    /// Столбец сортировки и направление (`true` — по возрастанию).
    sorted: Option<(usize, bool)>,
    edit: Option<Edit>,
    /// Столбец, правую границу которого тянут.
    resizing: Option<usize>,
}

/// Числа сравниваются как числа, остальное — как текст без учёта регистра.
fn compare(a: &str, b: &str) -> Ordering {
    match (a.trim().replace(',', ".").parse::<f64>(), b.trim().replace(',', ".").parse::<f64>()) {
        (Ok(a), Ok(b)) => a.total_cmp(&b),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

impl DataGridView {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(320.0, 150.0, true),
            columns: Vec::new(),
            shares: Vec::new(),
            rows: Vec::new(),
            selected: None,
            column: 0,
            top: 0,
            wheel: 0.0,
            row_height: 22.0,
            editable: false,
            sortable: true,
            sorted: None,
            edit: None,
            resizing: None,
        }
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    pub fn set_columns<S: Into<String>>(&mut self, columns: impl IntoIterator<Item = S>) {
        self.columns = columns.into_iter().map(Into::into).collect();
        self.shares = vec![1.0; self.columns.len()];
        self.sorted = None;
        self.edit = None;
        self.column = 0;
    }

    pub fn add_row<S: Into<String>>(&mut self, cells: impl IntoIterator<Item = S>) {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self.sorted = None;
    }

    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    pub fn cell(&self, row: usize, column: usize) -> Option<&str> {
        self.rows.get(row)?.get(column).map(String::as_str)
    }

    pub fn set_cell(&mut self, row: usize, column: usize, text: impl Into<String>) {
        if let Some(cells) = self.rows.get_mut(row)
            && column < self.columns.len()
        {
            cells.resize(cells.len().max(column + 1), String::new());
            cells[column] = text.into();
        }
    }

    pub fn clear_rows(&mut self) {
        self.rows.clear();
        self.selected = None;
        self.edit = None;
        self.top = 0;
    }

    pub fn remove_row(&mut self, index: usize) {
        if index < self.rows.len() {
            self.rows.remove(index);
            self.selected = None;
            self.edit = None;
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.selected = index.filter(|&i| i < self.rows.len());
        self.edit = None;
    }

    pub fn selected_row(&self) -> Option<&[String]> {
        self.selected.map(|i| self.rows[i].as_slice())
    }

    /// Разрешить правку ячеек.
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
        self.edit = None;
    }

    /// Сортировать ли щелчком по заголовку (по умолчанию да).
    pub fn set_sortable(&mut self, sortable: bool) {
        self.sortable = sortable;
    }

    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Сортирует строки; выбранная остаётся выбранной.
    pub fn sort_by(&mut self, column: usize, ascending: bool) {
        let mut order: Vec<usize> = (0..self.rows.len()).collect();
        order.sort_by(|&a, &b| {
            let cell = |row: usize| self.rows[row].get(column).map_or("", String::as_str);
            let by = compare(cell(a), cell(b));
            if ascending { by } else { by.reverse() }
        });
        self.selected = self.selected.and_then(|s| order.iter().position(|&i| i == s));
        let mut old: Vec<Option<Vec<String>>> =
            std::mem::take(&mut self.rows).into_iter().map(Some).collect();
        self.rows = order.into_iter().filter_map(|i| old[i].take()).collect();
        self.sorted = Some((column, ascending));
    }

    fn visible(&self, height: f32) -> usize {
        // Одна строка занята заголовком.
        (((height - 2.0) / self.row_height).floor() as usize).saturating_sub(1)
    }

    /// Левые края столбцов и правый край последнего.
    fn edges(&self, width: f32) -> Vec<f32> {
        let total: f32 = self.shares.iter().sum();
        let mut x = 1.0;
        let mut edges = vec![x];
        for share in &self.shares {
            x += (width - 2.0) * share / total.max(0.001);
            edges.push(x);
        }
        edges
    }

    fn reveal(&mut self, index: usize, height: f32) {
        let visible = self.visible(height);
        if index < self.top {
            self.top = index;
        } else if visible > 0 && index >= self.top + visible {
            self.top = index + 1 - visible;
        }
    }

    fn choose(&mut self, index: usize, cx: &mut EventCx) {
        if index >= self.rows.len() {
            return;
        }
        let changed = self.selected != Some(index);
        self.selected = Some(index);
        self.reveal(index, cx.size().1);
        if changed {
            let first = self.rows[index].first().cloned().unwrap_or_default();
            cx.selected_changed(Some(index), first);
        }
    }

    fn start_edit(&mut self, initial: Option<String>) {
        if let Some(row) = self.selected
            && self.editable
            && self.column < self.columns.len()
        {
            let text =
                initial.unwrap_or_else(|| self.cell(row, self.column).unwrap_or("").to_owned());
            self.edit = Some(Edit { row, column: self.column, text });
        }
    }

    fn commit(&mut self, cx: &mut EventCx) {
        let Some(edit) = self.edit.take() else { return };
        if self.cell(edit.row, edit.column).unwrap_or("") != edit.text {
            self.set_cell(edit.row, edit.column, edit.text.clone());
            cx.cell_changed(edit.row, edit.column, edit.text);
        }
    }
}

impl Default for DataGridView {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for DataGridView {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        let state = p.state();
        let back = p.back_color(if state.enabled { Color::WHITE } else { Color::FORM });
        p.fill_rect(area, back);
        self.row_height = p.text_system().line_height() + 4.0;
        let height = self.row_height;
        let visible = self.visible(area.height);
        self.top = self.top.min(self.rows.len().saturating_sub(visible));
        let ink = p.fore_color(Color::TEXT);
        let edges = self.edges(area.width);
        let cell = |column: usize, y: f32| {
            Rect::new(edges[column] + 5.0, y, (edges[column + 1] - edges[column] - 8.0).max(0.0), height)
        };
        p.fill_rect(Rect::new(1.0, 1.0, area.width - 2.0, height), Color::rgb(236, 236, 236));
        for (i, title) in self.columns.iter().enumerate() {
            p.text(title, cell(i, 1.0), ink, Align::Left, 0.0);
            if let Some((column, ascending)) = self.sorted
                && column == i
            {
                // Треугольник направления сортировки.
                let x = edges[i + 1] - 10.0;
                let y = 1.0 + height / 2.0 - 2.0;
                for k in 0..4 {
                    let k = k as f32;
                    let half = if ascending { k } else { 3.0 - k };
                    p.fill_rect(Rect::new(x - half, y + k, half * 2.0 + 1.0, 1.0), ink);
                }
            }
        }
        for (n, row) in self.rows.iter().skip(self.top).take(visible).enumerate() {
            let index = self.top + n;
            let y = 1.0 + (n + 1) as f32 * height;
            if self.selected == Some(index) {
                p.fill_rect(Rect::new(1.0, y, area.width - 2.0, height), Color::SELECTION);
                if self.editable && state.focused && self.column < self.columns.len() {
                    let w = edges[self.column + 1] - edges[self.column];
                    p.stroke_rect(Rect::new(edges[self.column], y, w, height), Color::ACCENT, 1.0);
                }
            }
            p.fill_rect(Rect::new(1.0, y + height - 1.0, area.width - 2.0, 1.0), Color::FORM);
            for (i, text) in row.iter().take(self.columns.len()).enumerate() {
                p.text(text, cell(i, y), ink, Align::Left, 0.0);
            }
            if let Some(edit) = &self.edit
                && edit.row == index
            {
                let w = edges[edit.column + 1] - edges[edit.column];
                let box_ = Rect::new(edges[edit.column], y, w, height);
                p.fill_rect(box_, Color::WHITE);
                p.stroke_rect(box_, Color::ACCENT, 2.0);
                let inner = cell(edit.column, y);
                let width = p.text_system().width(&edit.text);
                // Конец набираемого текста всегда виден.
                let scroll = (width - inner.width + 2.0).max(0.0);
                p.text(&edit.text, inner, Color::TEXT, Align::Left, scroll);
                let caret = inner.x + width - scroll;
                p.fill_rect(Rect::new(caret, y + 4.0, 1.0, height - 8.0), Color::TEXT);
            }
        }
        let grid = Color::rgb(215, 215, 215);
        for edge in edges.iter().take(self.columns.len()).skip(1) {
            p.fill_rect(Rect::new(*edge, 1.0, 1.0, area.height - 2.0), grid);
        }
        p.fill_rect(Rect::new(1.0, height, area.width - 2.0, 1.0), Color::BORDER);
        if self.rows.len() > visible && visible > 0 {
            let track = area.height - 2.0 - height;
            let size = (track * visible as f32 / self.rows.len() as f32).max(12.0);
            let at = (track - size) * self.top as f32 / (self.rows.len() - visible) as f32;
            p.fill_rect(Rect::new(area.width - 5.0, 1.0 + height + at, 4.0, size), Color::BORDER);
        }
        p.stroke_rect(area, Color::BORDER, 1.0);
    }

    fn on_mouse_down(&mut self, at: Point, _mods: Modifiers, cx: &mut EventCx) {
        let edges = self.edges(cx.size().0);
        let line = ((at.y - 1.0) / self.row_height).floor();
        let column = (0..self.columns.len()).find(|&i| at.x >= edges[i] && at.x < edges[i + 1]);
        if line < 1.0 {
            self.commit(cx);
            self.resizing = (1..self.columns.len())
                .find(|&i| (at.x - edges[i]).abs() <= GRIP)
                .map(|i| i - 1);
            if self.resizing.is_none()
                && self.sortable
                && let Some(column) = column
            {
                let ascending = self.sorted != Some((column, true));
                self.sort_by(column, ascending);
            }
            return;
        }
        let row = self.top + line as usize - 1;
        let Some(column) = column else { return };
        if row >= self.rows.len() {
            self.commit(cx);
            return;
        }
        if self.edit.as_ref().is_some_and(|e| e.row == row && e.column == column) {
            return;
        }
        self.commit(cx);
        let again = self.selected == Some(row) && self.column == column;
        self.column = column;
        self.choose(row, cx);
        if again {
            self.start_edit(None);
        }
    }

    fn on_mouse_drag(&mut self, at: Point, cx: &mut EventCx) {
        let Some(left) = self.resizing else { return };
        let width = cx.size().0;
        let edges = self.edges(width);
        // Меняются два соседних столбца, остальные стоят.
        let (from, to) = (edges[left], edges[left + 2]);
        if to - from < MIN_COLUMN * 2.0 {
            return;
        }
        let x = at.x.clamp(from + MIN_COLUMN, to - MIN_COLUMN);
        let total = self.shares[left] + self.shares[left + 1];
        self.shares[left] = total * (x - from) / (to - from);
        self.shares[left + 1] = total - self.shares[left];
    }

    fn on_mouse_up(&mut self, _at: Point, _inside: bool, _cx: &mut EventCx) {
        self.resizing = None;
    }

    fn on_focus_lost(&mut self, cx: &mut EventCx) {
        self.commit(cx);
    }

    fn on_text_input(&mut self, text: &str, _cx: &mut EventCx) {
        match &mut self.edit {
            Some(edit) => edit.text.push_str(text),
            // Пробел без правки — не начало набора.
            None if !text.trim().is_empty() => self.start_edit(Some(text.to_owned())),
            None => {}
        }
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        if let Some(edit) = &mut self.edit {
            match key {
                Key::Backspace => {
                    edit.text.pop();
                }
                Key::Escape => self.edit = None,
                Key::Enter => self.commit(cx),
                Key::Up | Key::Down => self.commit(cx),
                _ => {}
            }
            return;
        }
        let Some(last) = self.rows.len().checked_sub(1) else { return };
        let to = match key {
            Key::Up => self.selected.map_or(0, |i| i.saturating_sub(1)),
            Key::Down => self.selected.map_or(0, |i| (i + 1).min(last)),
            Key::Home => 0,
            Key::End => last,
            Key::Left => {
                self.column = self.column.saturating_sub(1);
                return;
            }
            Key::Right => {
                self.column = (self.column + 1).min(self.columns.len().saturating_sub(1));
                return;
            }
            Key::Enter => {
                self.start_edit(None);
                return;
            }
            _ => return,
        };
        self.choose(to, cx);
    }

    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        self.commit(cx);
        self.wheel += lines;
        let whole = self.wheel.trunc();
        self.wheel -= whole;
        let max = self.rows.len().saturating_sub(self.visible(cx.size().1));
        self.top = (self.top as f32 - whole).clamp(0.0, max as f32) as usize;
    }
}
