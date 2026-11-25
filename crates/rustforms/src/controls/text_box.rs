use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Align, Color, ControlBase, Control, ControlImpl, EventCx, Key, Modifiers, Painter, Point, Rect,
};

const PADDING: f32 = 6.0;
const LINE: f32 = 20.0;
const TOP: f32 = 4.0;

/// Поле ввода; по умолчанию однострочное, см. [`TextBox::set_multiline`].
#[derive(Control)]
pub struct TextBox {
    base: ControlBase,
    text: String,
    /// Байтовые индексы; выделение — то, что между ними.
    cursor: usize,
    anchor: usize,
    scroll: f32,
    multiline: bool,
    /// Первая видимая строка.
    top: usize,
    /// Прокрутить к курсору при следующей отрисовке (колесо это отменяет).
    follow: bool,
    wheel: f32,
}

impl TextBox {
    pub fn new() -> Self {
        Self {
            base: ControlBase::new(160.0, 28.0, true),
            text: String::new(),
            cursor: 0,
            anchor: 0,
            scroll: 0.0,
            multiline: false,
            top: 0,
            follow: true,
            wheel: 0.0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn multiline(&self) -> bool {
        self.multiline
    }

    /// Несколько строк: Enter переводит строку. Переносов по словам нет.
    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
        let text = std::mem::take(&mut self.text);
        self.set_text(text);
    }

    fn clean(&self, text: &str) -> String {
        if self.multiline {
            text.replace("\r\n", "\n").chars().filter(|&c| c == '\n' || !c.is_control()).collect()
        } else {
            single_line(text)
        }
    }

    /// Байтовые границы строки, в которой находится позиция.
    fn line_of(&self, index: usize) -> std::ops::Range<usize> {
        let start = self.text[..index].rfind('\n').map_or(0, |i| i + 1);
        let end = self.text[index..].find('\n').map_or(self.text.len(), |i| index + i);
        start..end
    }

    /// Позиция в строке `line`, ближайшая к горизонтальной координате.
    fn index_in(&self, line: std::ops::Range<usize>, x: f32, cx: &mut EventCx) -> usize {
        let offset = cx
            .text_system()
            .caret_positions(&self.text[line.clone()])
            .into_iter()
            .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()))
            .map_or(0, |p| p.0);
        line.start + offset
    }

    fn vertical(&mut self, down: bool, extend: bool, cx: &mut EventCx) {
        let line = self.line_of(self.cursor);
        let x = cx
            .text_system()
            .caret_positions(&self.text[line.clone()])
            .into_iter()
            .find(|p| p.0 == self.cursor - line.start)
            .map_or(0.0, |p| p.1);
        let target = if down {
            (line.end < self.text.len()).then(|| self.line_of(line.end + 1))
        } else {
            (line.start > 0).then(|| self.line_of(line.start - 1))
        };
        if let Some(target) = target {
            let index = self.index_in(target, x, cx);
            self.move_to(index, extend);
        }
    }

    fn paint_lines(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        let width = (area.width - PADDING * 2.0).max(0.0);
        let visible = (((area.height - TOP * 2.0) / LINE).floor() as usize).max(1);
        let lines: Vec<&str> = self.text.split('\n').collect();
        let cursor_line = self.text[..self.cursor].matches('\n').count();
        let cursor_start = self.line_of(self.cursor).start;
        let cursor_x = p
            .text_system()
            .caret_positions(lines[cursor_line])
            .into_iter()
            .find(|q| q.0 == self.cursor - cursor_start)
            .map_or(0.0, |q| q.1);
        if self.follow {
            self.follow = false;
            if cursor_line < self.top {
                self.top = cursor_line;
            } else if cursor_line >= self.top + visible {
                self.top = cursor_line + 1 - visible;
            }
            self.scroll = self.scroll.max(cursor_x - width + 1.0).min(cursor_x).max(0.0);
        }
        self.top = self.top.min(lines.len().saturating_sub(visible));

        let selection = self.selection();
        let ink = if state.enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        let mut start = 0;
        for (number, line) in lines.iter().enumerate() {
            let end = start + line.len();
            if number >= self.top && number < self.top + visible {
                let y = TOP + (number - self.top) as f32 * LINE;
                let inner = Rect::new(PADDING, y, width, LINE);
                if state.focused && selection.start <= end && selection.end > start {
                    let positions = p.text_system().caret_positions(line);
                    let x_of = |index: usize| {
                        positions.iter().find(|q| q.0 == index).map_or(0.0, |q| q.1)
                    };
                    let from = x_of(selection.start.saturating_sub(start));
                    // Выделенный перевод строки показывается небольшим хвостом.
                    let to = if selection.end > end {
                        x_of(line.len()) + 6.0
                    } else {
                        x_of(selection.end - start)
                    };
                    let band = Rect::new(inner.x + from - self.scroll, y, to - from, LINE);
                    p.fill_rect(band.intersect(&inner), Color::SELECTION);
                }
                p.text(line, inner, ink, Align::Left, self.scroll);
                let x = cursor_x - self.scroll;
                if state.focused && number == cursor_line && (0.0..=width).contains(&x) {
                    p.fill_rect(Rect::new(PADDING + x, y + 1.0, 1.0, LINE - 2.0), Color::TEXT);
                }
            }
            start = end + 1;
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = self.clean(&text.into());
        self.top = 0;
        self.follow = true;
        self.cursor = self.text.len();
        self.anchor = self.cursor;
        self.scroll = 0.0;
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    fn selection(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    fn replace_selection(&mut self, with: &str, cx: &mut EventCx) {
        let range = self.selection();
        if range.is_empty() && with.is_empty() {
            return;
        }
        self.follow = true;
        self.cursor = range.start + with.len();
        self.anchor = self.cursor;
        self.text.replace_range(range, with);
        cx.text_changed(self.text.clone());
    }

    fn previous(&self, from: usize) -> usize {
        self.text[..from].grapheme_indices(true).next_back().map_or(0, |(i, _)| i)
    }

    fn next(&self, from: usize) -> usize {
        self.text[from..]
            .graphemes(true)
            .next()
            .map_or(from, |g| from + g.len())
    }

    fn move_to(&mut self, index: usize, extend: bool) {
        self.cursor = index;
        self.follow = true;
        if !extend {
            self.anchor = index;
        }
    }

    fn index_at(&self, at: Point, cx: &mut EventCx) -> usize {
        let line = if self.multiline {
            let number = self.top + ((at.y - TOP) / LINE).floor().max(0.0) as usize;
            let mut start = 0;
            for _ in 0..number {
                match self.text[start..].find('\n') {
                    Some(i) => start += i + 1,
                    None => break,
                }
            }
            self.line_of(start)
        } else {
            0..self.text.len()
        };
        self.index_in(line, at.x - PADDING + self.scroll, cx)
    }
}

impl Default for TextBox {
    fn default() -> Self {
        Self::new()
    }
}

fn single_line(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

impl ControlImpl for TextBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        let face = if state.enabled { Color::WHITE } else { Color::FORM };
        p.fill_rect(area, face);
        p.stroke_rect(area, Color::BORDER, 1.0);
        if state.focused {
            p.fill_rect(Rect::new(0.0, area.height - 2.0, area.width, 2.0), Color::ACCENT);
        }
        if self.multiline {
            self.paint_lines(p);
            return;
        }

        let inner = Rect::new(PADDING, 2.0, (area.width - PADDING * 2.0).max(0.0), area.height - 4.0);
        let positions = p.text_system().caret_positions(&self.text);
        let x_of = |index: usize| {
            positions.iter().find(|q| q.0 == index).map_or(0.0, |q| q.1)
        };
        // Прокрутка так, чтобы курсор оставался виден.
        let cursor_x = x_of(self.cursor);
        let end_x = positions.last().map_or(0.0, |q| q.1);
        self.scroll = self
            .scroll
            .min((end_x - inner.width + 1.0).max(0.0))
            .max(cursor_x - inner.width + 1.0)
            .min(cursor_x);

        let selection = self.selection();
        if state.focused && !selection.is_empty() {
            let from = x_of(selection.start) - self.scroll;
            let to = x_of(selection.end) - self.scroll;
            let band = Rect::new(inner.x + from, 4.0, to - from, area.height - 8.0);
            p.fill_rect(band.intersect(&inner), Color::SELECTION);
        }
        let ink = if state.enabled { Color::TEXT } else { Color::TEXT_DISABLED };
        p.text(&self.text, inner, ink, Align::Left, self.scroll);
        if state.focused {
            let caret = Rect::new(inner.x + cursor_x - self.scroll, 5.0, 1.0, area.height - 10.0);
            p.fill_rect(caret, Color::TEXT);
        }
    }

    fn on_mouse_down(&mut self, at: Point, mods: Modifiers, cx: &mut EventCx) {
        let index = self.index_at(at, cx);
        self.move_to(index, mods.shift);
    }

    fn on_mouse_drag(&mut self, at: Point, cx: &mut EventCx) {
        let index = self.index_at(at, cx);
        self.move_to(index, true);
    }

    fn on_wheel(&mut self, lines: f32, _cx: &mut EventCx) {
        if self.multiline {
            self.wheel += lines;
            let whole = self.wheel.trunc();
            self.wheel -= whole;
            // Верхнюю границу проверит отрисовка.
            self.top = (self.top as f32 - whole).max(0.0) as usize;
        }
    }

    fn on_text_input(&mut self, text: &str, cx: &mut EventCx) {
        let text = self.clean(text);
        if !text.is_empty() {
            self.replace_selection(&text, cx);
        }
    }

    fn on_key(&mut self, key: Key, mods: Modifiers, cx: &mut EventCx) {
        let selection = self.selection();
        match key {
            Key::Left if !mods.shift && !selection.is_empty() => {
                self.move_to(selection.start, false)
            }
            Key::Right if !mods.shift && !selection.is_empty() => {
                self.move_to(selection.end, false)
            }
            Key::Left => self.move_to(self.previous(self.cursor), mods.shift),
            Key::Right => self.move_to(self.next(self.cursor), mods.shift),
            Key::Home => self.move_to(self.line_of(self.cursor).start, mods.shift),
            Key::End => self.move_to(self.line_of(self.cursor).end, mods.shift),
            Key::Up if self.multiline => self.vertical(false, mods.shift, cx),
            Key::Down if self.multiline => self.vertical(true, mods.shift, cx),
            Key::Enter if self.multiline => self.replace_selection("\n", cx),
            Key::SelectAll => {
                self.anchor = 0;
                self.cursor = self.text.len();
            }
            Key::Backspace => {
                if selection.is_empty() {
                    self.anchor = self.previous(self.cursor);
                }
                self.replace_selection("", cx);
            }
            Key::Delete => {
                if selection.is_empty() {
                    self.anchor = self.next(self.cursor);
                }
                self.replace_selection("", cx);
            }
            Key::Copy | Key::Cut if !selection.is_empty() => {
                cx.clipboard_set(&self.text[selection]);
                if key == Key::Cut {
                    self.replace_selection("", cx);
                }
            }
            Key::Paste => {
                if let Some(text) = cx.clipboard_get() {
                    self.on_text_input(&text, cx);
                }
            }
            _ => {}
        }
    }
}
