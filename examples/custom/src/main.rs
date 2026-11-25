// Свои контролы и производная форма.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dialog_form;
mod round_button;

use dialog_form::DialogForm;
use round_button::RoundButton;
use rustforms::prelude::*;

struct MainForm {
    base: DialogForm,
    plus: RoundButton,
    reset: RoundButton,
    count: u32,
}

impl MainForm {
    fn plus_click(&mut self, _ui: &mut Ui, _e: &ClickArgs) {
        self.count += 1;
        self.base.status.set_text(format!("Счёт: {}", self.count));
        self.plus.set_text(format!("Ещё ({})", self.count));
    }

    fn reset_click(&mut self, _ui: &mut Ui, _e: &ClickArgs) {
        self.count = 0;
        self.base.status.set_text("Сброшено");
        self.plus.set_text("Прибавить");
    }
}

fn build() -> Application<MainForm> {
    let ui = Ui::new("RustForms — свои контролы", 380.0, 180.0);
    let form = MainForm {
        base: DialogForm::new(&ui, "Эта форма построена на DialogForm"),
        plus: RoundButton::new("Прибавить", Color::rgb(0, 120, 90))
            .at(20.0, 56.0)
            .size(160.0, 40.0),
        reset: RoundButton::new("Сбросить", Color::rgb(170, 60, 50))
            .at(200.0, 56.0)
            .size(160.0, 40.0),
        count: 0,
    };

    let mut view = View::new();
    view.include(|form: &mut MainForm| &mut form.base, DialogForm::view);
    let plus = view.add(|f: &mut MainForm| &mut f.plus);
    let reset = view.add(|f: &mut MainForm| &mut f.reset);
    view.on_click(plus, MainForm::plus_click);
    view.on_click(reset, MainForm::reset_click);

    Application::new(ui, form, view)
}

fn main() -> Result<(), rustforms::RunError> {
    run(build())
}
