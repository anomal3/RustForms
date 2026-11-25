//! Окно дизайнера: файлы, панель инструментов, горячие клавиши.

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use eframe::egui::{self, Event as Input, Key, Modifiers, Sense, ViewportCommand};
use rustforms_model::{
    Event, FormModel, Kind, attach_handler, load, new_form, new_project, save,
};

use crate::{canvas::Drag, document::Document, preview::Preview};

/// Строка для плагина IDE, запустившего дизайнер.
fn tell_ide(message: std::fmt::Arguments) {
    let mut out = std::io::stdout().lock();
    // Если IDE уже закрыла канал, сообщать некому.
    let _ = writeln!(out, "{message}");
    let _ = out.flush();
}

pub struct Open {
    pub path: PathBuf,
    pub document: Document,
    saved: FormModel,
}

impl Open {
    fn dirty(&self) -> bool {
        self.document.model != self.saved
    }
}

/// Действия, которые нельзя выполнить посреди отрисовки панели.
#[derive(Clone)]
pub enum Request {
    Handler { control: Option<String>, event: Option<Event> },
    NewProject,
    NewForm,
    OpenFile,
    Quit,
}

pub struct Designer {
    pub open: Option<Open>,
    pub preview: Preview,
    pub tool: Option<Kind>,
    pub tool_dragged: bool,
    pub drag: Option<Drag>,
    /// Левый верхний угол клиентской области формы на экране (нужен тестам).
    pub origin: egui::Pos2,
    pub snap: bool,
    pub status: String,
    pub request: Option<Request>,
    /// Действие, ждущее ответа «сохранить изменения?».
    pending: Option<Request>,
    quitting: bool,
    /// Запущен из IDE: сообщать ей через stdout, что открыть.
    ide: bool,
}

impl Designer {
    pub fn new(file: Option<PathBuf>, ide: bool) -> Self {
        let mut designer = Self {
            open: None,
            preview: Preview::default(),
            tool: None,
            tool_dragged: false,
            drag: None,
            origin: egui::Pos2::ZERO,
            snap: true,
            status: String::new(),
            request: None,
            pending: None,
            quitting: false,
            ide,
        };
        if let Some(file) = file {
            designer.open_path(file);
        }
        designer
    }

    fn open_path(&mut self, path: PathBuf) {
        // Относительный путь IDE не поймёт.
        let path = std::path::absolute(&path).unwrap_or(path);
        match load(&path) {
            Ok(model) => {
                self.status = format!("Открыта {}", path.display());
                self.open =
                    Some(Open { path, saved: model.clone(), document: Document::new(model) });
                self.tool = None;
                self.drag = None;
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    fn save(&mut self) -> bool {
        let Some(open) = &mut self.open else { return false };
        match save(&open.path, &open.document.model) {
            Ok(()) => {
                open.saved = open.document.model.clone();
                self.status = "Сохранено".into();
                true
            }
            Err(error) => {
                self.status = format!("Не сохранено: {error}");
                false
            }
        }
    }

    /// Двойной клик: сохранить, создать обработчик и показать, где он.
    fn handler(&mut self, control: Option<String>, event: Option<Event>) {
        if !self.save() {
            return;
        }
        let Some(open) = &mut self.open else { return };
        match attach_handler(&open.path, control.as_deref(), event) {
            Ok(at) => {
                if let Ok(model) = load(&open.path) {
                    // Назначение обработчика — тоже правка, её можно отменить.
                    open.document.checkpoint(None);
                    open.document.reset(model.clone());
                    open.saved = model;
                }
                let file = at.file.display();
                self.status = format!("Обработчик: {file}, строка {}", at.line);
                if self.ide {
                    tell_ide(format_args!("RUSTFORMS-OPEN\t{}\t{file}", at.line));
                }
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    fn run_project(&mut self) {
        if !self.save() {
            return;
        }
        let Some(open) = &self.open else { return };
        let root = open.path.ancestors().skip(1).find(|d| d.join("Cargo.toml").exists());
        self.status = match root {
            None => "Рядом с формой нет Cargo.toml".into(),
            // В IDE вывод и ошибки компиляции должны быть видны в ней.
            Some(root) if self.ide => {
                tell_ide(format_args!("RUSTFORMS-RUN\t{}", root.display()));
                "Запуск передан в IDE".into()
            }
            Some(root) => match Command::new("cargo").arg("run").current_dir(root).spawn() {
                Ok(_) => "cargo run запущен, вывод — в консоли дизайнера".into(),
                Err(error) => format!("Не удалось запустить cargo: {error}"),
            },
        };
    }

    fn perform(&mut self, ctx: &egui::Context, request: Request) {
        match request {
            Request::Handler { control, event } => self.handler(control, event),
            Request::OpenFile => {
                let dialog = rfd::FileDialog::new().add_filter("Форма RustForms", &["rfrm"]);
                if let Some(path) = dialog.pick_file() {
                    self.open_path(path);
                }
            }
            Request::NewForm => {
                let dialog = rfd::FileDialog::new()
                    .set_title("Новая форма")
                    .add_filter("Форма RustForms", &["rfrm"])
                    .set_file_name("second_form.rfrm");
                if let Some(path) = dialog.save_file() {
                    self.create_form(&path);
                }
            }
            Request::NewProject => {
                let dialog = rfd::FileDialog::new()
                    .set_title("Пустая папка нового проекта (имя латиницей)");
                if let Some(dir) = dialog.pick_folder() {
                    let library = Path::new(env!("CARGO_MANIFEST_DIR")).with_file_name("rustforms");
                    match new_project(&dir, &library) {
                        Ok(form) => self.open_path(form),
                        Err(error) => self.status = error.to_string(),
                    }
                }
            }
            Request::Quit => {
                self.quitting = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }

    /// `second_form.rfrm` превращается в структуру `SecondForm`.
    fn create_form(&mut self, path: &Path) {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        let name: String = stem
            .split(['_', '-', ' '])
            .flat_map(|word| {
                let mut chars = word.chars();
                chars.next().map(|c| c.to_ascii_uppercase()).into_iter().chain(chars)
            })
            .collect();
        let dir = path.parent().unwrap_or(Path::new("."));
        match new_form(dir, &name) {
            Ok(form) => {
                self.open_path(form);
                self.status = "Форма создана. Не забудьте добавить `mod ...;` в main.rs".into();
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    /// Выполнить, предварительно спросив про несохранённое.
    fn guarded(&mut self, ctx: &egui::Context, request: Request) {
        let discards = !matches!(request, Request::Handler { .. });
        if discards && self.open.as_ref().is_some_and(Open::dirty) {
            self.pending = Some(request);
        } else {
            self.perform(ctx, request);
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let command = |key| ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, key));
        if command(Key::S) {
            self.save();
        }
        if command(Key::O) {
            self.request = Some(Request::OpenFile);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5)) {
            self.run_project();
        }
        // Остальное не должно мешать вводу в поля свойств.
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let redo = ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                || i.consume_key(Modifiers::COMMAND, Key::Y)
        });
        let undo = command(Key::Z);
        let events = ctx.input(|i| i.events.clone());
        let (delete, escape, shift, arrows) = ctx.input(|i| {
            let pressed = |key| i32::from(i.key_pressed(key));
            (
                i.key_pressed(Key::Delete),
                i.key_pressed(Key::Escape),
                i.modifiers.shift,
                (
                    (pressed(Key::ArrowRight) - pressed(Key::ArrowLeft)) as f32,
                    (pressed(Key::ArrowDown) - pressed(Key::ArrowUp)) as f32,
                ),
            )
        });
        let Some(open) = &mut self.open else { return };
        let document = &mut open.document;
        if redo {
            document.redo();
        } else if undo {
            document.undo();
        }
        if escape {
            if self.tool.take().is_none() {
                let parent = document
                    .selected
                    .as_deref()
                    .and_then(|n| document.control(n))
                    .and_then(|c| c.parent.clone());
                document.selected = parent;
            }
            return;
        }
        for event in events {
            match event {
                Input::Copy | Input::Cut => {
                    let Some(name) = document.selected.clone() else { continue };
                    if let Some(text) = document.copy(&name) {
                        ctx.copy_text(text);
                        if matches!(event, Input::Cut) {
                            document.delete(&name);
                        }
                    }
                }
                Input::Paste(text) => {
                    document.paste(&text);
                }
                _ => {}
            }
        }
        let Some(name) = document.selected.clone() else { return };
        if delete {
            document.delete(&name);
        } else if arrows != (0.0, 0.0) {
            document.checkpoint(Some("arrows"));
            if let Some(c) = document.control_mut(&name) {
                if shift {
                    c.width = (c.width + arrows.0).max(1.0);
                    c.height = (c.height + arrows.1).max(1.0);
                } else {
                    c.x += arrows.0;
                    c.y += arrows.1;
                }
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Новый проект").clicked() {
                self.request = Some(Request::NewProject);
            }
            if ui.button("Новая форма").clicked() {
                self.request = Some(Request::NewForm);
            }
            if ui.button("Открыть").clicked() {
                self.request = Some(Request::OpenFile);
            }
            let dirty = self.open.as_ref().is_some_and(Open::dirty);
            if ui.add_enabled(dirty, egui::Button::new("Сохранить")).clicked() {
                self.save();
            }
            ui.separator();
            if let Some(open) = &mut self.open {
                let document = &mut open.document;
                let undo = ui.add_enabled(document.can_undo(), egui::Button::new("Отменить"));
                if undo.clicked() {
                    document.undo();
                }
                let redo = ui.add_enabled(document.can_redo(), egui::Button::new("Повторить"));
                if redo.clicked() {
                    document.redo();
                }
                ui.separator();
                ui.checkbox(&mut self.snap, "Сетка").on_hover_text("Alt временно отключает");
                if ui.button("Запустить (F5)").clicked() {
                    self.run_project();
                }
            }
        });
    }

    fn toolbox(&mut self, ui: &mut egui::Ui) {
        ui.heading("Элементы");
        for kind in Kind::ALL {
            let button = egui::Button::new(kind.name())
                .selected(self.tool == Some(kind))
                .sense(Sense::click_and_drag())
                .min_size(egui::vec2(ui.available_width(), 26.0));
            let response = ui.add_enabled(self.open.is_some(), button);
            if response.clicked() {
                self.tool = (self.tool != Some(kind)).then_some(kind);
            }
            if response.drag_started() {
                self.tool = Some(kind);
                self.tool_dragged = true;
            }
        }
        ui.separator();
        ui.heading("Дерево");
        let Some(open) = &mut self.open else { return };
        let document = &mut open.document;
        egui::ScrollArea::vertical().show(ui, |ui| {
            let form = document.model.form.name.clone();
            if ui.selectable_label(document.selected.is_none(), form).clicked() {
                document.selected = None;
            }
            let mut clicked = None;
            for c in &document.model.controls {
                let mut depth = 1;
                let mut parent = c.parent.as_deref();
                while let Some(p) = parent {
                    depth += 1;
                    parent = document.control(p).and_then(|c| c.parent.as_deref());
                }
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 10.0);
                    let selected = document.selected.as_deref() == Some(&c.name);
                    if ui.selectable_label(selected, &c.name).clicked() {
                        clicked = Some(c.name.clone());
                    }
                });
            }
            if clicked.is_some() {
                document.selected = clicked;
                document.end_merge();
            }
        });
    }

    fn confirm(&mut self, ctx: &egui::Context) {
        let Some(request) = self.pending.clone() else { return };
        egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
            ui.label("Сохранить изменения в форме?");
            ui.horizontal(|ui| {
                if ui.button("Сохранить").clicked() {
                    self.pending = None;
                    if self.save() {
                        self.perform(ctx, request.clone());
                    }
                }
                if ui.button("Не сохранять").clicked() {
                    self.pending = None;
                    if let Some(open) = &mut self.open {
                        open.document.reset(open.saved.clone());
                    }
                    self.perform(ctx, request.clone());
                }
                if ui.button("Отмена").clicked() {
                    self.pending = None;
                }
            });
        });
    }
}

impl eframe::App for Designer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested())
            && !self.quitting
            && self.open.as_ref().is_some_and(Open::dirty)
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.pending = Some(Request::Quit);
        }
        if self.pending.is_none() {
            self.shortcuts(&ctx);
        }

        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(&self.status);
        });
        egui::Panel::left("toolbox").default_size(150.0).show(ui, |ui| self.toolbox(ui));
        egui::Panel::right("properties").default_size(280.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.properties(ui));
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| self.canvas(ui));
        });
        if ctx.input(|i| i.pointer.primary_released()) {
            self.tool_dragged = false;
            if let Some(open) = &mut self.open {
                open.document.end_merge();
            }
        }

        if let Some(open) = &mut self.open {
            open.document.arrange();
        }

        if let Some(request) = self.request.take() {
            self.guarded(&ctx, request);
        }
        self.confirm(&ctx);

        let title = match &self.open {
            Some(open) => format!(
                "{}{} — RustForms Designer",
                open.path.file_name().unwrap_or_default().to_string_lossy(),
                if open.dirty() { " *" } else { "" }
            ),
            None => "RustForms Designer".into(),
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
    }
}
