//! Операции над тремя файлами формы на диске. Ими пользуются дизайнер и плагины.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{Error, Event, FormModel, designer_code, ensure_handler, fail, model::identifier};

/// Куда перейти в редакторе.
#[derive(Debug, PartialEq)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
}

struct Paths {
    rfrm: PathBuf,
    code: PathBuf,
    designer: PathBuf,
}

fn paths(rfrm: &Path) -> Result<Paths, Error> {
    if rfrm.extension().is_none_or(|e| e != "rfrm") {
        return fail(format!("{}: ожидается файл .rfrm", rfrm.display()));
    }
    Ok(Paths {
        rfrm: rfrm.to_owned(),
        code: rfrm.with_extension("rs"),
        designer: rfrm.with_extension("designer.rs"),
    })
}

fn read(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))
}

/// Пишет, только если содержимое изменилось, чтобы не вызывать лишних пересборок.
fn write(path: &Path, text: &str) -> Result<(), Error> {
    if fs::read_to_string(path).is_ok_and(|old| old == text) {
        return Ok(());
    }
    fs::write(path, text).map_err(|e| Error(format!("{}: {e}", path.display())))
}

pub fn load(rfrm: &Path) -> Result<FormModel, Error> {
    let files = paths(rfrm)?;
    FormModel::from_toml(&read(&files.rfrm)?)
        .map_err(|e| Error(format!("{}: {e}", rfrm.display())))
}

fn template(name: &str, stem: &str) -> String {
    format!(
        "// Пока обработчиков нет, этим ничего не пользуется.
#[allow(unused_imports)]
use rustforms::prelude::*;

#[path = \"{stem}.designer.rs\"]
mod designer;

pub use designer::{name};

/// Свои данные формы: добавляй поля сюда, в обработчиках они доступны как `self.data`.
#[derive(Default)]
pub struct Data {{}}

impl {name} {{
}}
"
    )
}

/// Сохраняет описание, генерирует `*.designer.rs` и дописывает недостающие обработчики.
pub fn save(rfrm: &Path, model: &FormModel) -> Result<(), Error> {
    let files = paths(rfrm)?;
    // Всё вычисляется до первой записи, чтобы ошибка не оставила файлы вразнобой.
    let designer = designer_code(model)?;
    let description = model.to_toml()?;
    let stem = files.rfrm.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let mut code = if files.code.exists() {
        read(&files.code)?
    } else {
        template(&model.form.name, stem)
    };
    for (handler, event) in model.handlers() {
        if let Some(updated) = ensure_handler(&code, &model.form.name, handler, event)?.source {
            code = updated;
        }
    }
    write(&files.code, &code)?;
    write(&files.designer, &designer)?;
    write(&files.rfrm, &description)
}

/// Двойной клик в дизайнере: назначает обработчик (если его ещё нет) и говорит, где он.
/// `control = None` — сама форма, `event = None` — событие по умолчанию.
pub fn attach_handler(
    rfrm: &Path,
    control: Option<&str>,
    event: Option<Event>,
) -> Result<Location, Error> {
    let files = paths(rfrm)?;
    let mut model = load(rfrm)?;
    let (handler, event) = match control {
        None => {
            if event.is_some_and(|e| e != Event::Load) {
                return fail("у формы есть только событие load");
            }
            let stem = files.rfrm.file_stem().and_then(|s| s.to_str()).unwrap_or("form");
            let name = model.form.load.get_or_insert_with(|| format!("{stem}_load"));
            (name.clone(), Event::Load)
        }
        Some(wanted) => {
            let Some(c) = model.controls.iter_mut().find(|c| c.name == wanted) else {
                return fail(format!("контрола `{wanted}` нет на форме"));
            };
            let Some(event) = event.or_else(|| c.kind.events().first().copied()) else {
                return fail(format!("у {} нет событий", c.kind.name()));
            };
            let default = format!("{}_{}", c.name, event.name());
            (c.events.entry(event).or_insert(default).clone(), event)
        }
    };
    save(rfrm, &model)?;
    let line = ensure_handler(&read(&files.code)?, &model.form.name, &handler, event)?.line;
    Ok(Location { file: files.code, line })
}

/// Создаёт три файла новой формы; существующие не перезаписывает.
pub fn new_form(dir: &Path, name: &str) -> Result<PathBuf, Error> {
    identifier(name)?;
    let mut stem = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            stem.push('_');
        }
        stem.push(ch.to_ascii_lowercase());
    }
    let files = paths(&dir.join(format!("{stem}.rfrm")))?;
    if let Some(taken) = [&files.rfrm, &files.code, &files.designer].into_iter().find(|p| p.exists())
    {
        return fail(format!("{} уже существует", taken.display()));
    }
    save(&files.rfrm, &FormModel::new(name))?;
    Ok(files.rfrm)
}

/// Создаёт новый проект Cargo с главной формой и возвращает путь к её `.rfrm`.
/// `rustforms` — папка крейта библиотеки.
pub fn new_project(dir: &Path, rustforms: &Path) -> Result<PathBuf, Error> {
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_owned();
    let valid = name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !valid {
        return fail("имя папки проекта: латиница, цифры, - и _, начинается с буквы");
    }
    if dir.join("Cargo.toml").exists() {
        return fail(format!("в {} уже есть проект", dir.display()));
    }
    let src = dir.join("src");
    fs::create_dir_all(&src).map_err(|e| Error(format!("{}: {e}", src.display())))?;
    if src.join("main.rs").exists() {
        return fail(format!("{} уже существует", src.join("main.rs").display()));
    }
    let library = rustforms.display().to_string().replace('\\', "/");
    // Пустой [workspace] не даёт проекту прицепиться к внешнему workspace.
    let manifest = format!(
        "[package]
name = \"{name}\"
version = \"0.1.0\"
edition = \"2024\"

[dependencies]
rustforms = {{ path = {library:?} }}

[profile.dev.package.\"*\"]
opt-level = 2

[workspace]
"
    );
    let rfrm = new_form(&src, "MainForm")?;
    write(
        &src.join("main.rs"),
        "#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]

mod main_form;

fn main() -> Result<(), rustforms::RunError> {
    rustforms::run(main_form::MainForm::create())
}
",
    )?;
    write(&dir.join(".gitignore"), "/target/\n")?;
    write(&dir.join("Cargo.toml"), &manifest)?;
    Ok(rfrm)
}
