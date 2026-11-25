use crate::{
    Align, Color, Control, ControlBase, ControlImpl, Date, EventCx, Key, Modifiers, Painter, Point,
    Rect,
};

const CELL: f32 = 30.0;
const ROW: f32 = 24.0;
const WIDTH: f32 = CELL * 7.0 + 12.0;
const HEIGHT: f32 = 30.0 + ROW * 7.0 + 8.0;
const MONTHS: [&str; 12] = [
    "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь", "Июль", "Август", "Сентябрь",
    "Октябрь", "Ноябрь", "Декабрь",
];
const WEEK: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

/// Выбор даты с выпадающим календарём. Времени не хранит.
#[derive(Control)]
pub struct DateTimePicker {
    base: ControlBase,
    value: Date,
    /// Открытый календарь показывает месяц этой даты.
    shown: Option<Date>,
}

impl DateTimePicker {
    pub fn new() -> Self {
        Self { base: ControlBase::new(150.0, 28.0, true), value: Date::today(), shown: None }
    }

    pub fn value(&self) -> Date {
        self.value
    }

    pub fn set_value(&mut self, value: Date) {
        self.value = value;
    }

    pub fn is_open(&self) -> bool {
        self.shown.is_some()
    }

    fn change(&mut self, value: Date, cx: &mut EventCx) {
        if value != self.value {
            self.value = value;
            cx.selected_changed(None, value.to_string());
        }
    }

    /// Первая клетка сетки — понедельник на неделе первого числа.
    fn first_cell(month: Date) -> Date {
        let first = Date::new(month.year, month.month, 1);
        first.add_days(-i64::from(first.weekday()))
    }
}

impl Default for DateTimePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for DateTimePicker {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let state = p.state();
        let area = p.area();
        let back = p.back_color(if state.enabled { Color::WHITE } else { Color::FORM });
        p.fill_rect(area, back);
        let ink = p.fore_color(Color::TEXT);
        let d = self.value;
        let text = format!("{:02}.{:02}.{:04}", d.day, d.month, d.year);
        p.text(&text, Rect::new(8.0, 0.0, area.width - 30.0, area.height), ink, Align::Left, 0.0);
        let (x, y) = (area.width - 16.0, area.height / 2.0 - 2.0);
        for i in 0..4 {
            let i = i as f32;
            p.fill_rect(Rect::new(x - 4.0 + i, y + i, 8.0 - i * 2.0, 1.0), ink);
        }
        p.stroke_rect(area, if state.focused { Color::ACCENT } else { Color::BORDER }, 1.0);
    }

    fn on_mouse_down(&mut self, _at: Point, _mods: Modifiers, _cx: &mut EventCx) {
        self.shown = match self.shown {
            Some(_) => None,
            None => Some(self.value),
        };
    }

    fn on_key(&mut self, key: Key, _mods: Modifiers, cx: &mut EventCx) {
        let days = match key {
            Key::Left => -1,
            Key::Right => 1,
            Key::Up => -7,
            Key::Down => 7,
            Key::Enter | Key::Space => {
                self.shown = match self.shown {
                    Some(_) => None,
                    None => Some(self.value),
                };
                return;
            }
            Key::Escape => {
                self.shown = None;
                return;
            }
            _ => return,
        };
        let value = self.value.add_days(days);
        self.change(value, cx);
        if self.shown.is_some() {
            self.shown = Some(value);
        }
    }

    fn popup_size(&self) -> Option<(f32, f32)> {
        self.shown.map(|_| (WIDTH, HEIGHT))
    }

    fn close_popup(&mut self) {
        self.shown = None;
    }

    fn on_paint_popup(&mut self, p: &mut Painter) {
        let Some(month) = self.shown else { return };
        let area = p.area();
        p.fill_rect(area, Color::WHITE);
        let title = format!("{} {}", MONTHS[month.month as usize - 1], month.year);
        p.text(&title, Rect::new(30.0, 2.0, WIDTH - 60.0, 26.0), Color::TEXT, Align::Center, 0.0);
        p.text("<", Rect::new(4.0, 2.0, 26.0, 26.0), Color::ACCENT, Align::Center, 0.0);
        p.text(">", Rect::new(WIDTH - 30.0, 2.0, 26.0, 26.0), Color::ACCENT, Align::Center, 0.0);
        for (i, day) in WEEK.iter().enumerate() {
            let cell = Rect::new(6.0 + i as f32 * CELL, 30.0, CELL, ROW);
            p.text(day, cell, Color::TEXT_DISABLED, Align::Center, 0.0);
        }
        let today = Date::today();
        let mut date = Self::first_cell(month);
        for n in 0..42 {
            let cell = Rect::new(
                6.0 + (n % 7) as f32 * CELL,
                30.0 + ROW + (n / 7) as f32 * ROW,
                CELL,
                ROW,
            );
            if date == self.value {
                p.fill_round_rect(cell.inset(1.0), 4.0, Color::SELECTION);
            }
            if date == today {
                p.stroke_rect(cell.inset(1.0), Color::ACCENT, 1.0);
            }
            let ink = if date.month == month.month { Color::TEXT } else { Color::BORDER };
            p.text(&date.day.to_string(), cell, ink, Align::Center, 0.0);
            date = date.add_days(1);
        }
        p.stroke_rect(area, Color::BORDER, 1.0);
    }

    fn on_popup_mouse_down(&mut self, at: Point, cx: &mut EventCx) {
        let Some(month) = self.shown else { return };
        if at.y < 30.0 {
            if at.x < 34.0 {
                self.shown = Some(month.add_months(-1));
            } else if at.x > WIDTH - 34.0 {
                self.shown = Some(month.add_months(1));
            }
            return;
        }
        let column = ((at.x - 6.0) / CELL).floor();
        let row = ((at.y - 30.0 - ROW) / ROW).floor();
        if (0.0..7.0).contains(&column) && (0.0..6.0).contains(&row) {
            let date = Self::first_cell(month).add_days((row * 7.0 + column) as i64);
            self.shown = None;
            self.change(date, cx);
        }
    }
}
