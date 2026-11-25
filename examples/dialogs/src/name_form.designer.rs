// Создано дизайнером RustForms из .rfrm. Не редактируйте: изменения пропадут.

#![allow(non_snake_case)]

use rustforms::prelude::*;

use super::Data;

/// Форма: её контролы и твои данные (`data`). Обработчики — в соседнем файле.
#[allow(dead_code)]
pub struct NameForm {
    pub data: Data,
    pub name_box: TextBox,
    pub ok_button: Button,
    pub cancel_button: Button,
}

impl NameForm {
    pub fn create() -> Application<Self> {
        let ui = Ui::new("Как вас зовут?", 300.0, 110.0);

        let name_box = TextBox::new().at(16.0, 20.0).size(268.0, 28.0);

        let ok_button = Button::new("ОК").at(96.0, 66.0).size(90.0, 28.0);

        let cancel_button = Button::new("Отмена").at(194.0, 66.0).size(90.0, 28.0);

        let form = Self {
            data: Data::default(),
            name_box,
            ok_button,
            cancel_button,
        };

        let mut view = View::new();
        view.add(|f: &mut Self| &mut f.name_box);
        let ok_button_id = view.add(|f: &mut Self| &mut f.ok_button);
        let cancel_button_id = view.add(|f: &mut Self| &mut f.cancel_button);
        view.on_click(ok_button_id, Self::ok_button_click);
        view.on_click(cancel_button_id, Self::cancel_button_click);

        Application::new(ui, form, view)
    }
}
