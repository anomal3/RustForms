// Свои контролы и производная форма.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dialog_form;
mod round_button;

use dialog_form::DialogForm;
use round_button::RoundButton;
use rustforms::prelude::*;

struct MainForm {
    base: DialogForm,
    plus: Handle<RoundButton>,
    count: u32,
}

impl MainForm {
    fn plus_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        self.count += 1;
        ui[self.base.status].set_text(format!("Счёт: {}", self.count));
        ui[self.plus].set_text(format!("Ещё ({})", self.count));
    }

    fn reset_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        self.count = 0;
        ui[self.base.status].set_text("Сброшено");
        ui[self.plus].set_text("Прибавить");
    }
}

fn build() -> Application<MainForm> {
    let mut ui = Ui::new("RustForms — свои контролы", 380.0, 180.0);
    let mut events = Events::new();
    let base = events.include(
        |form: &mut MainForm| &mut form.base,
        |events| DialogForm::build(&mut ui, events, "Эта форма построена на DialogForm"),
    );

    let plus = ui.add(
        RoundButton::new("Прибавить", Color::rgb(0, 120, 90))
            .at(20.0, 56.0)
            .size(160.0, 40.0),
    );
    let reset = ui.add(
        RoundButton::new("Сбросить", Color::rgb(170, 60, 50))
            .at(200.0, 56.0)
            .size(160.0, 40.0),
    );
    events.on_click(plus, MainForm::plus_click);
    events.on_click(reset, MainForm::reset_click);

    Application::new(ui, MainForm { base, plus, count: 0 }, events)
}

fn main() -> Result<(), rustforms::RunError> {
    run(build())
}
