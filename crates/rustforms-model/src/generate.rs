use std::fmt::Write;

use crate::{Control, Error, Event, FormModel, Kind, model::parse_color};

/// Содержимое `*.designer.rs`. Модуль подключается из файла формы как дочерний.
pub fn designer_code(model: &FormModel) -> Result<String, Error> {
    model.validate()?;
    let form = &model.form;
    let name = &form.name;
    let mut out = String::new();
    let w = &mut out;

    // Запись в String не может завершиться ошибкой.
    macro_rules! line {
        ($($arg:tt)*) => {
            let _ = writeln!(w, $($arg)*);
        };
    }

    line!("// Создано дизайнером RustForms из .rfrm. Не редактируйте: изменения пропадут.");
    line!("");
    // Имена контролов выбирает пользователь, замечания о стиле здесь только мешают.
    line!("#![allow(non_snake_case)]");
    line!("");
    line!("use rustforms::prelude::*;");
    line!("");
    line!("use super::{name};");
    line!("");
    line!("#[allow(dead_code)]");
    if model.controls.is_empty() {
        line!("pub struct Controls {{}}");
    } else {
        line!("pub struct Controls {{");
        for c in &model.controls {
            line!("    pub {}: Handle<{}>,", c.name, type_name(c));
        }
        line!("}}");
    }
    line!("");
    line!("impl Controls {{");
    line!("    pub fn build(events: &mut Events<{name}>) -> (Ui, Self) {{");
    let mutable = if model.controls.is_empty() && form.back_color.is_none() { "" } else { "mut " };
    line!(
        "        let {mutable}ui = Ui::new({:?}, {:?}, {:?});",
        form.title,
        form.width,
        form.height
    );
    if let Some(color) = &form.back_color {
        line!("        ui.back_color = {};", color_code(color)?);
    }

    for c in &model.controls {
        line!("");
        let constructor = match (&c.custom, c.kind) {
            (Some(path), _) => format!("<{path}>::default()"),
            (None, Kind::TextBox | Kind::Panel) => format!("{}::new()", c.kind.name()),
            (None, kind) => {
                format!("{}::new({:?})", kind.name(), c.text.as_deref().unwrap_or(""))
            }
        };
        let place = format!(".at({:?}, {:?}).size({:?}, {:?})", c.x, c.y, c.width, c.height);
        let setup = setup_lines(c)?;
        if setup.is_empty() {
            line!("        let control = {constructor}{place};");
        } else {
            line!("        let mut control = {constructor}{place};");
            for setting in setup {
                line!("        control.{setting};");
            }
        }
        match &c.parent {
            Some(parent) => {
                line!("        let {} = ui.add_to({parent}, control);", c.name);
            }
            None => {
                line!("        let {} = ui.add(control);", c.name);
            }
        }
    }

    line!("");
    if model.handlers().is_empty() {
        line!("        let _ = events;");
    }
    if let Some(handler) = &form.load {
        line!("        events.on_load({name}::{handler});");
    }
    for c in &model.controls {
        for (event, handler) in &c.events {
            if *event != Event::Load {
                line!("        events.on_{}({}, {name}::{handler});", event.name(), c.name);
            }
        }
    }

    let fields: Vec<&str> = model.controls.iter().map(|c| c.name.as_str()).collect();
    line!("");
    if fields.is_empty() {
        line!("        (ui, Self {{}})");
    } else {
        line!("        (ui, Self {{ {} }})", fields.join(", "));
    }
    line!("    }}");
    line!("}}");
    Ok(out)
}

fn type_name(c: &Control) -> &str {
    c.custom.as_deref().unwrap_or(c.kind.name())
}

/// Вызовы после конструктора: всё, что не задаётся им самим.
fn setup_lines(c: &Control) -> Result<Vec<String>, Error> {
    let mut lines = Vec::new();
    let text_in_constructor =
        c.custom.is_none() && matches!(c.kind, Kind::Button | Kind::Label | Kind::CheckBox);
    if let Some(text) = &c.text
        && !text_in_constructor
        && c.kind != Kind::Panel
    {
        lines.push(format!("set_text({text:?})"));
    }
    if c.kind == Kind::CheckBox && c.checked {
        lines.push("set_checked(true)".into());
    }
    if !c.visible {
        lines.push("set_visible(false)".into());
    }
    if !c.enabled {
        lines.push("set_enabled(false)".into());
    }
    if c.kind == Kind::Label
        && let Some(color) = &c.fore_color
    {
        lines.push(format!("fore_color = {}", color_code(color)?));
    }
    if c.kind == Kind::Panel {
        if let Some(color) = &c.back_color {
            lines.push(format!("back_color = {}", color_code(color)?));
        }
        if !c.border {
            lines.push("border = false".into());
        }
    }
    Ok(lines)
}

fn color_code(text: &str) -> Result<String, Error> {
    let (r, g, b) = parse_color(text)?;
    Ok(format!("Color::rgb({r}, {g}, {b})"))
}
