use rustforms::prelude::*;

use crate::name_form::NameForm;

#[path = "main_form.designer.rs"]
mod designer;

pub use designer::MainForm;

#[derive(Default)]
pub struct Data {}

impl MainForm {
    fn menu_item_clicked(&mut self, ui: &mut Ui, e: &MenuItemArgs) {
        match e.item.as_str() {
            "Файл/Спросить имя…" => self.ask(ui),
            "Файл/Открыть файл…" => {
                if let Some(path) = ui.open_file(&[("Текст", &["txt", "md"])]) {
                    self.result.set_text(path.display().to_string());
                }
            }
            "Файл/Выход" => {
                if ui.confirm("Выход", "Закрыть программу?") {
                    ui.close();
                }
            }
            "Справка/О программе" => ui.message_box("О программе", "Пример RustForms"),
            _ => {}
        }
    }

    fn ask_button_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        self.ask(ui);
    }

    /// Открывает вторую форму диалогом и забирает из неё введённое.
    fn ask(&mut self, ui: &mut Ui) {
        ui.show_dialog(NameForm::create(), |me: &mut MainForm, dialog: &mut NameForm, ok, _ui| {
            if ok {
                me.result.set_text(format!("Привет, {}!", dialog.name_box.text()));
            } else {
                me.result.set_text("Отменено");
            }
        });
    }
}
