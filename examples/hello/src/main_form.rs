use rustforms::prelude::*;

#[path = "main_form.designer.rs"]
mod designer;

pub use designer::MainForm;

/// Свои данные формы: в обработчиках доступны как `self.data`.
#[derive(Default)]
pub struct Data {
    clicks: u32,
}

impl MainForm {
    fn greet_button_click(&mut self, _ui: &mut Ui, _e: &ClickArgs) {
        self.data.clicks += 1;
        let name = match self.name_box.text().trim() {
            "" => "незнакомец",
            name => name,
        };
        let mut text = format!("Привет, {name}! Нажатий: {}", self.data.clicks);
        if self.shout.checked() {
            text = text.to_uppercase();
        }
        self.result.set_text(text);
    }

    fn close_button_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        ui.close();
    }
}
