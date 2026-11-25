use std::fmt::Write;

use crate::{
    Control, Dock, Error, Event, FormModel, Kind, default_anchor, model::parse_color,
};

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
    line!("use super::Data;");
    line!("");
    line!("/// Форма: её контролы и твои данные (`data`). Обработчики — в соседнем файле.");
    line!("#[allow(dead_code)]");
    line!("pub struct {name} {{");
    line!("    pub data: Data,");
    for c in &model.controls {
        line!("    pub {}: {},", c.name, type_name(c));
    }
    line!("}}");
    line!("");
    line!("impl {name} {{");
    line!("    pub fn create() -> Application<Self> {{");
    let mutable = if form.back_color.is_none() { "" } else { "mut " };
    line!(
        "        let {mutable}ui = Ui::new({:?}, {:?}, {:?});",
        form.title,
        form.width,
        form.height
    );
    if let Some(color) = &form.back_color {
        line!("        ui.back_color = {};", color_code(color)?);
    }

    // Сначала значения контролов, затем структура из них.
    for c in &model.controls {
        line!("");
        let constructor = match (&c.custom, c.kind) {
            (Some(path), _) => format!("<{path}>::default()"),
            (None, kind) if kind.captioned() => {
                format!("{}::new({:?})", kind.name(), c.text.as_deref().unwrap_or(""))
            }
            (None, kind) => format!("{}::new()", kind.name()),
        };
        let place = format!(".at({:?}, {:?}).size({:?}, {:?})", c.x, c.y, c.width, c.height);
        let setup = setup_lines(c)?;
        if setup.is_empty() {
            line!("        let {} = {constructor}{place};", c.name);
        } else {
            line!("        let mut {} = {constructor}{place};", c.name);
            for setting in setup {
                line!("        {}.{setting};", c.name);
            }
        }
    }
    line!("");
    line!("        let form = Self {{");
    line!("            data: Data::default(),");
    for c in &model.controls {
        line!("            {},", c.name);
    }
    line!("        }};");

    line!("");
    let empty = model.controls.is_empty() && form.load.is_none();
    line!("        let {}view = View::new();", if empty { "" } else { "mut " });
    for c in &model.controls {
        let field = format!("|f: &mut Self| &mut f.{}", c.name);
        // Номер нужен только тем, на кого ссылаются: событиям и дочерним контролам.
        let used = !c.events.is_empty()
            || model.controls.iter().any(|child| child.parent.as_deref() == Some(&c.name));
        let binding = if used { format!("let {}_id = ", c.name) } else { String::new() };
        match &c.parent {
            Some(parent) => {
                line!("        {binding}view.add_to({parent}_id, {field});");
            }
            None => {
                line!("        {binding}view.add({field});");
            }
        }
    }
    if let Some(handler) = &form.load {
        line!("        view.on_load(Self::{handler});");
    }
    for c in &model.controls {
        for (event, handler) in &c.events {
            if *event != Event::Load {
                line!("        view.on_{}({}_id, Self::{handler});", event.name(), c.name);
            }
        }
    }
    line!("");
    line!("        Application::new(ui, form, view)");
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
    let text_in_constructor = c.custom.is_none() && c.kind.captioned();
    // Режим задаётся до текста, иначе переводы строк потеряются.
    if c.kind == Kind::TextBox && c.multiline {
        lines.push("set_multiline(true)".into());
    }
    if let Some(text) = &c.text
        && !text_in_constructor
        && c.kind.has_text()
    {
        lines.push(format!("set_text({text:?})"));
    }
    if c.kind.checkable() && c.checked {
        lines.push("set_checked(true)".into());
    }
    if c.kind.has_items() {
        if !c.items.is_empty() {
            lines.push(format!("set_items({:?})", c.items));
        }
        if let Some(selected) = c.selected.filter(|_| c.kind.selectable()) {
            lines.push(format!("set_selected_index(Some({selected}))"));
        }
    }
    if c.kind == Kind::ProgressBar {
        if (c.minimum, c.maximum) != (0, 100) {
            lines.push(format!("set_range({}, {})", c.minimum, c.maximum));
        }
        if c.value != 0 {
            lines.push(format!("set_value({})", c.value));
        }
    }
    if c.dock != Dock::None {
        let side = match c.dock {
            Dock::Top => "Top",
            Dock::Bottom => "Bottom",
            Dock::Left => "Left",
            Dock::Right => "Right",
            Dock::Fill => "Fill",
            Dock::None => "None",
        };
        lines.push(format!("set_dock(Dock::{side})"));
    } else if c.anchor != default_anchor() {
        let sides: Vec<String> =
            c.anchor.iter().map(|s| format!("Anchor::{}", s.name().to_uppercase())).collect();
        let value =
            if sides.is_empty() { "Anchor::NONE".to_owned() } else { sides.join(" | ") };
        lines.push(format!("set_anchor({value})"));
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
