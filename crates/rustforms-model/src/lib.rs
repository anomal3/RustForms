//! Формат `.rfrm`, генерация кода и вставка обработчиков.
//!
//! У формы три файла: `main_form.rfrm` (описание, главный источник),
//! `main_form.designer.rs` (генерируется) и `main_form.rs` (код пользователя —
//! сюда только дописываются заготовки обработчиков).

mod generate;
mod handlers;
mod model;
mod project;

pub use generate::designer_code;
pub use handlers::{Inserted, ensure_handler};
pub use model::{Control, Dock, Event, FormModel, FormProps, Kind, Side, default_anchor};
pub use project::{Location, attach_handler, load, new_form, new_project, save};

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

fn fail<T>(message: impl Into<String>) -> Result<T, Error> {
    Err(Error(message.into()))
}
