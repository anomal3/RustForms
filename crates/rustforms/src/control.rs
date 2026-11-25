use std::any::Any;

use crate::{
    Anchor, Dock, Key, Modifiers, Painter, Point, Rect, TextSystem, app::Clipboard, view::Signal,
};

/// Свойства, общие для всех контролов.
#[derive(Clone, Debug)]
pub struct ControlBase {
    /// Положение относительно родителя.
    pub bounds: Rect,
    pub visible: bool,
    pub enabled: bool,
    pub tab_stop: bool,
    pub anchor: Anchor,
    /// Отличное от `None` отменяет якорь.
    pub dock: Dock,
}

impl ControlBase {
    pub fn new(width: f32, height: f32, tab_stop: bool) -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, width, height),
            visible: true,
            enabled: true,
            tab_stop,
            anchor: Anchor::default(),
            dock: Dock::None,
        }
    }
}

/// То, что контрол может сделать во время обработки ввода.
pub struct EventCx<'a> {
    pub(crate) text: &'a mut TextSystem,
    pub(crate) clipboard: &'a mut Clipboard,
    pub(crate) signals: &'a mut Vec<Signal>,
    pub(crate) size: (f32, f32),
}

impl EventCx<'_> {
    pub fn size(&self) -> (f32, f32) {
        self.size
    }

    pub fn text_system(&mut self) -> &mut TextSystem {
        self.text
    }

    pub fn click(&mut self) {
        self.signals.push(Signal::Click);
    }

    pub fn text_changed(&mut self, text: impl Into<String>) {
        self.signals.push(Signal::TextChanged(text.into()));
    }

    pub fn checked_changed(&mut self, checked: bool) {
        self.signals.push(Signal::CheckedChanged(checked));
    }

    pub fn clipboard_get(&mut self) -> Option<String> {
        self.clipboard.get()
    }

    pub fn clipboard_set(&mut self, text: &str) {
        self.clipboard.set(text);
    }
}

/// Связь контрола с родителем. Обычно выводится через `#[derive(Control)]`.
pub trait Control {
    fn parent(&self) -> Option<&dyn ControlImpl> {
        None
    }

    fn parent_mut(&mut self) -> Option<&mut dyn ControlImpl> {
        None
    }
}

/// Поведение контрола.
///
/// Свой контрол на основе готового помечает его поле `#[base]`: все не переопределённые
/// методы уходят туда. Контрол с нуля вместо этого переопределяет [`base`](Self::base)
/// и [`base_mut`](Self::base_mut).
///
/// Отличие от наследования в C#: если код родителя вызывает свой же метод,
/// переопределение во внешней структуре не срабатывает.
pub trait ControlImpl: Control + 'static {
    fn base(&self) -> &ControlBase {
        self.parent()
            .expect("контрол должен переопределить base() или parent()")
            .base()
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        self.parent_mut()
            .expect("контрол должен переопределить base_mut() или parent_mut()")
            .base_mut()
    }

    fn on_paint(&mut self, painter: &mut Painter) {
        if let Some(parent) = self.parent_mut() {
            parent.on_paint(painter);
        }
    }

    fn on_mouse_down(&mut self, at: Point, mods: Modifiers, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_mouse_down(at, mods, cx);
        }
    }

    /// Приходит, пока кнопка мыши зажата на этом контроле.
    fn on_mouse_drag(&mut self, at: Point, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_mouse_drag(at, cx);
        }
    }

    /// `inside` — отпустили ли кнопку над тем же контролом.
    fn on_mouse_up(&mut self, at: Point, inside: bool, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_mouse_up(at, inside, cx);
        }
    }

    fn on_key(&mut self, key: Key, mods: Modifiers, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_key(key, mods, cx);
        }
    }

    fn on_text_input(&mut self, text: &str, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_text_input(text, cx);
        }
    }
}

/// Позволяет получить из дерева контрол конкретного типа. Реализован для всех [`ControlImpl`].
pub trait AnyControl: ControlImpl {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: ControlImpl> AnyControl for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Общие свойства, доступные у любого контрола.
pub trait ControlExt: ControlImpl + Sized {
    fn at(mut self, x: f32, y: f32) -> Self {
        let bounds = &mut self.base_mut().bounds;
        bounds.x = x;
        bounds.y = y;
        self
    }

    fn size(mut self, width: f32, height: f32) -> Self {
        let bounds = &mut self.base_mut().bounds;
        bounds.width = width;
        bounds.height = height;
        self
    }

    fn anchor(mut self, anchor: Anchor) -> Self {
        self.base_mut().anchor = anchor;
        self
    }

    fn dock(mut self, dock: Dock) -> Self {
        self.base_mut().dock = dock;
        self
    }

    fn set_anchor(&mut self, anchor: Anchor) {
        self.base_mut().anchor = anchor;
    }

    fn set_dock(&mut self, dock: Dock) {
        self.base_mut().dock = dock;
    }

    fn bounds(&self) -> Rect {
        self.base().bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base_mut().bounds = bounds;
    }

    fn visible(&self) -> bool {
        self.base().visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.base_mut().visible = visible;
    }

    fn enabled(&self) -> bool {
        self.base().enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.base_mut().enabled = enabled;
    }
}

impl<T: ControlImpl> ControlExt for T {}
