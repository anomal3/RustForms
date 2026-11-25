// Меню, системные диалоги и вторая форма.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_form;
mod name_form;

fn main() -> Result<(), rustforms::RunError> {
    rustforms::run(main_form::MainForm::create())
}
