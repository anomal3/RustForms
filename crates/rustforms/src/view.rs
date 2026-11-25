use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    AnyControl, CheckBox, Container, ControlId, ControlImpl, Handle, TextBox, Ui,
};

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
type Getter<S> = Box<dyn Fn(&mut S) -> &mut (dyn AnyControl + 'static)>;

pub(crate) struct Node<S> {
    pub id: ControlId,
    pub parent: Option<ControlId>,
    pub get: Getter<S>,
}

/// Помогает компилятору вывести замыкание с «любым» временем жизни.
fn getter<S>(get: impl Fn(&mut S) -> &mut (dyn AnyControl + 'static) + 'static) -> Getter<S> {
    Box::new(get)
}

/// Описание формы: где в структуре `S` лежат её контролы и какие у них обработчики.
///
/// Сами контролы — обычные поля твоей формы, поэтому в обработчике пишется
/// `self.label1.set_text("...")`. Обработчики — обычные методы вида
/// `fn button1_click(&mut self, ui: &mut Ui, e: &ClickArgs)`.
///
/// События возникают только от действий пользователя: изменение свойства из кода их не вызывает.
pub struct View<S> {
    pub(crate) nodes: Vec<Node<S>>,
    load: Vec<Handler<S, LoadArgs>>,
    click: Vec<(ControlId, Handler<S, ClickArgs>)>,
    text_changed: Vec<(ControlId, Handler<S, TextChangedArgs>)>,
    checked_changed: Vec<(ControlId, Handler<S, CheckedChangedArgs>)>,
}

impl<S> Default for View<S> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            load: Vec::new(),
            click: Vec::new(),
            text_changed: Vec::new(),
            checked_changed: Vec::new(),
        }
    }
}

impl<S: 'static> View<S> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Показывает на форме контрол из поля структуры: `view.add(|f| &mut f.button1)`.
    /// Порядок вызовов — это порядок отрисовки и перехода по Tab.
    pub fn add<T: ControlImpl>(
        &mut self,
        field: impl Fn(&mut S) -> &mut T + 'static,
    ) -> Handle<T> {
        self.insert(None, field)
    }

    /// То же внутри панели; координаты контрола считаются от её угла.
    pub fn add_to<P: Container, T: ControlImpl>(
        &mut self,
        panel: Handle<P>,
        field: impl Fn(&mut S) -> &mut T + 'static,
    ) -> Handle<T> {
        self.insert(Some(panel.id()), field)
    }

    fn insert<T: ControlImpl>(
        &mut self,
        parent: Option<ControlId>,
        field: impl Fn(&mut S) -> &mut T + 'static,
    ) -> Handle<T> {
        let handle = Handle::<T>::fresh();
        let get = getter(move |state| field(state) as &mut (dyn AnyControl + 'static));
        self.nodes.push(Node { id: handle.id(), parent, get });
        handle
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

    /// Подключает базовую форму: `part` достаёт её из производной, а `build` добавляет
    /// её контролы и подписки. Они идут раньше добавленных после этого вызова.
    pub fn include<B: 'static, R>(
        &mut self,
        part: fn(&mut S) -> &mut B,
        build: impl FnOnce(&mut View<B>) -> R,
    ) -> R {
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

        let mut inner = View::<B>::new();
        let result = build(&mut inner);
        self.nodes.extend(inner.nodes.into_iter().map(|node| {
            let get = node.get;
            Node {
                id: node.id,
                parent: node.parent,
                get: getter(move |state| get(part(state))),
            }
        }));
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
