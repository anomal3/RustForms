use std::fmt::Write;

use crate::{
    Align, Control, Dock, Error, Event, FormModel, Kind, default_anchor, model::{parse_color, parse_date},
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
        // Таймер на форме не виден, поэтому у него нет ни места, ни размера.
        if c.kind == Kind::Timer {
            let (name, interval) = (&c.name, c.interval);
            if c.enabled {
                line!("        let mut {name} = Timer::new({interval});");
                line!("        {name}.start();");
            } else {
                line!("        let {name} = Timer::new({interval});");
            }
            continue;
        }
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
    let empty = model.controls.is_empty() && model.handlers().is_empty();
    line!("        let {}view = View::new();", if empty { "" } else { "mut " });
    for c in &model.controls {
        let field = format!("|f: &mut Self| &mut f.{}", c.name);
        // Номер нужен только тем, на кого ссылаются: событиям и дочерним контролам.
        let used = !c.events.is_empty()
            || model.controls.iter().any(|child| child.parent.as_deref() == Some(&c.name));
        let binding = if used { format!("let {}_id = ", c.name) } else { String::new() };
        if c.kind == Kind::Timer {
            line!("        {binding}view.add_timer({field});");
            continue;
        }
        match &c.parent {
            Some(parent) => {
                line!("        {binding}view.add_to({parent}_id, {field});");
            }
            None => {
                line!("        {binding}view.add({field});");
            }
        }
    }
    for (event, handler) in
        [(Event::Load, &form.load), (Event::Closing, &form.closing), (Event::Resize, &form.resize)]
    {
        if let Some(handler) = handler {
            line!("        view.on_{}(Self::{handler});", event.name());
        }
    }
    for c in &model.controls {
        for (event, handler) in &c.events {
            if !Event::FORM.contains(event) {
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
    if c.kind == Kind::DateTimePicker
        && let Some(date) = &c.date
    {
        let (year, month, day) = parse_date(date)?;
        lines.push(format!("set_value(Date::new({year}, {month}, {day}))"));
    }
    if c.kind == Kind::SplitContainer {
        if c.distance != 100 {
            lines.push(format!("set_distance({:?})", c.distance as f32));
        }
        if c.horizontal {
            lines.push("set_horizontal(true)".into());
        }
    }
    if c.kind == Kind::DataGridView && c.editable {
        lines.push("set_editable(true)".into());
    }
    if !c.context.is_empty() {
        lines.push(format!("set_context_menu({:?})", c.context));
    }
    if c.kind == Kind::TabControl
        && let Some(selected) = c.selected
    {
        lines.push(format!("set_selected_index({selected})"));
    }
    if c.kind.checkable() && c.checked {
        lines.push("set_checked(true)".into());
    }
    if c.kind.has_items() {
        if !c.items.is_empty() {
            let method = if c.kind == Kind::DataGridView { "set_columns" } else { "set_items" };
            lines.push(format!("{method}({:?})", c.items));
        }
        if let Some(selected) = c.selected.filter(|_| c.kind.selectable()) {
            lines.push(format!("set_selected_index(Some({selected}))"));
        }
    }
    if c.kind == Kind::PictureBox
        && let Some(image) = c.image.as_deref().filter(|i| !i.is_empty())
    {
        // Картинка вшивается в программу; путь — от папки формы.
        lines.push(format!("set_image_png(include_bytes!({:?}))", image.replace('\\', "/")));
    }
    if c.kind.ranged() {
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
    if let Some(color) = &c.fore_color {
        lines.push(format!("set_fore_color({})", color_code(color)?));
    }
    if let Some(color) = &c.back_color {
        lines.push(format!("set_back_color({})", color_code(color)?));
    }
    if let Some(size) = c.font_size {
        lines.push(format!("set_font_size({size:?})"));
    }
    if c.bold {
        lines.push("set_bold(true)".into());
    }
    if let Some(text) = c.tooltip.as_deref().filter(|t| !t.is_empty()) {
        lines.push(format!("set_tooltip({text:?})"));
    }
    if c.tab_index != 0 {
        lines.push(format!("set_tab_index({})", c.tab_index));
    }
    if let Some(align) = c.align {
        let value = match align {
            Align::Left => "Left",
            Align::Center => "Center",
            Align::Right => "Right",
        };
        lines.push(format!("set_text_align(TextAlign::{value})"));
    }
    if c.kind == Kind::TextBox {
        if c.read_only {
            lines.push("set_read_only(true)".into());
        }
        if c.password {
            lines.push("set_password(true)".into());
        }
    }
    if c.kind == Kind::Panel && !c.border {
        lines.push("border = false".into());
    }
    Ok(lines)
}

fn color_code(text: &str) -> Result<String, Error> {
    let (r, g, b) = parse_color(text)?;
    Ok(format!("Color::rgb({r}, {g}, {b})"))
}
