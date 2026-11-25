//! Базовая форма: заголовок, строка состояния и кнопка «Закрыть».
//! Другие формы подключают её и получают всё это вместе с обработчиками.

use rustforms::prelude::*;

pub struct DialogForm {
    pub heading: Label,
    pub status: Label,
    pub close: Button,
}

impl DialogForm {
    pub fn new(ui: &Ui, heading: &str) -> Self {
        Self {
            heading: Label::new(heading).at(20.0, 16.0).size(320.0, 22.0),
            status: Label::new("").at(20.0, ui.height - 40.0).size(220.0, 22.0),
            close: Button::new("Закрыть").at(ui.width - 110.0, ui.height - 44.0),
        }
    }

    pub fn view(view: &mut View<Self>) {
        view.add(|f: &mut Self| &mut f.heading);
        view.add(|f: &mut Self| &mut f.status);
        let close = view.add(|f: &mut Self| &mut f.close);
        view.on_load(Self::dialog_load);
        view.on_click(close, Self::close_click);
    }

    fn dialog_load(&mut self, _ui: &mut Ui, _e: &LoadArgs) {
        self.status.set_text("Готово к работе");
    }

    fn close_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        ui.close();
    }
}
