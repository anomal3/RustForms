use rustforms::prelude::*;

#[path = "main_form.designer.rs"]
mod designer;

use designer::Controls;

pub struct MainForm {
    controls: Controls,
    clicks: u32,
}

impl MainForm {
    pub fn create() -> Application<Self> {
        let mut events = Events::new();
        let (ui, controls) = Controls::build(&mut events);
        Application::new(ui, Self { controls, clicks: 0 }, events)
    }

    fn greet_button_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        self.clicks += 1;
        let name = match ui[self.controls.name_box].text().trim() {
            "" => "незнакомец",
            name => name,
        };
        let mut text = format!("Привет, {name}! Нажатий: {}", self.clicks);
        if ui[self.controls.shout].checked() {
            text = text.to_uppercase();
        }
        ui[self.controls.result].set_text(text);
    }

    fn close_button_click(&mut self, ui: &mut Ui, _e: &ClickArgs) {
        ui.close();
    }
}
