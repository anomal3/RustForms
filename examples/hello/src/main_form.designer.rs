// Создано дизайнером RustForms из .rfrm. Не редактируйте: изменения пропадут.

#![allow(non_snake_case)]

use rustforms::prelude::*;

use super::Data;

/// Форма: её контролы и твои данные (`data`). Обработчики — в соседнем файле.
#[allow(dead_code)]
pub struct MainForm {
    pub data: Data,
    pub question: Label,
    pub name_box: TextBox,
    pub shout: CheckBox,
    pub panel1: Panel,
    pub result: Label,
    pub greet_button: Button,
    pub close_button: Button,
}

impl MainForm {
    pub fn create() -> Application<Self> {
        let ui = Ui::new("RustForms — привет", 420.0, 230.0);

        let question = Label::new("Как тебя зовут?").at(20.0, 20.0).size(200.0, 22.0);

        let name_box = TextBox::new().at(20.0, 48.0).size(380.0, 30.0);

        let shout = CheckBox::new("Громко").at(20.0, 90.0).size(140.0, 22.0);

        let panel1 = Panel::new().at(20.0, 122.0).size(380.0, 44.0);

        let result = Label::new("").at(10.0, 11.0).size(360.0, 22.0);

        let greet_button = Button::new("Поздороваться").at(180.0, 184.0).size(124.0, 30.0);

        let close_button = Button::new("Закрыть").at(310.0, 184.0).size(90.0, 30.0);

        let form = Self {
            data: Data::default(),
            question,
            name_box,
            shout,
            panel1,
            result,
            greet_button,
            close_button,
        };

        let mut view = View::new();
        view.add(|f: &mut Self| &mut f.question);
        view.add(|f: &mut Self| &mut f.name_box);
        view.add(|f: &mut Self| &mut f.shout);
        let panel1_id = view.add(|f: &mut Self| &mut f.panel1);
        view.add_to(panel1_id, |f: &mut Self| &mut f.result);
        let greet_button_id = view.add(|f: &mut Self| &mut f.greet_button);
        let close_button_id = view.add(|f: &mut Self| &mut f.close_button);
        view.on_click(greet_button_id, Self::greet_button_click);
        view.on_click(close_button_id, Self::close_button_click);

        Application::new(ui, form, view)
    }
}
