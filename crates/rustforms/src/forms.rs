//! Несколько окон: открытие второй формы из первой.

use std::any::Any;

use tiny_skia::Pixmap;

use crate::{Application, Key, Modifiers, Point, Ui};

type Done = Box<dyn FnOnce(&mut dyn Any, &mut dyn Any, bool, &mut Ui)>;

/// Форма без знания о типе её состояния — так ими управляет оконный слой.
pub(crate) trait AnyForm {
    fn ui(&mut self) -> &mut Ui;
    fn state(&mut self) -> &mut dyn Any;
    fn load(&mut self);
    fn use_system_clipboard(&mut self);
    fn mouse_move(&mut self, at: Point);
    fn mouse_leave(&mut self);
    fn mouse_down(&mut self, mods: Modifiers);
    fn mouse_up(&mut self);
    fn wheel(&mut self, lines: f32);
    fn key(&mut self, key: Key, mods: Modifiers);
    fn text_input(&mut self, text: &str);
    fn close_popups(&mut self);
    fn render(&mut self, pixmap: &mut Pixmap, scale: f32);
    fn finish(&mut self, done: Done, child: &mut dyn Any, accepted: bool);
}

impl<S: 'static> AnyForm for Application<S> {
    fn ui(&mut self) -> &mut Ui {
        &mut self.ui
    }

    fn state(&mut self) -> &mut dyn Any {
        &mut self.state
    }

    fn load(&mut self) {
        Application::load(self);
    }

    fn use_system_clipboard(&mut self) {
        Application::use_system_clipboard(self);
    }

    fn mouse_move(&mut self, at: Point) {
        Application::mouse_move(self, at);
    }

    fn mouse_leave(&mut self) {
        Application::mouse_leave(self);
    }

    fn mouse_down(&mut self, mods: Modifiers) {
        Application::mouse_down(self, mods);
    }

    fn mouse_up(&mut self) {
        Application::mouse_up(self);
    }

    fn wheel(&mut self, lines: f32) {
        Application::wheel(self, lines);
    }

    fn key(&mut self, key: Key, mods: Modifiers) {
        Application::key(self, key, mods);
    }

    fn text_input(&mut self, text: &str) {
        Application::text_input(self, text);
    }

    fn close_popups(&mut self) {
        Application::close_popups(self);
    }

    fn render(&mut self, pixmap: &mut Pixmap, scale: f32) {
        Application::render(self, pixmap, scale);
    }

    fn finish(&mut self, done: Done, child: &mut dyn Any, accepted: bool) {
        done(&mut self.state, child, accepted, &mut self.ui);
        self.layout();
    }
}

/// Форма, которую обработчик попросил открыть.
pub struct PendingForm {
    pub(crate) form: Box<dyn AnyForm>,
    pub(crate) modal: bool,
    pub(crate) done: Option<Done>,
}

impl PendingForm {
    pub fn is_modal(&self) -> bool {
        self.modal
    }

    pub fn ui(&mut self) -> &mut Ui {
        self.form.ui()
    }

    /// Состояние открываемой формы (для проверок без окна).
    pub fn state<C: 'static>(&mut self) -> Option<&mut C> {
        self.form.state().downcast_mut()
    }
}

impl Ui {
    /// Открывает ещё одну форму; обе остаются доступными.
    pub fn show<C: 'static>(&mut self, form: Application<C>) {
        self.pending.push(PendingForm { form: Box::new(form), modal: false, done: None });
    }

    /// Открывает форму как диалог: пока она открыта, эта не принимает ввод.
    ///
    /// `done` вызывается после закрытия: `me` — эта форма, `dialog` — закрытая,
    /// `accepted` — закрыли ли её через [`Ui::accept`] (кнопка «ОК»).
    /// Обработчик возвращается сразу, не дожидаясь закрытия.
    pub fn show_dialog<P: 'static, C: 'static>(
        &mut self,
        form: Application<C>,
        done: impl FnOnce(&mut P, &mut C, bool, &mut Ui) + 'static,
    ) {
        let done: Done = Box::new(move |parent, child, accepted, ui| {
            let parent = parent.downcast_mut().expect("тип формы в show_dialog не тот");
            let child = child.downcast_mut().expect("тип диалога в show_dialog не тот");
            done(parent, child, accepted, ui);
        });
        self.pending.push(PendingForm { form: Box::new(form), modal: true, done: Some(done) });
    }
}

impl<S: 'static> Application<S> {
    /// Забирает запрошенную к открытию форму. Нужен проверкам без окна.
    pub fn take_pending(&mut self) -> Option<PendingForm> {
        (!self.ui.pending.is_empty()).then(|| self.ui.pending.remove(0))
    }

    /// Сообщает, что открытая из этой формы закрылась.
    pub fn finish_pending(&mut self, mut child: PendingForm) {
        if let Some(done) = child.done.take() {
            let accepted = child.form.ui().accepted;
            AnyForm::finish(self, done, child.form.state(), accepted);
        }
    }
}
