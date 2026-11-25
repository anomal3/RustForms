//! Правая панель: свойства и события выбранного контрола или самой формы.

use eframe::egui::{self, DragValue, Grid, TextEdit};
use rustforms_model::{Align, Dock, Event, Kind};

use crate::{
    app::{Designer, Request},
    document::Document,
    pickers,
};

/// Поле, значение которого применяется после ввода целиком (имя, путь): пока текст
/// набирается, он хранится отдельно. Возвращает новое значение.
fn committed(ui: &mut egui::Ui, id: &str, current: &str) -> Option<String> {
    let id = ui.make_persistent_id(id);
    let mut text = ui
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.to_owned());
    let response = ui.add(TextEdit::singleline(&mut text).desired_width(f32::INFINITY));
    if response.has_focus() || response.changed() {
        ui.data_mut(|d| d.insert_temp(id, text.clone()));
    }
    if response.lost_focus() {
        ui.data_mut(|d| d.remove_temp::<String>(id));
        return (text != current).then_some(text);
    }
    None
}

fn color_row(ui: &mut egui::Ui, label: &str, value: &mut Option<String>) {
    ui.label(label);
    ui.horizontal(|ui| {
        let mut own = value.is_some();
        if ui.checkbox(&mut own, "").changed() {
            *value = own.then(|| "#ffffff".to_owned());
        }
        if let Some(text) = value {
            let hex = u32::from_str_radix(text.trim_start_matches('#'), 16).unwrap_or(0);
            let [_, r, g, b] = hex.to_be_bytes();
            let mut rgb = [r, g, b];
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                *text = format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);
            }
        } else {
            ui.weak("по умолчанию");
        }
    });
    ui.end_row();
}

fn number(ui: &mut egui::Ui, label: &str, value: &mut f32, min: f32) {
    ui.label(label);
    ui.add(DragValue::new(value).range(min..=4000.0).speed(1.0).max_decimals(0));
    ui.end_row();
}

impl Designer {
    pub(crate) fn properties(&mut self, ui: &mut egui::Ui) {
        let Some(open) = &mut self.open else { return };
        let document = &mut open.document;
        let selected = document.selected.clone();
        let result = match selected.as_deref() {
            Some(name) => control_properties(ui, document, name, &mut self.request),
            None => form_properties(ui, document, &mut self.request),
        };
        if let Err(message) = result {
            self.status = message;
        }
    }
}

fn event_rows(
    ui: &mut egui::Ui,
    control: Option<&str>,
    event: Event,
    current: Option<&str>,
    request: &mut Option<Request>,
) -> Option<String> {
    ui.label(event.name());
    let mut changed = None;
    ui.horizontal(|ui| {
        let label = if current.is_some() { "К коду" } else { "Создать" };
        if ui.button(label).clicked() {
            *request = Some(Request::Handler {
                control: control.map(str::to_owned),
                event: Some(event),
            });
        }
        let id = format!("event {control:?} {}", event.name());
        changed = committed(ui, &id, current.unwrap_or(""));
    });
    ui.end_row();
    changed
}

fn form_properties(
    ui: &mut egui::Ui,
    document: &mut Document,
    request: &mut Option<Request>,
) -> Result<(), String> {
    ui.heading("Форма");
    let mut form = document.model.form.clone();
    let mut rename = None;
    let mut load = None;
    Grid::new("form").num_columns(2).show(ui, |ui| {
        ui.label("Структура");
        rename = committed(ui, "form name", &form.name);
        ui.end_row();
        ui.label("Заголовок");
        ui.add(TextEdit::singleline(&mut form.title).desired_width(f32::INFINITY));
        ui.end_row();
        number(ui, "Ширина", &mut form.width, 80.0);
        number(ui, "Высота", &mut form.height, 40.0);
        color_row(ui, "Фон", &mut form.back_color);
    });
    ui.separator();
    ui.strong("События");
    let mut closing = None;
    let mut resize = None;
    Grid::new("form events").num_columns(2).show(ui, |ui| {
        load = event_rows(ui, None, Event::Load, form.load.as_deref(), request);
        closing = event_rows(ui, None, Event::Closing, form.closing.as_deref(), request);
        resize = event_rows(ui, None, Event::Resize, form.resize.as_deref(), request);
    });

    if form != document.model.form {
        document.checkpoint(Some("form"));
        document.model.form = form;
    }
    if let Some(name) = rename {
        ui.label("Имя структуры поменяйте и в своём .rs");
        document.edit(|m| m.form.name = name)?;
    }
    if let Some(handler) = load {
        document.edit(|m| m.form.load = (!handler.is_empty()).then_some(handler))?;
    }
    if let Some(handler) = closing {
        document.edit(|m| m.form.closing = (!handler.is_empty()).then_some(handler))?;
    }
    if let Some(handler) = resize {
        document.edit(|m| m.form.resize = (!handler.is_empty()).then_some(handler))?;
    }
    Ok(())
}

fn control_properties(
    ui: &mut egui::Ui,
    document: &mut Document,
    name: &str,
    request: &mut Option<Request>,
) -> Result<(), String> {
    let Some(original) = document.control(name).cloned() else { return Ok(()) };
    let mut c = original.clone();
    ui.heading(c.kind.name());
    let mut rename = None;
    let mut custom = None;
    let mut date = None;
    Grid::new(("control", name)).num_columns(2).show(ui, |ui| {
        ui.label("Имя");
        rename = committed(ui, &format!("name {name}"), &c.name);
        ui.end_row();
        if c.kind.has_text() {
            ui.label("Текст");
            let mut text = c.text.clone().unwrap_or_default();
            let edit = if c.multiline {
                TextEdit::multiline(&mut text).desired_rows(3)
            } else {
                TextEdit::singleline(&mut text)
            };
            if ui.add(edit.desired_width(f32::INFINITY)).changed() {
                c.text = Some(text);
            }
            ui.end_row();
        }
        number(ui, "X", &mut c.x, -4000.0);
        number(ui, "Y", &mut c.y, -4000.0);
        number(ui, "Ширина", &mut c.width, 1.0);
        number(ui, "Высота", &mut c.height, 1.0);
        ui.label("Видимый");
        ui.checkbox(&mut c.visible, "");
        ui.end_row();
        ui.label("Доступен");
        ui.checkbox(&mut c.enabled, "");
        ui.end_row();
        match c.kind {
            Kind::CheckBox | Kind::RadioButton => {
                ui.label("Отмечен");
                ui.checkbox(&mut c.checked, "");
                ui.end_row();
            }
            Kind::TextBox => {
                ui.label("Многострочный");
                if ui.checkbox(&mut c.multiline, "").changed()
                    && !c.multiline
                    && let Some(text) = &mut c.text
                {
                    *text = text.replace('\n', " ");
                }
                ui.end_row();
            }
            Kind::ComboBox
            | Kind::ListBox
            | Kind::MenuStrip
            | Kind::DataGridView
            | Kind::TreeView
            | Kind::ToolStrip => {
                let hint = match c.kind {
                    Kind::MenuStrip => "Пункты вида Меню/Пункт; Меню/- это разделитель",
                    Kind::DataGridView => "Заголовки столбцов, каждый с новой строки",
                    Kind::ToolStrip => "Кнопки, каждая с новой строки; - это разделитель",
                    Kind::TreeView => "Узлы вида Корень/Ветка/Лист",
                    _ => "Каждая строка списка с новой строки",
                };
                ui.label("Строки").on_hover_text(hint);
                // Пустая последняя строка нужна, пока набирается следующая.
                let mut text = c.items.join("\n");
                let id = ui.make_persistent_id(("items", name));
                if let Some(draft) = ui.data_mut(|d| d.get_temp::<String>(id)) {
                    text = draft;
                }
                let edit = TextEdit::multiline(&mut text).desired_rows(4);
                let response = ui.add(edit.desired_width(f32::INFINITY));
                if response.changed() {
                    c.items = text.lines().filter(|l| !l.is_empty()).map(str::to_owned).collect();
                    c.selected = c.selected.filter(|&i| i < c.items.len());
                    ui.data_mut(|d| d.insert_temp(id, text));
                }
                if response.lost_focus() {
                    ui.data_mut(|d| d.remove_temp::<String>(id));
                }
                ui.end_row();
                if c.kind == Kind::DataGridView {
                    ui.label("Правка ячеек");
                    ui.checkbox(&mut c.editable, "");
                    ui.end_row();
                }
                if c.kind.selectable() {
                    ui.label("Выбрана");
                    egui::ComboBox::from_id_salt(("selected", name))
                        .selected_text(c.selected.and_then(|i| c.items.get(i)).map_or("—", |s| s))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut c.selected, None, "—");
                            for (i, item) in c.items.iter().enumerate() {
                                ui.selectable_value(&mut c.selected, Some(i), item);
                            }
                        });
                    ui.end_row();
                }
            }
            Kind::PictureBox => {
                ui.label("Файл PNG").on_hover_text("Путь от папки формы, например logo.png");
                let mut path = c.image.clone().unwrap_or_default();
                if ui.add(TextEdit::singleline(&mut path).desired_width(f32::INFINITY)).changed() {
                    c.image = (!path.is_empty()).then_some(path);
                }
                ui.end_row();
            }
            Kind::ProgressBar | Kind::NumericUpDown | Kind::TrackBar => {
                for (label, value) in [
                    ("Минимум", &mut c.minimum),
                    ("Максимум", &mut c.maximum),
                    ("Значение", &mut c.value),
                ] {
                    ui.label(label);
                    ui.add(DragValue::new(value));
                    ui.end_row();
                }
                c.maximum = c.maximum.max(c.minimum);
            }
            Kind::Label => {}
            Kind::Panel => {
                ui.label("Рамка");
                ui.checkbox(&mut c.border, "");
                ui.end_row();
            }
            Kind::Button
            | Kind::GroupBox
            | Kind::LinkLabel
            | Kind::StatusStrip
            | Kind::TabPage
            | Kind::SplitPanel => {}
            Kind::DateTimePicker => {
                ui.label("Дата").on_hover_text("ГГГГ-ММ-ДД; пусто — сегодняшняя");
                date = committed(ui, &format!("date {name}"), c.date.as_deref().unwrap_or(""));
                ui.end_row();
            }
            Kind::SplitContainer => {
                ui.label("Разделитель");
                ui.add(DragValue::new(&mut c.distance).range(20..=4000));
                ui.end_row();
                ui.label("Горизонтальный");
                ui.checkbox(&mut c.horizontal, "");
                ui.end_row();
            }
            Kind::TabControl => {
                ui.label("Страница");
                let mut page = c.selected.unwrap_or(0);
                if ui.add(DragValue::new(&mut page).range(0..=99)).changed() {
                    c.selected = Some(page);
                }
                ui.end_row();
            }
            Kind::Timer => {
                ui.label("Интервал, мс");
                ui.add(DragValue::new(&mut c.interval).range(1..=3_600_000));
                ui.end_row();
            }
        }
        if c.kind != Kind::Timer {
        color_row(ui, "Цвет текста", &mut c.fore_color);
        color_row(ui, "Фон", &mut c.back_color);
        ui.label("Шрифт");
        ui.horizontal(|ui| {
            let mut size = c.font_size.unwrap_or(14.0);
            if ui.add(DragValue::new(&mut size).range(4.0..=200.0).max_decimals(0)).changed() {
                c.font_size = (size != 14.0).then_some(size);
            }
            ui.checkbox(&mut c.bold, "жирный");
        });
        ui.end_row();
        ui.label("Подсказка");
        let mut tooltip = c.tooltip.clone().unwrap_or_default();
        if ui.add(TextEdit::singleline(&mut tooltip).desired_width(f32::INFINITY)).changed() {
            c.tooltip = (!tooltip.is_empty()).then_some(tooltip);
        }
        ui.end_row();
        if c.kind.focusable() {
            ui.label("Порядок Tab");
            ui.add(DragValue::new(&mut c.tab_index));
            ui.end_row();
        }
        if matches!(c.kind, Kind::Button | Kind::Label | Kind::LinkLabel | Kind::StatusStrip) {
            ui.label("Выравнивание");
            egui::ComboBox::from_id_salt(("align", name))
                .selected_text(c.align.map_or("по умолчанию", Align::name))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut c.align, None, "по умолчанию");
                    for align in Align::ALL {
                        ui.selectable_value(&mut c.align, Some(align), align.name());
                    }
                });
            ui.end_row();
        }
        if c.kind == Kind::TextBox {
            ui.label("Только чтение");
            ui.checkbox(&mut c.read_only, "");
            ui.end_row();
            ui.label("Пароль");
            ui.checkbox(&mut c.password, "");
            ui.end_row();
        }
        ui.label("Dock");
        pickers::dock(ui, &mut c.dock);
        ui.end_row();
        ui.label("Anchor");
        if let Some(side) = pickers::anchor(ui, |side| c.anchor.contains(&side)) {
            if !c.anchor.remove(&side) {
                c.anchor.insert(side);
            }
            // Как в WinForms: якоря и Dock исключают друг друга.
            c.dock = Dock::None;
        }
        ui.end_row();
        }
        if c.kind != Kind::Timer {
            ui.label("Меню (правая кнопка)").on_hover_text("Пункты через ;");
            let current = c.context.join("; ");
            if let Some(text) = committed(ui, &format!("context {name}"), &current) {
                c.context =
                    text.split(';').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect();
            }
            ui.end_row();
        }
        ui.label("Свой тип").on_hover_text(
            "Путь к своему контролу на базе этого, например crate::round_button::RoundButton. \
             Тип должен реализовывать Default.",
        );
        custom = committed(ui, &format!("custom {name}"), c.custom.as_deref().unwrap_or(""));
        ui.end_row();
    });

    ui.horizontal(|ui| {
        if ui.button("На передний план").clicked() {
            document.bring_to_front(name);
        }
        if ui.button("Назад").clicked() {
            document.send_to_back(name);
        }
    });

    let mut handlers = Vec::new();
    if !c.kind.events().is_empty() {
        ui.separator();
        ui.strong("События");
        Grid::new(("events", name)).num_columns(2).show(ui, |ui| {
            for event in c.kind.events() {
                let current = c.events.get(&event).map(String::as_str);
                if let Some(value) = event_rows(ui, Some(name), event, current, request) {
                    handlers.push((event, value));
                }
            }
        });
    }

    if c != original {
        document.checkpoint(Some(name));
        if let Some(target) = document.control_mut(name) {
            *target = c;
        }
    }
    if let Some(value) = date {
        document.edit(|m| {
            if let Some(c) = m.controls.iter_mut().find(|c| c.name == name) {
                c.date = (!value.is_empty()).then_some(value);
            }
        })?;
    }
    if let Some(value) = custom {
        document.edit(|m| {
            if let Some(c) = m.controls.iter_mut().find(|c| c.name == name) {
                c.custom = (!value.is_empty()).then_some(value);
            }
        })?;
    }
    for (event, value) in handlers {
        document.edit(|m| {
            if let Some(c) = m.controls.iter_mut().find(|c| c.name == name) {
                if value.is_empty() {
                    c.events.remove(&event);
                } else {
                    c.events.insert(event, value);
                }
            }
        })?;
    }
    if let Some(new) = rename {
        document.rename(name, &new)?;
    }
    Ok(())
}
