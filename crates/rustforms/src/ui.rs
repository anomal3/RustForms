use std::{
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{Color, Panel, forms::PendingForm};

/// Контролы, внутрь которых можно класть другие: [`Panel`] и «унаследованные» от неё.
pub trait Container {}
impl Container for Panel {}
impl<T: Deref> Container for T where T::Target: Container {}

/// Номер контрола на форме; по нему различаются отправители событий (`e.sender`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlId(usize);

/// Типизированный номер контрола: им подписываются на события, сравнивают `e.sender`.
pub struct Handle<T> {
    id: ControlId,
    kind: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    /// Номера уникальны на всю программу, чтобы базовые формы подключались без пересчёта.
    pub(crate) fn fresh() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = ControlId(NEXT.fetch_add(1, Ordering::Relaxed));
        Self { id, kind: PhantomData }
    }

    pub fn id(&self) -> ControlId {
        self.id
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

/// Окно формы.
pub struct Ui {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub back_color: Color,
    pub(crate) close_requested: bool,
    /// Закрыта кнопкой «ОК».
    pub(crate) accepted: bool,
    /// Формы, которые просили открыть.
    pub(crate) pending: Vec<PendingForm>,
    /// Окно формы, если оно есть: владелец системных диалогов.
    pub(crate) owner: Option<Rc<winit::window::Window>>,
}

impl Ui {
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            back_color: Color::FORM,
            close_requested: false,
            accepted: false,
            pending: Vec::new(),
            owner: None,
        }
    }

    pub fn close(&mut self) {
        self.close_requested = true;
    }

    /// Закрыть как «ОК»: открывший диалог получит `accepted = true`.
    pub fn accept(&mut self) {
        self.accepted = true;
        self.close_requested = true;
    }
}
