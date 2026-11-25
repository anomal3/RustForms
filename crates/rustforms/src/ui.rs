use std::{
    marker::PhantomData,
    ops::{Deref, Index, IndexMut},
};

use crate::{AnyControl, Color, ControlImpl, Panel, Point, Rect};

/// Контролы, внутрь которых можно класть другие: [`Panel`] и «унаследованные» от неё.
pub trait Container {}
impl Container for Panel {}
impl<T: Deref> Container for T where T::Target: Container {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlId(pub(crate) usize);

/// Типизированная ссылка на контрол формы: `ui[self.label1].set_text("...")`.
pub struct Handle<T> {
    id: ControlId,
    kind: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
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

pub(crate) struct Node {
    pub control: Box<dyn AnyControl>,
    pub parent: Option<ControlId>,
}

/// Окно формы и дерево его контролов.
pub struct Ui {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub back_color: Color,
    pub(crate) nodes: Vec<Node>,
    pub(crate) close_requested: bool,
}

impl Ui {
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            back_color: Color::FORM,
            nodes: Vec::new(),
            close_requested: false,
        }
    }

    pub fn add<T: ControlImpl>(&mut self, control: T) -> Handle<T> {
        self.insert(control, None)
    }

    /// Кладёт контрол внутрь панели; его координаты считаются от угла панели.
    pub fn add_to<P: Container, T: ControlImpl>(
        &mut self,
        panel: Handle<P>,
        control: T,
    ) -> Handle<T> {
        self.insert(control, Some(panel.id()))
    }

    fn insert<T: ControlImpl>(&mut self, control: T, parent: Option<ControlId>) -> Handle<T> {
        let id = ControlId(self.nodes.len());
        self.nodes.push(Node { control: Box::new(control), parent });
        Handle { id, kind: PhantomData }
    }

    pub fn close(&mut self) {
        self.close_requested = true;
    }

    /// Положение контрола в координатах окна и видимая часть (обрезка родителями).
    pub(crate) fn placement(&self, id: ControlId) -> (Rect, Rect) {
        let node = &self.nodes[id.0];
        let own = node.control.base().bounds;
        let (origin, clip) = match node.parent {
            Some(parent) => {
                let (rect, clip) = self.placement(parent);
                (Point::new(rect.x, rect.y), clip)
            }
            None => (Point::default(), Rect::new(0.0, 0.0, f32::MAX, f32::MAX)),
        };
        let rect = Rect::new(origin.x + own.x, origin.y + own.y, own.width, own.height);
        (rect, rect.intersect(&clip))
    }

    /// Виден ли контрол с учётом всех родителей.
    pub(crate) fn is_shown(&self, id: ControlId) -> bool {
        let node = &self.nodes[id.0];
        node.control.base().visible && node.parent.is_none_or(|p| self.is_shown(p))
    }

    pub(crate) fn is_active(&self, id: ControlId) -> bool {
        let node = &self.nodes[id.0];
        node.control.base().enabled && node.parent.is_none_or(|p| self.is_active(p))
    }

    pub(crate) fn hit_test(&self, at: Point) -> Option<ControlId> {
        (0..self.nodes.len())
            .rev()
            .map(ControlId)
            .find(|&id| self.is_shown(id) && self.placement(id).1.contains(at))
    }

    pub(crate) fn can_focus(&self, id: ControlId) -> bool {
        self.nodes[id.0].control.base().tab_stop && self.is_shown(id) && self.is_active(id)
    }
}

impl<T: ControlImpl> Index<Handle<T>> for Ui {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &T {
        // Через `*Box`, иначе `as_any` сработает для самого Box.
        (*self.nodes[handle.id.0].control)
            .as_any()
            .downcast_ref()
            .expect("Handle от другой формы")
    }
}

impl<T: ControlImpl> IndexMut<Handle<T>> for Ui {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut T {
        (*self.nodes[handle.id.0].control)
            .as_any_mut()
            .downcast_mut()
            .expect("Handle от другой формы")
    }
}
