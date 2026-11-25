//! Дизайнер форм RustForms: окно и те же действия из командной строки.

mod app;
mod canvas;
mod document;
mod preview;
mod properties;
mod render;
mod serve;
#[cfg(test)]
mod ui_tests;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use rustforms_model::{Error, Event, attach_handler, load, new_form, new_project, save};

const USAGE: &str = "rustforms-designer [--ide] [форма.rfrm]
rustforms-designer project <папка>
rustforms-designer new <папка> <ИмяФормы>
rustforms-designer generate <форма.rfrm>
rustforms-designer handler <форма.rfrm> [контрол] [событие]";

fn run(args: &[&str]) -> Result<String, Error> {
    match args {
        ["project", dir] => {
            let library = Path::new(env!("CARGO_MANIFEST_DIR")).with_file_name("rustforms");
            let dir = std::path::absolute(dir).map_err(|e| Error(e.to_string()))?;
            Ok(new_project(&dir, &library)?.display().to_string())
        }
        ["new", dir, name] => Ok(new_form(Path::new(dir), name)?.display().to_string()),
        ["generate", file] => {
            let file = Path::new(file);
            save(file, &load(file)?)?;
            Ok(String::new())
        }
        ["handler", file, rest @ ..] if rest.len() <= 2 => {
            let event = match rest.get(1) {
                None => None,
                Some(&name) => Some(
                    Event::ALL
                        .into_iter()
                        .find(|e| e.name() == name)
                        .ok_or_else(|| Error(format!("неизвестное событие `{name}`")))?,
                ),
            };
            let at = attach_handler(Path::new(file), rest.first().copied(), event)?;
            Ok(format!("{}:{}", at.file.display(), at.line))
        }
        _ => Err(Error(USAGE.to_owned())),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
    let ide = args.first() == Some(&"--ide");
    if ide {
        args.remove(0);
    }
    match args.as_slice() {
        ["serve", file] => return serve::run(Path::new(file)),
        [] => return window(None, ide),
        [file] if file.ends_with(".rfrm") => return window(Some(PathBuf::from(file)), ide),
        _ => {}
    }
    match run(&args) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn window(file: Option<PathBuf>, ide: bool) -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "RustForms Designer",
        options,
        Box::new(move |_| Ok(Box::new(app::Designer::new(file, ide)))),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
