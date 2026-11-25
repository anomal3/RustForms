//! Базовая форма: заголовок, строка состояния и кнопка «Закрыть».
//! Другие формы подключают её и получают всё это вместе с обработчиками.

use rustforms::prelude::*;

pub struct DialogForm {
    pub status: Handle<Label>,
}

impl DialogForm {
    pub fn build(ui: &mut Ui, events: &mut Events<Self>, heading: &str) -> Self {
        ui.add(Label::new(heading).at(20.0, 16.0).size(320.0, 22.0));
        let status = ui.add(Label::new("").at(20.0, ui.height - 40.0).size(220.0, 22.0));
        let close = ui.add(Button::new("Закрыть").at(ui.width - 110.0, ui.height - 44.0));
        events.on_load(Self::dialog_load);
        events.on_click(close, Self::close_click);
        Self { status }
    }

    fn dialog_load(&mut self, ui: &mut Ui, _e: &LoadArgs) {
        ui[self.status].set_text("Готово к работе");
    }

    fn close_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        ui.close();
    }
}
