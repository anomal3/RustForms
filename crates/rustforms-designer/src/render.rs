//! Модель формы → настоящие контролы RustForms → картинка.

use std::collections::HashMap;

use rustforms::prelude::*;
use rustforms_model::{Control as Item, FormModel, Kind};

#[derive(Default)]
pub struct Renderer {
    /// Живёт всё время: загрузка системных шрифтов дорогая.
    app: Option<Application<()>>,
}

impl Renderer {
    pub fn render(&mut self, model: &FormModel, scale: f32) -> Option<tiny_skia::Pixmap> {
        let width = (model.form.width * scale).round() as u32;
        let height = (model.form.height * scale).round() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width.min(8192), height.min(8192))?;
        let ui = build(model);
        let app = match &mut self.app {
            Some(app) => {
                app.ui = ui;
                app
            }
            None => self.app.insert(Application::new(ui, (), Events::new())),
        };
        app.render(&mut pixmap, scale);
        Some(pixmap)
    }
}

fn color(text: &Option<String>) -> Option<Color> {
    let hex = u32::from_str_radix(text.as_deref()?.strip_prefix('#')?, 16).ok()?;
    let [_, r, g, b] = hex.to_be_bytes();
    Some(Color::rgb(r, g, b))
}

pub fn build(model: &FormModel) -> Ui {
    let form = &model.form;
    let mut ui = Ui::new(form.title.clone(), form.width, form.height);
    if let Some(back) = color(&form.back_color) {
        ui.back_color = back;
    }
    let mut panels: HashMap<&str, Handle<Panel>> = HashMap::new();
    for item in &model.controls {
        let parent = item.parent.as_deref().and_then(|p| panels.get(p)).copied();
        let text = item.text.clone().unwrap_or_default();
        match item.kind {
            Kind::Button => {
                place(&mut ui, parent, item, Button::new(text));
            }
            Kind::Label => {
                let mut label = Label::new(text);
                if let Some(fore) = color(&item.fore_color) {
                    label.fore_color = fore;
                }
                place(&mut ui, parent, item, label);
            }
            Kind::TextBox => {
                let mut input = TextBox::new();
                input.set_text(text);
                place(&mut ui, parent, item, input);
            }
            Kind::CheckBox => {
                let mut check = CheckBox::new(text);
                check.set_checked(item.checked);
                place(&mut ui, parent, item, check);
            }
            Kind::Panel => {
                let mut panel = Panel::new();
                if let Some(back) = color(&item.back_color) {
                    panel.back_color = back;
                }
                panel.border = item.border;
                panels.insert(&item.name, place(&mut ui, parent, item, panel));
            }
        }
    }
    ui
}

/// Скрытые контролы в дизайнере показываются, иначе их не выбрать.
fn place<T: ControlImpl>(
    ui: &mut Ui,
    parent: Option<Handle<Panel>>,
    item: &Item,
    control: T,
) -> Handle<T> {
    let mut control = control.at(item.x, item.y).size(item.width, item.height);
    control.set_enabled(item.enabled);
    match parent {
        Some(panel) => ui.add_to(panel, control),
        None => ui.add(control),
    }
}
