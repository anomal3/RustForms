//! Модель формы → настоящие контролы RustForms → картинка.

use std::{any::Any, collections::HashMap};

use rustforms::prelude::*;
use rustforms_model::{Control as Item, FormModel, Kind};

/// Контролы предпросмотра: их набор известен только во время работы.
type Controls = Vec<Box<dyn Any>>;

#[derive(Default)]
pub struct Renderer {
    /// Живёт всё время: загрузка системных шрифтов дорогая.
    app: Option<Application<Controls>>,
}

impl Renderer {
    pub fn render(&mut self, model: &FormModel, scale: f32) -> Option<tiny_skia::Pixmap> {
        let width = (model.form.width * scale).round() as u32;
        let height = (model.form.height * scale).round() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width.min(8192), height.min(8192))?;
        let (ui, controls, view) = build(model);
        let app = match &mut self.app {
            Some(app) => {
                app.replace(ui, controls, view);
                app
            }
            None => self.app.insert(Application::new(ui, controls, view)),
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

fn build(model: &FormModel) -> (Ui, Controls, View<Controls>) {
    let form = &model.form;
    let mut ui = Ui::new(form.title.clone(), form.width, form.height);
    if let Some(back) = color(&form.back_color) {
        ui.back_color = back;
    }
    let mut controls = Controls::new();
    let mut view = View::new();
    let mut panels: HashMap<&str, Handle<Panel>> = HashMap::new();
    for item in &model.controls {
        let parent = item.parent.as_deref().and_then(|p| panels.get(p)).copied();
        let text = item.text.clone().unwrap_or_default();
        let target = (&mut controls, &mut view);
        match item.kind {
            Kind::Button => {
                place(target, parent, item, Button::new(text));
            }
            Kind::Label => {
                let mut label = Label::new(text);
                if let Some(fore) = color(&item.fore_color) {
                    label.fore_color = fore;
                }
                place(target, parent, item, label);
            }
            Kind::TextBox => {
                let mut input = TextBox::new();
                input.set_text(text);
                place(target, parent, item, input);
            }
            Kind::CheckBox => {
                let mut check = CheckBox::new(text);
                check.set_checked(item.checked);
                place(target, parent, item, check);
            }
            Kind::Panel => {
                let mut panel = Panel::new();
                if let Some(back) = color(&item.back_color) {
                    panel.back_color = back;
                }
                panel.border = item.border;
                panels.insert(&item.name, place(target, parent, item, panel));
            }
        }
    }
    (ui, controls, view)
}

/// Скрытые контролы в дизайнере показываются, иначе их не выбрать.
fn place<T: ControlImpl>(
    (controls, view): (&mut Controls, &mut View<Controls>),
    parent: Option<Handle<Panel>>,
    item: &Item,
    control: T,
) -> Handle<T> {
    // Якоря и Dock уже учтены дизайнером в координатах модели.
    let mut control = control.at(item.x, item.y).size(item.width, item.height);
    control.set_enabled(item.enabled);
    let index = controls.len();
    controls.push(Box::new(control));
    match parent {
        Some(panel) => view.add_to(panel, slot::<T>(index)),
        None => view.add(slot::<T>(index)),
    }
}

/// Явная сигнатура: из замыкания компилятор не выведет, что ссылка живёт столько же,
/// сколько переданный список.
fn slot<T: 'static>(index: usize) -> impl Fn(&mut Controls) -> &mut T + 'static {
    fn typed<T: 'static>(item: &mut Box<dyn Any>) -> &mut T {
        item.downcast_mut().expect("тип контрола задан в build")
    }
    move |all| typed(&mut all[index])
}
