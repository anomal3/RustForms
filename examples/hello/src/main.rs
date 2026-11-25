// Форма описана в main_form.rfrm; main_form.designer.rs создан из неё.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_form;

fn main() -> Result<(), rustforms::RunError> {
    rustforms::run(main_form::MainForm::create())
}
