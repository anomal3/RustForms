// Создано дизайнером RustForms из .rfrm. Не редактируйте: изменения пропадут.

#![allow(non_snake_case)]

use rustforms::prelude::*;

use super::MainForm;

#[allow(dead_code)]
pub struct Controls {
    pub question: Handle<Label>,
    pub name_box: Handle<TextBox>,
    pub shout: Handle<CheckBox>,
    pub panel1: Handle<Panel>,
    pub result: Handle<Label>,
    pub greet_button: Handle<Button>,
    pub close_button: Handle<Button>,
}

impl Controls {
    pub fn build(events: &mut Events<MainForm>) -> (Ui, Self) {
        let mut ui = Ui::new("RustForms — привет", 420.0, 230.0);

        let control = Label::new("Как тебя зовут?").at(20.0, 20.0).size(200.0, 22.0);
        let question = ui.add(control);

        let control = TextBox::new().at(20.0, 48.0).size(380.0, 30.0);
        let name_box = ui.add(control);

        let control = CheckBox::new("Громко").at(20.0, 90.0).size(140.0, 22.0);
        let shout = ui.add(control);

        let control = Panel::new().at(20.0, 122.0).size(380.0, 44.0);
        let panel1 = ui.add(control);

        let control = Label::new("").at(10.0, 11.0).size(360.0, 22.0);
        let result = ui.add_to(panel1, control);

        let control = Button::new("Поздороваться").at(180.0, 184.0).size(124.0, 30.0);
        let greet_button = ui.add(control);

        let control = Button::new("Закрыть").at(310.0, 184.0).size(90.0, 30.0);
        let close_button = ui.add(control);

        events.on_click(greet_button, MainForm::greet_button_click);
        events.on_click(close_button, MainForm::close_button_click);

        (ui, Self { question, name_box, shout, panel1, result, greet_button, close_button })
    }
}
