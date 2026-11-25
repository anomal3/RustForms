use std::time::{Duration, Instant};

/// Таймер формы: вызывает `tick` через равные промежутки, пока запущен.
///
/// Это не контрол — на форме его не видно. Подключается через `view.add_timer`.
#[derive(Clone, Debug)]
pub struct Timer {
    interval: Duration,
    enabled: bool,
    /// Когда сработать в следующий раз; ставит оконный цикл.
    pub(crate) next: Option<Instant>,
}

impl Timer {
    /// Интервал в миллисекундах. Таймер создаётся остановленным.
    pub fn new(interval_ms: u64) -> Self {
        Self { interval: Duration::from_millis(interval_ms.max(1)), enabled: false, next: None }
    }

    pub fn start(&mut self) {
        self.enabled = true;
    }

    pub fn stop(&mut self) {
        self.enabled = false;
        self.next = None;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn interval_ms(&self) -> u64 {
        self.interval.as_millis() as u64
    }

    pub fn set_interval_ms(&mut self, interval_ms: u64) {
        self.interval = Duration::from_millis(interval_ms.max(1));
        self.next = None;
    }

    pub(crate) fn interval(&self) -> Duration {
        self.interval
    }
}
