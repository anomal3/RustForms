use std::{any::Any, cell::Cell, cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    AnyControl, CheckBox, ComboBox, Container, ControlId, ControlImpl, DataGridView, Handle,
    ListBox, MenuStrip, Modifiers, NumericUpDown, RadioButton, TabControl, TextBox, Timer, TrackBar,
    TreeView, Ui, DateTimePicker, ToolStrip,
};

/// [`TextBox`] и всё, что от него «унаследовано».
pub trait RaisesTextChanged {}
impl RaisesTextChanged for TextBox {}
impl<T: Deref> RaisesTextChanged for T where T::Target: RaisesTextChanged {}

/// [`CheckBox`], [`RadioButton`] и всё, что от них «унаследовано».
pub trait RaisesCheckedChanged {}
impl RaisesCheckedChanged for CheckBox {}
impl RaisesCheckedChanged for RadioButton {}
impl<T: Deref> RaisesCheckedChanged for T where T::Target: RaisesCheckedChanged {}

/// [`DataGridView`] и всё, что от него «унаследовано».
pub trait RaisesCellChanged {}
impl RaisesCellChanged for DataGridView {}
impl<T: Deref> RaisesCellChanged for T where T::Target: RaisesCellChanged {}

/// [`NumericUpDown`], [`TrackBar`] и всё, что от них «унаследовано».
pub trait RaisesValueChanged {}
impl RaisesValueChanged for NumericUpDown {}
impl RaisesValueChanged for TrackBar {}
impl<T: Deref> RaisesValueChanged for T where T::Target: RaisesValueChanged {}

/// [`ListBox`], [`ComboBox`], [`DataGridView`], [`TreeView`] и всё, что от них «унаследовано».
pub trait RaisesSelectedChanged {}
impl RaisesSelectedChanged for ListBox {}
impl RaisesSelectedChanged for ComboBox {}
impl RaisesSelectedChanged for DataGridView {}
impl RaisesSelectedChanged for TabControl {}
impl RaisesSelectedChanged for DateTimePicker {}
impl RaisesSelectedChanged for TreeView {}
impl<T: Deref> RaisesSelectedChanged for T where T::Target: RaisesSelectedChanged {}

/// [`MenuStrip`] и всё, что от него «унаследовано».
pub trait RaisesItemClicked {}
impl RaisesItemClicked for MenuStrip {}
impl RaisesItemClicked for ToolStrip {}
impl<T: Deref> RaisesItemClicked for T where T::Target: RaisesItemClicked {}

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

#[derive(Clone, Debug)]
pub struct SelectedChangedArgs {
    pub sender: ControlId,
    pub index: Option<usize>,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct MenuItemArgs {
    pub sender: ControlId,
    /// Путь пункта, например `"Файл/Выход"`.
    pub item: String,
}

/// Мышь: координаты от левого верхнего угла контрола.
#[derive(Clone, Copy, Debug)]
pub struct MouseArgs {
    pub sender: ControlId,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct KeyArgs {
    pub sender: ControlId,
    /// Клавиша по её месту на клавиатуре, не зависит от раскладки: `"A"`, `"1"`,
    /// `"Enter"`, `"Escape"`, `"ArrowUp"`, `"F5"`.
    pub key: String,
    pub modifiers: Modifiers,
}

/// Получение и потеря фокуса.
#[derive(Clone, Copy, Debug)]
pub struct FocusArgs {
    pub sender: ControlId,
}

#[derive(Clone, Copy, Debug)]
pub struct ValueChangedArgs {
    pub sender: ControlId,
    pub value: i32,
}

/// Пользователь изменил ячейку таблицы.
#[derive(Clone, Debug)]
pub struct CellChangedArgs {
    pub sender: ControlId,
    pub row: usize,
    pub column: usize,
    pub text: String,
}

/// Сработал таймер.
#[derive(Clone, Copy, Debug)]
pub struct TickArgs {
    pub sender: ControlId,
}

#[derive(Clone, Copy, Debug)]
pub struct LoadArgs;

#[derive(Clone, Copy, Debug)]
pub struct ResizeArgs {
    pub width: f32,
    pub height: f32,
}

/// Окно собираются закрыть.
#[derive(Debug, Default)]
pub struct ClosingArgs {
    cancelled: Cell<bool>,
}

impl ClosingArgs {
    /// Оставить окно открытым.
    pub fn cancel(&self) {
        self.cancelled.set(true);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventKind {
    Load,
    Closing,
    Resize,
    Click,
    DoubleClick,
    TextChanged,
    CheckedChanged,
    SelectedChanged,
    ItemClicked,
    ValueChanged,
    Tick,
    CellChanged,
    ContextItemClicked,
    MouseEnter,
    MouseLeave,
    MouseDown,
    MouseUp,
    KeyDown,
    KeyUp,
    GotFocus,
    LostFocus,
}

/// Что произошло с контролом; из этого строятся аргументы обработчиков.
#[derive(Clone, Debug)]
pub(crate) enum Signal {
    Click,
    TextChanged(String),
    CheckedChanged(bool),
    SelectedChanged(Option<usize>, String),
    ItemClicked(String),
    ValueChanged(i32),
    CellChanged(usize, usize, String),
}

type Handler<S> = Box<dyn FnMut(&mut S, &mut Ui, &dyn Any)>;
type Getter<S> = Box<dyn Fn(&mut S) -> &mut (dyn AnyControl + 'static)>;

pub(crate) struct Node<S> {
    pub id: ControlId,
    pub parent: Option<ControlId>,
    pub get: Getter<S>,
}

pub(crate) struct TimerSlot<S> {
    pub id: ControlId,
    pub get: Box<dyn Fn(&mut S) -> &mut Timer>,
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
/// События изменения (`text_changed` и подобные) возникают только от действий
/// пользователя: изменение свойства из кода их не вызывает.
pub struct View<S> {
    pub(crate) nodes: Vec<Node<S>>,
    pub(crate) timers: Vec<TimerSlot<S>>,
    /// `None` вместо номера — событие самой формы.
    handlers: Vec<(Option<ControlId>, EventKind, Handler<S>)>,
}

impl<S> Default for View<S> {
    fn default() -> Self {
        Self { nodes: Vec::new(), timers: Vec::new(), handlers: Vec::new() }
    }
}

macro_rules! control_events {
    ($($(#[$doc:meta])* $method:ident $(<$bound:ident>)? => $kind:ident, $args:ty;)*) => {
        $(
            $(#[$doc])*
            pub fn $method<T $(: $bound)?>(
                &mut self,
                control: Handle<T>,
                handler: impl FnMut(&mut S, &mut Ui, &$args) + 'static,
            ) {
                self.subscribe(Some(control.id()), EventKind::$kind, handler);
            }
        )*
    };
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

    fn subscribe<A: 'static>(
        &mut self,
        sender: Option<ControlId>,
        kind: EventKind,
        mut handler: impl FnMut(&mut S, &mut Ui, &A) + 'static,
    ) {
        let erased = Box::new(move |state: &mut S, ui: &mut Ui, args: &dyn Any| {
            handler(state, ui, args.downcast_ref().expect("вид события задаёт тип аргументов"));
        });
        self.handlers.push((sender, kind, erased));
    }

    /// Подключает таймер из поля структуры: `view.add_timer(|f| &mut f.timer1)`.
    pub fn add_timer(&mut self, field: impl Fn(&mut S) -> &mut Timer + 'static) -> Handle<Timer> {
        let handle = Handle::<Timer>::fresh();
        self.timers.push(TimerSlot { id: handle.id(), get: Box::new(field) });
        handle
    }

    pub fn on_tick(
        &mut self,
        timer: Handle<Timer>,
        handler: impl FnMut(&mut S, &mut Ui, &TickArgs) + 'static,
    ) {
        self.subscribe(Some(timer.id()), EventKind::Tick, handler);
    }

    pub fn on_load(&mut self, handler: impl FnMut(&mut S, &mut Ui, &LoadArgs) + 'static) {
        self.subscribe(None, EventKind::Load, handler);
    }

    /// Перед закрытием окна; `e.cancel()` оставляет его открытым.
    pub fn on_closing(&mut self, handler: impl FnMut(&mut S, &mut Ui, &ClosingArgs) + 'static) {
        self.subscribe(None, EventKind::Closing, handler);
    }

    pub fn on_resize(&mut self, handler: impl FnMut(&mut S, &mut Ui, &ResizeArgs) + 'static) {
        self.subscribe(None, EventKind::Resize, handler);
    }

    control_events! {
        on_click => Click, ClickArgs;
        on_double_click => DoubleClick, MouseArgs;
        on_text_changed<RaisesTextChanged> => TextChanged, TextChangedArgs;
        on_checked_changed<RaisesCheckedChanged> => CheckedChanged, CheckedChangedArgs;
        on_selected_changed<RaisesSelectedChanged> => SelectedChanged, SelectedChangedArgs;
        on_item_clicked<RaisesItemClicked> => ItemClicked, MenuItemArgs;
        on_value_changed<RaisesValueChanged> => ValueChanged, ValueChangedArgs;
        on_cell_changed<RaisesCellChanged> => CellChanged, CellChangedArgs;
        /// Выбран пункт меню по правой кнопке.
        on_context_item_clicked => ContextItemClicked, MenuItemArgs;
        on_mouse_enter => MouseEnter, MouseArgs;
        on_mouse_leave => MouseLeave, MouseArgs;
        on_mouse_down => MouseDown, MouseArgs;
        on_mouse_up => MouseUp, MouseArgs;
        /// Приходит контролу с фокусом.
        on_key_down => KeyDown, KeyArgs;
        on_key_up => KeyUp, KeyArgs;
        on_got_focus => GotFocus, FocusArgs;
        on_lost_focus => LostFocus, FocusArgs;
    }

    /// Подключает базовую форму: `part` достаёт её из производной, а `build` добавляет
    /// её контролы и подписки. Они идут раньше добавленных после этого вызова.
    pub fn include<B: 'static, R>(
        &mut self,
        part: fn(&mut S) -> &mut B,
        build: impl FnOnce(&mut View<B>) -> R,
    ) -> R {
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
        self.timers.extend(inner.timers.into_iter().map(|slot| {
            let get = slot.get;
            TimerSlot { id: slot.id, get: Box::new(move |state: &mut S| get(part(state))) }
        }));
        self.handlers.extend(inner.handlers.into_iter().map(|(sender, kind, handler)| {
            let handler = Rc::new(RefCell::new(handler));
            let adapted: Handler<S> = Box::new(move |state, ui, args| {
                (handler.borrow_mut())(part(state), ui, args)
            });
            (sender, kind, adapted)
        }));
        result
    }

    pub(crate) fn emit(
        &mut self,
        sender: Option<ControlId>,
        kind: EventKind,
        args: &dyn Any,
        state: &mut S,
        ui: &mut Ui,
    ) {
        for (id, of, handler) in &mut self.handlers {
            if *id == sender && *of == kind {
                handler(state, ui, args);
            }
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        sender: ControlId,
        signal: Signal,
        state: &mut S,
        ui: &mut Ui,
    ) {
        let from = Some(sender);
        match signal {
            Signal::Click => {
                self.emit(from, EventKind::Click, &ClickArgs { sender }, state, ui)
            }
            Signal::TextChanged(text) => {
                let args = TextChangedArgs { sender, text };
                self.emit(from, EventKind::TextChanged, &args, state, ui)
            }
            Signal::CheckedChanged(checked) => {
                let args = CheckedChangedArgs { sender, checked };
                self.emit(from, EventKind::CheckedChanged, &args, state, ui)
            }
            Signal::SelectedChanged(index, text) => {
                let args = SelectedChangedArgs { sender, index, text };
                self.emit(from, EventKind::SelectedChanged, &args, state, ui)
            }
            Signal::CellChanged(row, column, text) => {
                let args = CellChangedArgs { sender, row, column, text };
                self.emit(from, EventKind::CellChanged, &args, state, ui)
            }
            Signal::ValueChanged(value) => {
                let args = ValueChangedArgs { sender, value };
                self.emit(from, EventKind::ValueChanged, &args, state, ui)
            }
            Signal::ItemClicked(item) => {
                let args = MenuItemArgs { sender, item };
                self.emit(from, EventKind::ItemClicked, &args, state, ui)
            }
        }
    }
}
