use std::any::Any;

use crate::{
    Anchor, Color, Dock, Key, TextAlign, Modifiers, Painter, Point, Rect, TextSystem, app::Clipboard, view::Signal,
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
    /// `None` — цвет по умолчанию для этого контрола.
    pub fore_color: Option<Color>,
    pub back_color: Option<Color>,
    /// `None` — обычный размер шрифта.
    pub font_size: Option<f32>,
    pub bold: bool,
    /// Подсказка при наведении; пустая не показывается.
    pub tooltip: String,
    /// Порядок перехода по Tab; при равенстве — порядок добавления.
    pub tab_index: i32,
    /// `None` — выравнивание по умолчанию (у кнопки по центру, у надписи слева).
    pub text_align: Option<TextAlign>,
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
            fore_color: None,
            back_color: None,
            font_size: None,
            bold: false,
            tooltip: String::new(),
            tab_index: 0,
            text_align: None,
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

    pub fn selected_changed(&mut self, index: Option<usize>, text: impl Into<String>) {
        self.signals.push(Signal::SelectedChanged(index, text.into()));
    }

    pub fn item_clicked(&mut self, item: impl Into<String>) {
        self.signals.push(Signal::ItemClicked(item.into()));
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

    /// Мышь движется над контролом.
    fn on_mouse_move(&mut self, at: Point, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_mouse_move(at, cx);
        }
    }

    /// Сдвиг всплывающего окошка от левого края контрола.
    fn popup_offset(&self) -> f32 {
        self.parent().map_or(0.0, |p| p.popup_offset())
    }

    /// Колесо мыши; положительное значение — вверх, в строках.
    fn on_wheel(&mut self, lines: f32, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_wheel(lines, cx);
        }
    }

    /// `Some`, пока открыто всплывающее окошко: оно рисуется поверх всей формы под контролом
    /// (или над ним, если внизу нет места) и первым получает мышь.
    fn popup_size(&self) -> Option<(f32, f32)> {
        self.parent().and_then(|p| p.popup_size())
    }

    fn close_popup(&mut self) {
        if let Some(parent) = self.parent_mut() {
            parent.close_popup();
        }
    }

    fn on_paint_popup(&mut self, painter: &mut Painter) {
        if let Some(parent) = self.parent_mut() {
            parent.on_paint_popup(painter);
        }
    }

    fn on_popup_mouse_move(&mut self, at: Point, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_popup_mouse_move(at, cx);
        }
    }

    fn on_popup_mouse_down(&mut self, at: Point, cx: &mut EventCx) {
        if let Some(parent) = self.parent_mut() {
            parent.on_popup_mouse_down(at, cx);
        }
    }

    /// Отмеченным среди соседей может быть только один (радиокнопка).
    fn exclusive(&self) -> bool {
        self.parent().is_some_and(|p| p.exclusive())
    }

    /// Снимает отметку; возвращает, была ли она.
    fn uncheck(&mut self) -> bool {
        self.parent_mut().is_some_and(|p| p.uncheck())
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

    fn fore_color(mut self, color: Color) -> Self {
        self.base_mut().fore_color = Some(color);
        self
    }

    fn back_color(mut self, color: Color) -> Self {
        self.base_mut().back_color = Some(color);
        self
    }

    fn set_fore_color(&mut self, color: Color) {
        self.base_mut().fore_color = Some(color);
    }

    fn set_back_color(&mut self, color: Color) {
        self.base_mut().back_color = Some(color);
    }

    fn set_font_size(&mut self, size: f32) {
        self.base_mut().font_size = Some(size);
    }

    fn set_bold(&mut self, bold: bool) {
        self.base_mut().bold = bold;
    }

    fn set_tooltip(&mut self, text: impl Into<String>) {
        self.base_mut().tooltip = text.into();
    }

    fn set_tab_index(&mut self, index: i32) {
        self.base_mut().tab_index = index;
    }

    fn set_text_align(&mut self, align: TextAlign) {
        self.base_mut().text_align = Some(align);
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
