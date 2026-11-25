//! Правая панель: свойства и события выбранного контрола или самой формы.

use eframe::egui::{self, DragValue, Grid, TextEdit};
use rustforms_model::{Dock, Event, Kind, Side};

use crate::{
    app::{Designer, Request},
    document::Document,
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
    Grid::new("form events").num_columns(2).show(ui, |ui| {
        load = event_rows(ui, None, Event::Load, form.load.as_deref(), request);
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
            Kind::ComboBox | Kind::ListBox | Kind::MenuStrip => {
                let hint = if c.kind == Kind::MenuStrip {
                    "Пункты вида Меню/Пункт; Меню/- это разделитель"
                } else {
                    "Каждая строка списка с новой строки"
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
            Kind::ProgressBar => {
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
            Kind::Label => color_row(ui, "Цвет текста", &mut c.fore_color),
            Kind::Panel => {
                color_row(ui, "Фон", &mut c.back_color);
                ui.label("Рамка");
                ui.checkbox(&mut c.border, "");
                ui.end_row();
            }
            Kind::Button => {}
        }
        ui.label("Dock");
        egui::ComboBox::from_id_salt(("dock", name))
            .selected_text(c.dock.name())
            .show_ui(ui, |ui| {
                for dock in Dock::ALL {
                    ui.selectable_value(&mut c.dock, dock, dock.name());
                }
            });
        ui.end_row();
        ui.label("Якорь");
        ui.add_enabled_ui(c.dock == Dock::None, |ui| {
            ui.horizontal(|ui| {
                for side in Side::ALL {
                    let mut on = c.anchor.contains(&side);
                    if ui.checkbox(&mut on, side.name()).changed() {
                        if on {
                            c.anchor.insert(side);
                        } else {
                            c.anchor.remove(&side);
                        }
                    }
                }
            });
        });
        ui.end_row();
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
            for &event in c.kind.events() {
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
