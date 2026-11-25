use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{CheckBox, Handle, TextBox, Ui, ui::ControlId};

/// [`TextBox`] и всё, что от него «унаследовано».
pub trait RaisesTextChanged {}
impl RaisesTextChanged for TextBox {}
impl<T: Deref> RaisesTextChanged for T where T::Target: RaisesTextChanged {}

/// [`CheckBox`] и всё, что от него «унаследовано».
pub trait RaisesCheckedChanged {}
impl RaisesCheckedChanged for CheckBox {}
impl<T: Deref> RaisesCheckedChanged for T where T::Target: RaisesCheckedChanged {}

#[derive(Clone, Copy, Debug)]
pub struct ClickArgs {
    pub sender: ControlId,
}

#[derive(Clone, Debug)]
pub struct TextChangedArgs {
    pub sender: ControlId,
    pub text: String,
}

#[derive(Clone, Copy, Debug)]
pub struct CheckedChangedArgs {
    pub sender: ControlId,
    pub checked: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct LoadArgs;

/// Что произошло с контролом; из этого строятся аргументы обработчиков.
#[derive(Clone, Debug)]
pub(crate) enum Signal {
    Click,
    TextChanged(String),
    CheckedChanged(bool),
}

type Handler<S, A> = Box<dyn FnMut(&mut S, &mut Ui, &A)>;

/// Подписки формы. `S` — твоя структура формы, поэтому обработчики это обычные методы
/// вида `fn button1_click(&mut self, ui: &mut Ui, e: &ClickArgs)`.
///
/// События возникают только от действий пользователя: изменение свойства из кода их не вызывает.
pub struct Events<S> {
    load: Vec<Handler<S, LoadArgs>>,
    click: Vec<(ControlId, Handler<S, ClickArgs>)>,
    text_changed: Vec<(ControlId, Handler<S, TextChangedArgs>)>,
    checked_changed: Vec<(ControlId, Handler<S, CheckedChangedArgs>)>,
}

impl<S> Default for Events<S> {
    fn default() -> Self {
        Self {
            load: Vec::new(),
            click: Vec::new(),
            text_changed: Vec::new(),
            checked_changed: Vec::new(),
        }
    }
}

impl<S> Events<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_load(&mut self, handler: impl FnMut(&mut S, &mut Ui, &LoadArgs) + 'static) {
        self.load.push(Box::new(handler));
    }

    pub fn on_click<T>(
        &mut self,
        control: Handle<T>,
        handler: impl FnMut(&mut S, &mut Ui, &ClickArgs) + 'static,
    ) {
        self.click.push((control.id(), Box::new(handler)));
    }

    pub fn on_text_changed<T: RaisesTextChanged>(
        &mut self,
        control: Handle<T>,
        handler: impl FnMut(&mut S, &mut Ui, &TextChangedArgs) + 'static,
    ) {
        self.text_changed.push((control.id(), Box::new(handler)));
    }

    pub fn on_checked_changed<T: RaisesCheckedChanged>(
        &mut self,
        control: Handle<T>,
        handler: impl FnMut(&mut S, &mut Ui, &CheckedChangedArgs) + 'static,
    ) {
        self.checked_changed.push((control.id(), Box::new(handler)));
    }

    /// Подключает базовую форму: `build` создаёт её контролы и подписки, а `part`
    /// достаёт её из производной. Обработчики базовой формы вызываются раньше тех,
    /// что подписаны после этого вызова.
    pub fn include<B: 'static, R>(
        &mut self,
        part: fn(&mut S) -> &mut B,
        build: impl FnOnce(&mut Events<B>) -> R,
    ) -> R
    where
        S: 'static,
    {
        fn adapt<S: 'static, B: 'static, A: 'static>(
            part: fn(&mut S) -> &mut B,
            handler: Handler<B, A>,
        ) -> Handler<S, A> {
            let handler = Rc::new(RefCell::new(handler));
            Box::new(move |state, ui, args| (handler.borrow_mut())(part(state), ui, args))
        }
        fn adapt_all<S: 'static, B: 'static, A: 'static>(
            part: fn(&mut S) -> &mut B,
            list: Vec<(ControlId, Handler<B, A>)>,
        ) -> impl Iterator<Item = (ControlId, Handler<S, A>)> {
            list.into_iter().map(move |(id, h)| (id, adapt(part, h)))
        }

        let mut inner = Events::<B>::new();
        let result = build(&mut inner);
        self.load.extend(inner.load.into_iter().map(|h| adapt(part, h)));
        self.click.extend(adapt_all(part, inner.click));
        self.text_changed.extend(adapt_all(part, inner.text_changed));
        self.checked_changed.extend(adapt_all(part, inner.checked_changed));
        result
    }

    pub(crate) fn load(&mut self, state: &mut S, ui: &mut Ui) {
        for handler in &mut self.load {
            handler(state, ui, &LoadArgs);
        }
    }

    pub(crate) fn dispatch(&mut self, sender: ControlId, signal: Signal, state: &mut S, ui: &mut Ui) {
        fn call<S, A>(
            list: &mut [(ControlId, Handler<S, A>)],
            sender: ControlId,
            args: A,
            state: &mut S,
            ui: &mut Ui,
        ) {
            for (_, handler) in list.iter_mut().filter(|(id, _)| *id == sender) {
                handler(state, ui, &args);
            }
        }
        match signal {
            Signal::Click => call(&mut self.click, sender, ClickArgs { sender }, state, ui),
            Signal::TextChanged(text) => {
                let args = TextChangedArgs { sender, text };
                call(&mut self.text_changed, sender, args, state, ui)
            }
            Signal::CheckedChanged(checked) => {
                let args = CheckedChangedArgs { sender, checked };
                call(&mut self.checked_changed, sender, args, state, ui)
            }
        }
    }
}
