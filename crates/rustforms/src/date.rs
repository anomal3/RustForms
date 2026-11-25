//! Календарная дата без часов и часовых поясов.

use std::{fmt, str::FromStr, time::SystemTime};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    /// С единицы.
    pub day: u32,
}

impl Date {
    /// Несуществующий день прижимается к границам месяца.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        let year = year.clamp(1, 9999);
        let month = month.clamp(1, 12);
        Self { year, month, day: day.clamp(1, days_in_month(year, month)) }
    }

    /// Сегодняшняя дата **по UTC**: местный часовой пояс стандартная библиотека не знает.
    pub fn today() -> Self {
        let seconds = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Self::from_days((seconds / 86_400) as i64)
    }

    /// Номер дня от 1 января 1970 года.
    pub fn to_days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let year_of_era = y - era * 400;
        let month = i64::from((self.month + 9) % 12);
        let day_of_year = (153 * month + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let mp = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = (year_of_era + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// Тот же день через `months` месяцев (или последний день месяца).
    pub fn add_months(self, months: i32) -> Self {
        let total = self.year * 12 + self.month as i32 - 1 + months;
        Self::new(total.div_euclid(12), total.rem_euclid(12) as u32 + 1, self.day)
    }

    /// 0 — понедельник.
    pub fn weekday(self) -> u32 {
        (self.to_days() + 3).rem_euclid(7) as u32
    }
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `2026-11-25`.
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for Date {
    type Err = &'static str;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut parts = text.split('-').map(|p| p.parse::<u32>().ok());
        let (Some(Some(y)), Some(Some(m)), Some(Some(d)), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err("дата пишется как ГГГГ-ММ-ДД");
        };
        let date = Date::new(y as i32, m, d);
        if (date.year, date.month, date.day) == (y as i32, m, d) {
            Ok(date)
        } else {
            Err("такой даты не бывает")
        }
    }
}
