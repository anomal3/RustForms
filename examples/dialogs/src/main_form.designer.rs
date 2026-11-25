// Создано дизайнером RustForms из .rfrm. Не редактируйте: изменения пропадут.

#![allow(non_snake_case)]

use rustforms::prelude::*;

use super::Data;

/// Форма: её контролы и твои данные (`data`). Обработчики — в соседнем файле.
#[allow(dead_code)]
pub struct MainForm {
    pub data: Data,
    pub menu: MenuStrip,
    pub result: Label,
    pub ask_button: Button,
}

impl MainForm {
    pub fn create() -> Application<Self> {
        let ui = Ui::new("RustForms — меню и диалоги", 440.0, 260.0);

        let mut menu = MenuStrip::new().at(0.0, 0.0).size(440.0, 26.0);
        menu.set_items(["Файл/Спросить имя…", "Файл/Открыть файл…", "Файл/-", "Файл/Выход", "Справка/О программе"]);
        menu.set_dock(Dock::Top);

        let mut result = Label::new("Выберите пункт меню").at(20.0, 60.0).size(400.0, 22.0);
        result.set_anchor(Anchor::LEFT | Anchor::TOP | Anchor::RIGHT);

        let ask_button = Button::new("Спросить имя").at(20.0, 100.0).size(140.0, 30.0);

        let form = Self {
            data: Data::default(),
            menu,
            result,
            ask_button,
        };

        let mut view = View::new();
        let menu_id = view.add(|f: &mut Self| &mut f.menu);
        view.add(|f: &mut Self| &mut f.result);
        let ask_button_id = view.add(|f: &mut Self| &mut f.ask_button);
        view.on_item_clicked(menu_id, Self::menu_item_clicked);
        view.on_click(ask_button_id, Self::ask_button_click);

        Application::new(ui, form, view)
    }
}
