//! Модель формы → настоящие контролы RustForms → картинка.

use std::{any::Any, collections::HashMap, path::Path};

use rustforms::prelude::*;
use rustforms_model::{Align, Control as Item, FormModel, Kind};

/// Контролы предпросмотра: их набор известен только во время работы.
type Controls = Vec<Box<dyn Any>>;

#[derive(Default)]
pub struct Renderer {
    /// Живёт всё время: загрузка системных шрифтов дорогая.
    app: Option<Application<Controls>>,
    /// Вкладки последней картинки: имя и номер в списке контролов.
    tabs: Vec<(String, usize)>,
}

impl Renderer {
    /// `folder` — папка формы: от неё считаются пути к картинкам.
    pub fn render(&mut self, model: &FormModel, scale: f32, folder: &Path) -> Option<tiny_skia::Pixmap> {
        let width = (model.form.width * scale).round() as u32;
        let height = (model.form.height * scale).round() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width.min(8192), height.min(8192))?;
        let (ui, controls, view, tabs) = build(model, folder);
        self.tabs = tabs;
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

    /// Левые края ярлычков каждого `TabControl` (и правый край последнего) — чтобы
    /// щелчок по ярлычку в дизайнере переключал страницу.
    pub fn tab_edges(&self) -> Vec<(String, Vec<f32>)> {
        let Some(app) = &self.app else { return Vec::new() };
        self.tabs
            .iter()
            .filter_map(|(name, index)| {
                let tabs = app.state[*index].downcast_ref::<TabControl>()?;
                Some((name.clone(), tabs.tab_edges().to_vec()))
            })
            .collect()
    }
}

fn color(text: &Option<String>) -> Option<Color> {
    let hex = u32::from_str_radix(text.as_deref()?.strip_prefix('#')?, 16).ok()?;
    let [_, r, g, b] = hex.to_be_bytes();
    Some(Color::rgb(r, g, b))
}

/// Контейнер, в который кладутся дети.
#[derive(Clone, Copy)]
enum Parent {
    Panel(Handle<Panel>),
    Group(Handle<GroupBox>),
    Tabs(Handle<TabControl>),
    Page(Handle<TabPage>),
    Split(Handle<SplitContainer>),
    Half(Handle<SplitPanel>),
}

type Built = (Ui, Controls, View<Controls>, Vec<(String, usize)>);

fn build(model: &FormModel, folder: &Path) -> Built {
    let form = &model.form;
    let mut ui = Ui::new(form.title.clone(), form.width, form.height);
    if let Some(back) = color(&form.back_color) {
        ui.back_color = back;
    }
    let mut controls = Controls::new();
    let mut view = View::new();
    let mut panels: HashMap<&str, Parent> = HashMap::new();
    let mut tab_controls = Vec::new();
    for item in &model.controls {
        let parent = item.parent.as_deref().and_then(|p| panels.get(p)).copied();
        let text = item.text.clone().unwrap_or_default();
        let target = (&mut controls, &mut view);
        match item.kind {
            Kind::Button => {
                place(target, parent, item, Button::new(text));
            }
            Kind::Label => {
                place(target, parent, item, Label::new(text));
            }
            Kind::TextBox => {
                let mut input = TextBox::new();
                input.set_multiline(item.multiline);
                input.set_password(item.password);
                input.set_text(text);
                place(target, parent, item, input);
            }
            Kind::CheckBox => {
                let mut check = CheckBox::new(text);
                check.set_checked(item.checked);
                place(target, parent, item, check);
            }
            Kind::RadioButton => {
                let mut radio = RadioButton::new(text);
                radio.set_checked(item.checked);
                place(target, parent, item, radio);
            }
            Kind::ComboBox => {
                let mut combo = ComboBox::new();
                combo.set_items(item.items.iter().cloned());
                combo.set_selected_index(item.selected);
                place(target, parent, item, combo);
            }
            Kind::ListBox => {
                let mut list = ListBox::new();
                list.set_items(item.items.iter().cloned());
                list.set_selected_index(item.selected);
                place(target, parent, item, list);
            }
            Kind::ProgressBar => {
                let mut bar = ProgressBar::new();
                bar.set_range(item.minimum, item.maximum);
                bar.set_value(item.value);
                place(target, parent, item, bar);
            }
            Kind::MenuStrip => {
                let mut menu = MenuStrip::new();
                menu.set_items(item.items.iter().cloned());
                place(target, parent, item, menu);
            }
            Kind::TabControl => {
                let mut tabs = TabControl::new();
                tabs.set_selected_index(item.selected.unwrap_or(0));
                tab_controls.push((item.name.clone(), target.0.len()));
                let tabs = place(target, parent, item, tabs);
                panels.insert(&item.name, Parent::Tabs(tabs));
            }
            Kind::TabPage => {
                let page = place(target, parent, item, TabPage::new(text));
                panels.insert(&item.name, Parent::Page(page));
            }
            Kind::Timer => {
                // В программе таймера не видно; в дизайнере он — значок, чтобы его выбрать.
                let mut icon = PictureBox::new();
                icon.set_image_png(include_bytes!("../icons/Timer.png"));
                icon.set_back_color(Color::rgb(250, 250, 250));
                icon.set_enabled(true);
                let mut shown = item.clone();
                shown.enabled = true;
                place(target, parent, &shown, icon);
            }
            Kind::DateTimePicker => {
                let mut picker = DateTimePicker::new();
                if let Some(date) = item.date.as_deref().and_then(|d| d.parse().ok()) {
                    picker.set_value(date);
                }
                place(target, parent, item, picker);
            }
            Kind::SplitContainer => {
                let mut split = SplitContainer::new();
                split.set_distance(item.distance as f32);
                split.set_horizontal(item.horizontal);
                let split = place(target, parent, item, split);
                panels.insert(&item.name, Parent::Split(split));
            }
            Kind::SplitPanel => {
                let half = place(target, parent, item, SplitPanel::new());
                panels.insert(&item.name, Parent::Half(half));
            }
            Kind::ToolStrip => {
                let mut tools = ToolStrip::new();
                tools.set_items(item.items.iter().cloned());
                place(target, parent, item, tools);
            }
            Kind::GroupBox => {
                let group = place(target, parent, item, GroupBox::new(text));
                panels.insert(&item.name, Parent::Group(group));
            }
            Kind::LinkLabel => {
                place(target, parent, item, LinkLabel::new(text));
            }
            Kind::StatusStrip => {
                place(target, parent, item, StatusStrip::new(text));
            }
            Kind::NumericUpDown => {
                let mut number = NumericUpDown::new();
                number.set_range(item.minimum, item.maximum);
                number.set_value(item.value);
                place(target, parent, item, number);
            }
            Kind::TrackBar => {
                let mut track = TrackBar::new();
                track.set_range(item.minimum, item.maximum);
                track.set_value(item.value);
                place(target, parent, item, track);
            }
            Kind::PictureBox => {
                let mut picture = PictureBox::new();
                if let Some(image) = item.image.as_deref().filter(|i| !i.is_empty()) {
                    // Нет файла — остаётся пустая рамка.
                    let _ = picture.load(folder.join(image));
                }
                place(target, parent, item, picture);
            }
            Kind::DataGridView => {
                let mut grid = DataGridView::new();
                grid.set_columns(item.items.iter().cloned());
                place(target, parent, item, grid);
            }
            Kind::TreeView => {
                let mut tree = TreeView::new();
                tree.set_items(item.items.iter().cloned());
                tree.expand_all();
                place(target, parent, item, tree);
            }
            Kind::Panel => {
                let mut panel = Panel::new();
                panel.border = item.border;
                panels.insert(&item.name, Parent::Panel(place(target, parent, item, panel)));
            }
        }
    }
    (ui, controls, view, tab_controls)
}

/// Скрытые контролы в дизайнере показываются, иначе их не выбрать.
fn place<T: ControlImpl>(
    (controls, view): (&mut Controls, &mut View<Controls>),
    parent: Option<Parent>,
    item: &Item,
    control: T,
) -> Handle<T> {
    // Якоря и Dock уже учтены дизайнером в координатах модели.
    let mut control = control.at(item.x, item.y).size(item.width, item.height);
    control.set_enabled(item.enabled);
    if let Some(fore) = color(&item.fore_color) {
        control.set_fore_color(fore);
    }
    if let Some(back) = color(&item.back_color) {
        control.set_back_color(back);
    }
    if let Some(size) = item.font_size {
        control.set_font_size(size);
    }
    control.set_bold(item.bold);
    if let Some(align) = item.align {
        control.set_text_align(match align {
            Align::Left => TextAlign::Left,
            Align::Center => TextAlign::Center,
            Align::Right => TextAlign::Right,
        });
    }
    let index = controls.len();
    controls.push(Box::new(control));
    match parent {
        Some(Parent::Panel(panel)) => view.add_to(panel, slot::<T>(index)),
        Some(Parent::Group(group)) => view.add_to(group, slot::<T>(index)),
        Some(Parent::Tabs(tabs)) => view.add_to(tabs, slot::<T>(index)),
        Some(Parent::Page(page)) => view.add_to(page, slot::<T>(index)),
        Some(Parent::Split(split)) => view.add_to(split, slot::<T>(index)),
        Some(Parent::Half(half)) => view.add_to(half, slot::<T>(index)),
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
