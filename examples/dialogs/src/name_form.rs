use rustforms::prelude::*;

#[path = "name_form.designer.rs"]
mod designer;

pub use designer::NameForm;

#[derive(Default)]
pub struct Data {}

impl NameForm {
    fn ok_button_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        ui.accept();
    }

    fn cancel_button_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        ui.close();
    }
}
