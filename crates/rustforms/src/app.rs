use tiny_skia::Pixmap;

use std::time::{Duration, Instant};

use crate::{
    Align, ClosingArgs, Color, FocusArgs, FontStyle, KeyArgs, LoadArgs, MouseArgs, ResizeArgs,
    paint::FONT_SIZE, view::EventKind, Dock, AnyControl, ControlId, EventCx, Key, Modifiers, Painter, Point, Rect, TextSystem, Ui, View,
    VisualState, layout, view::Signal,
};

const TOOLTIP_DELAY: Duration = Duration::from_millis(600);
const DOUBLE_CLICK: Duration = Duration::from_millis(500);

/// Системный буфер обмена; без него (тесты, нет доступа) — внутренний.
pub(crate) struct Clipboard {
    system: Option<arboard::Clipboard>,
    local: String,
}

impl Clipboard {
    pub(crate) fn get(&mut self) -> Option<String> {
        match &mut self.system {
            Some(system) => system.get_text().ok(),
            None => Some(self.local.clone()),
        }
    }

    pub(crate) fn set(&mut self, text: &str) {
        match &mut self.system {
            Some(system) => {
                // Занятый другим приложением буфер не должен ронять программу.
                let _ = system.set_text(text);
            }
            None => self.local = text.to_owned(),
        }
    }
}

/// Окно, форма с её контролами (`state`) и описание формы. Не зависит от настоящего окна,
/// поэтому ввод можно подавать из тестов, а кадр рисовать в память.
///
/// Внутри контролы обозначаются номером по порядку в [`View`].
pub struct Application<S> {
    pub ui: Ui,
    pub state: S,
    view: View<S>,
    text: TextSystem,
    clipboard: Clipboard,
    focus: Option<usize>,
    hover: Option<usize>,
    capture: Option<usize>,
    mouse: Point,
    loaded: bool,
    /// Когда мышь пришла на нынешний контрол: от этого отсчитывается подсказка.
    hover_since: Option<Instant>,
    tooltip: Option<(usize, Point)>,
    last_click: Option<(usize, Instant, Point)>,
    /// Размер окна, о котором форме уже сообщили.
    size: (f32, f32),
    /// Размер родителя каждого контрола при прошлом размещении: якоря считают от разницы.
    parent_sizes: Vec<(f32, f32)>,
}

impl<S: 'static> Application<S> {
    pub fn new(ui: Ui, state: S, view: View<S>) -> Self {
        let size = (ui.width, ui.height);
        let mut app = Self {
            size,
            hover_since: None,
            tooltip: None,
            last_click: None,
            ui,
            state,
            view,
            parent_sizes: Vec::new(),
            text: TextSystem::new(),
            clipboard: Clipboard { system: None, local: String::new() },
            focus: None,
            hover: None,
            capture: None,
            mouse: Point::default(),
            loaded: false,
        };
        app.reset_layout();
        app
    }

    fn parent_size(&mut self, index: usize) -> (f32, f32) {
        match self.parent(index) {
            Some(parent) => {
                let bounds = self.control(parent).base().bounds;
                (bounds.width, bounds.height)
            }
            None => (self.ui.width, self.ui.height),
        }
    }

    /// Текущие размеры считаются исходными для якорей.
    fn reset_layout(&mut self) {
        self.parent_sizes =
            (0..self.view.nodes.len()).map(|i| self.parent_size(i)).collect();
        self.layout();
    }

    /// Применяет якоря и Dock. Вызывается перед вводом и отрисовкой.
    pub fn layout(&mut self) {
        // Родители стоят в списке раньше детей, поэтому хватает одного прохода по уровням.
        let mut parents: Vec<Option<usize>> = vec![None];
        parents.extend((0..self.view.nodes.len()).map(Some));
        for parent in parents {
            let children: Vec<usize> =
                (0..self.view.nodes.len()).filter(|&i| self.parent(i) == parent).collect();
            let Some(&first) = children.first() else { continue };
            let size = self.parent_size(first);
            let mut docked = Vec::new();
            for &index in &children {
                let (old_w, old_h) = std::mem::replace(&mut self.parent_sizes[index], size);
                let base = self.control(index).base_mut();
                if base.dock == Dock::None {
                    base.bounds =
                        layout::anchored(base.bounds, base.anchor, size.0 - old_w, size.1 - old_h);
                } else if base.visible {
                    docked.push((index, (base.dock, base.bounds)));
                }
            }
            let mut rects: Vec<_> = docked.iter().map(|d| d.1).collect();
            layout::dock(size, &mut rects);
            for ((index, _), (_, rect)) in docked.into_iter().zip(rects) {
                self.control(index).base_mut().bounds = rect;
            }
        }
    }

    /// Заменяет форму целиком, не загружая шрифты заново.
    pub fn replace(&mut self, ui: Ui, state: S, view: View<S>) {
        (self.ui, self.state, self.view) = (ui, state, view);
        (self.focus, self.hover, self.capture) = (None, None, None);
        self.loaded = false;
        self.size = (self.ui.width, self.ui.height);
        (self.tooltip, self.hover_since, self.last_click) = (None, None, None);
        self.reset_layout();
    }

    pub(crate) fn use_system_clipboard(&mut self) {
        self.clipboard.system = arboard::Clipboard::new().ok();
    }

    /// Вызывает `on_load` один раз.
    pub fn load(&mut self) {
        if !self.loaded {
            self.loaded = true;
            self.view.emit(None, EventKind::Load, &LoadArgs, &mut self.state, &mut self.ui);
            self.layout();
        }
    }

    /// Спрашивает форму, можно ли закрыться (событие `closing`).
    pub fn try_close(&mut self) -> bool {
        let args = ClosingArgs::default();
        self.view.emit(None, EventKind::Closing, &args, &mut self.state, &mut self.ui);
        if args.is_cancelled() {
            self.ui.close_requested = false;
            self.ui.accepted = false;
            self.layout();
        }
        !args.is_cancelled()
    }

    /// Окно изменило размер (`ui.width`/`ui.height` уже новые).
    pub fn resized(&mut self) {
        self.layout();
        let size = (self.ui.width, self.ui.height);
        if std::mem::replace(&mut self.size, size) != size {
            let args = ResizeArgs { width: size.0, height: size.1 };
            self.view.emit(None, EventKind::Resize, &args, &mut self.state, &mut self.ui);
            self.layout();
        }
    }

    fn emit<A: 'static>(&mut self, index: usize, kind: EventKind, args: impl FnOnce(ControlId) -> A) {
        let id = self.view.nodes[index].id;
        let args = args(id);
        self.view.emit(Some(id), kind, &args, &mut self.state, &mut self.ui);
        self.layout();
    }

    fn emit_mouse(&mut self, index: usize, kind: EventKind) {
        let (rect, _) = self.placement(index);
        let (x, y) = (self.mouse.x - rect.x, self.mouse.y - rect.y);
        self.emit(index, kind, |sender| MouseArgs { sender, x, y });
    }

    fn set_hover(&mut self, hover: Option<usize>) {
        if self.hover == hover {
            return;
        }
        let old = std::mem::replace(&mut self.hover, hover);
        self.tooltip = None;
        self.hover_since = hover.map(|_| Instant::now());
        if let Some(old) = old {
            self.emit_mouse(old, EventKind::MouseLeave);
        }
        if let Some(new) = hover {
            self.emit_mouse(new, EventKind::MouseEnter);
        }
    }

    fn set_focus(&mut self, focus: Option<usize>) {
        if self.focus == focus {
            return;
        }
        let old = std::mem::replace(&mut self.focus, focus);
        if let Some(old) = old {
            self.emit(old, EventKind::LostFocus, |sender| FocusArgs { sender });
        }
        if let Some(new) = focus {
            self.emit(new, EventKind::GotFocus, |sender| FocusArgs { sender });
        }
    }

    /// Когда пора показать подсказку; `None` — ждать нечего.
    pub fn tooltip_deadline(&mut self) -> Option<Instant> {
        let index = self.hover?;
        if self.tooltip.is_some() || self.control(index).base().tooltip.is_empty() {
            return None;
        }
        Some(self.hover_since? + TOOLTIP_DELAY)
    }

    /// Показывает подсказку, если её время пришло; возвращает, нужна ли перерисовка.
    pub fn tick(&mut self, now: Instant) -> bool {
        let due = self.tooltip_deadline().is_some_and(|at| now >= at);
        if due {
            self.tooltip = self.hover.map(|index| (index, self.mouse));
        }
        due
    }

    /// Клавиша по месту на клавиатуре — для событий `key_down` и `key_up`.
    pub fn raw_key(&mut self, key: &str, modifiers: Modifiers, pressed: bool) {
        if let Some(index) = self.focus_target() {
            let kind = if pressed { EventKind::KeyDown } else { EventKind::KeyUp };
            let key = key.to_owned();
            self.emit(index, kind, |sender| KeyArgs { sender, key, modifiers });
        }
    }

    pub fn focused(&self) -> Option<ControlId> {
        self.focus.map(|i| self.view.nodes[i].id)
    }

    pub fn close_requested(&self) -> bool {
        self.ui.close_requested
    }

    fn control(&mut self, index: usize) -> &mut (dyn AnyControl + 'static) {
        (self.view.nodes[index].get)(&mut self.state)
    }

    fn parent(&self, index: usize) -> Option<usize> {
        let parent = self.view.nodes[index].parent?;
        self.view.nodes.iter().position(|n| n.id == parent)
    }

    /// Положение контрола в координатах окна и видимая часть (обрезка родителями).
    fn placement(&mut self, index: usize) -> (Rect, Rect) {
        let own = self.control(index).base().bounds;
        let (origin, clip) = match self.parent(index) {
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
    fn is_shown(&mut self, index: usize) -> bool {
        self.control(index).base().visible
            && self.parent(index).is_none_or(|p| self.is_shown(p))
    }

    fn is_active(&mut self, index: usize) -> bool {
        self.control(index).base().enabled
            && self.parent(index).is_none_or(|p| self.is_active(p))
    }

    fn hit_test(&mut self, at: Point) -> Option<usize> {
        (0..self.view.nodes.len())
            .rev()
            .find(|&i| self.is_shown(i) && self.placement(i).1.contains(at))
    }

    fn can_focus(&mut self, index: usize) -> bool {
        self.control(index).base().tab_stop && self.is_shown(index) && self.is_active(index)
    }

    fn with_control(
        &mut self,
        index: usize,
        action: impl FnOnce(&mut dyn AnyControl, Point, &mut EventCx),
    ) {
        let (rect, _) = self.placement(index);
        let local = Point::new(self.mouse.x - rect.x, self.mouse.y - rect.y);
        let local = if self.control(index).popup_size().is_some() {
            let popup = self.popup_rect(index).unwrap_or(rect);
            if popup.contains(self.mouse) {
                Point::new(self.mouse.x - popup.x, self.mouse.y - popup.y)
            } else {
                local
            }
        } else {
            local
        };
        let size = match self.popup_rect(index) {
            Some(popup) if popup.contains(self.mouse) => (popup.width, popup.height),
            _ => (rect.width, rect.height),
        };
        self.text.style = self.font_of(index);
        let node = &self.view.nodes[index];
        let id = node.id;
        let mut signals: Vec<Signal> = Vec::new();
        let mut cx = EventCx {
            text: &mut self.text,
            clipboard: &mut self.clipboard,
            signals: &mut signals,
            size,
        };
        action((node.get)(&mut self.state), local, &mut cx);
        // Отметка радиокнопки снимает отметки соседних — до вызова обработчиков.
        let mut cleared = Vec::new();
        if signals.iter().any(|s| matches!(s, Signal::CheckedChanged(true)))
            && self.control(index).exclusive()
        {
            let parent = self.view.nodes[index].parent;
            for other in 0..self.view.nodes.len() {
                if other != index
                    && self.view.nodes[other].parent == parent
                    && self.control(other).exclusive()
                    && self.control(other).uncheck()
                {
                    cleared.push(self.view.nodes[other].id);
                }
            }
        }
        for other in cleared {
            let signal = Signal::CheckedChanged(false);
            self.view.dispatch(other, signal, &mut self.state, &mut self.ui);
        }
        for signal in signals {
            self.view.dispatch(id, signal, &mut self.state, &mut self.ui);
        }
        // Обработчик мог изменить размеры или видимость.
        self.layout();
    }

    fn font_of(&mut self, index: usize) -> FontStyle {
        let base = self.control(index).base();
        FontStyle { size: base.font_size.unwrap_or(FONT_SIZE), bold: base.bold }
    }

    /// Рисует контрол или его окошко с его цветами и шрифтом.
    #[allow(clippy::too_many_arguments)]
    fn paint_control(
        &mut self,
        index: usize,
        pixmap: &mut Pixmap,
        scale: f32,
        frame: Rect,
        clip: Rect,
        state: VisualState,
        popup: bool,
    ) {
        self.text.style = self.font_of(index);
        let base = self.control(index).base();
        let (fore, back, align) = (base.fore_color, base.back_color, base.text_align);
        let mut painter =
            Painter { pixmap, text: &mut self.text, scale, frame, clip, state, fore, back, align };
        let control = (self.view.nodes[index].get)(&mut self.state);
        if popup {
            control.on_paint_popup(&mut painter);
        } else {
            control.on_paint(&mut painter);
        }
        self.text.style = FontStyle::default();
    }

    fn paint_tooltip(&mut self, pixmap: &mut Pixmap, scale: f32, window: Rect) {
        let Some((index, at)) = self.tooltip else { return };
        let text = self.control(index).base().tooltip.clone();
        let width = self.text.width(&text).ceil() + 14.0;
        let x = at.x.min(window.width - width).max(0.0);
        let below = at.y + 20.0;
        let y = if below + 26.0 > window.height { at.y - 30.0 } else { below };
        let frame = Rect::new(x, y, width, 24.0);
        let mut painter = Painter {
            pixmap,
            text: &mut self.text,
            scale,
            frame,
            clip: frame.intersect(&window),
            state: VisualState { enabled: true, ..VisualState::default() },
            fore: None,
            back: None,
            align: None,
        };
        let area = painter.area();
        painter.fill_rect(area, Color::rgb(255, 255, 225));
        painter.stroke_rect(area, Color::BORDER, 1.0);
        painter.text(&text, area, Color::TEXT, Align::Center, 0.0);
    }

    /// Контрол с открытым всплывающим окошком.
    fn popup_owner(&mut self) -> Option<usize> {
        (0..self.view.nodes.len())
            .find(|&i| self.control(i).popup_size().is_some() && self.is_shown(i))
    }

    /// Где в окне рисуется всплывающее окошко контрола.
    fn popup_rect(&mut self, index: usize) -> Option<Rect> {
        let (width, height) = self.control(index).popup_size()?;
        let (frame, _) = self.placement(index);
        let below = frame.y + frame.height;
        let y = if below + height > self.ui.height && frame.y >= height {
            frame.y - height
        } else {
            below
        };
        let offset = self.control(index).popup_offset();
        // Окошко не выходит за правый край.
        let x = (frame.x + offset).min(self.ui.width - width).max(0.0);
        Some(Rect::new(x, y, width, height))
    }

    pub fn close_popups(&mut self) {
        for index in 0..self.view.nodes.len() {
            self.control(index).close_popup();
        }
    }

    /// Владелец окошка, если мышь сейчас над ним.
    fn popup_under_mouse(&mut self) -> Option<usize> {
        let owner = self.popup_owner()?;
        self.popup_rect(owner)?.contains(self.mouse).then_some(owner)
    }

    pub fn wheel(&mut self, lines: f32) {
        self.layout();
        let target = match self.popup_under_mouse() {
            Some(owner) => Some(owner),
            None => {
                self.close_popups();
                self.hit_test(self.mouse).filter(|&i| self.is_active(i))
            }
        };
        if let Some(index) = target {
            self.with_control(index, |c, _, cx| c.on_wheel(lines, cx));
        }
    }

    pub fn mouse_move(&mut self, at: Point) {
        self.layout();
        self.mouse = at;
        if let Some(owner) = self.popup_under_mouse() {
            self.set_hover(None);
            self.with_control(owner, |c, local, cx| c.on_popup_mouse_move(local, cx));
            return;
        }
        let hover = self.hit_test(at);
        self.set_hover(hover);
        if let Some(index) = self.hover.filter(|&i| self.capture.is_none() && self.is_active(i)) {
            self.with_control(index, |c, local, cx| c.on_mouse_move(local, cx));
        }
        if let Some(index) = self.capture {
            self.with_control(index, |c, local, cx| c.on_mouse_drag(local, cx));
        }
    }

    pub fn mouse_leave(&mut self) {
        self.set_hover(None);
    }

    pub fn mouse_down(&mut self, mods: Modifiers) {
        self.tooltip = None;
        self.hover_since = None;
        if let Some(owner) = self.popup_owner() {
            if self.popup_under_mouse().is_some() {
                self.with_control(owner, |c, local, cx| c.on_popup_mouse_down(local, cx));
                return;
            }
            // Щелчок по самому владельцу он обрабатывает сам.
            if self.hit_test(self.mouse) != Some(owner) {
                self.close_popups();
            }
        }
        let Some(index) = self.hit_test(self.mouse) else { return };
        if !self.is_active(index) {
            return;
        }
        if self.can_focus(index) {
            self.set_focus(Some(index));
        }
        self.capture = Some(index);
        self.emit_mouse(index, EventKind::MouseDown);
        self.with_control(index, |c, local, cx| c.on_mouse_down(local, mods, cx));
        // Второе нажатие на тот же контрол вскоре и рядом — двойной клик.
        let now = Instant::now();
        let again = self.last_click.take().is_some_and(|(last, at, point)| {
            last == index
                && now - at < DOUBLE_CLICK
                && (point.x - self.mouse.x).abs() < 4.0
                && (point.y - self.mouse.y).abs() < 4.0
        });
        if again {
            self.emit_mouse(index, EventKind::DoubleClick);
        } else {
            self.last_click = Some((index, now, self.mouse));
        }
    }

    pub fn mouse_up(&mut self) {
        let Some(index) = self.capture.take() else { return };
        let inside = self.hit_test(self.mouse) == Some(index);
        self.emit_mouse(index, EventKind::MouseUp);
        self.with_control(index, |c, local, cx| c.on_mouse_up(local, inside, cx));
    }

    fn focus_target(&mut self) -> Option<usize> {
        let index = self.focus?;
        self.can_focus(index).then_some(index)
    }

    pub fn key(&mut self, key: Key, mods: Modifiers) {
        // Escape закрывает окошко, даже если у его владельца нет фокуса (меню).
        if key == Key::Escape && self.popup_owner().is_some() {
            self.close_popups();
            return;
        }
        if key == Key::Tab {
            self.close_popups();
            self.move_focus(mods.shift);
            return;
        }
        if let Some(index) = self.focus_target() {
            self.with_control(index, |c, _, cx| c.on_key(key, mods, cx));
        }
    }

    pub fn text_input(&mut self, text: &str) {
        if let Some(index) = self.focus_target() {
            self.with_control(index, |c, _, cx| c.on_text_input(text, cx));
        }
    }

    fn move_focus(&mut self, backwards: bool) {
        let mut order: Vec<usize> =
            (0..self.view.nodes.len()).filter(|&i| self.can_focus(i)).collect();
        // Устойчивая сортировка: при равных номерах остаётся порядок добавления.
        let mut keys: Vec<(i32, usize)> =
            order.iter().map(|&i| (self.control(i).base().tab_index, i)).collect();
        keys.sort();
        order = keys.into_iter().map(|k| k.1).collect();
        if order.is_empty() {
            return;
        }
        let current = self.focus.and_then(|f| order.iter().position(|&i| i == f));
        let next = match (current, backwards) {
            (Some(i), false) => (i + 1) % order.len(),
            (Some(i), true) => (i + order.len() - 1) % order.len(),
            (None, false) => 0,
            (None, true) => order.len() - 1,
        };
        self.set_focus(Some(order[next]));
    }

    /// Рисует форму. `scale` — масштаб экрана (1.0 = 96 DPI).
    pub fn render(&mut self, pixmap: &mut Pixmap, scale: f32) {
        self.layout();
        let back = self.ui.back_color;
        pixmap.fill(tiny_skia::Color::from_rgba8(back.r, back.g, back.b, 255));
        let window = Rect::new(
            0.0,
            0.0,
            pixmap.width() as f32 / scale,
            pixmap.height() as f32 / scale,
        );
        for index in 0..self.view.nodes.len() {
            if !self.is_shown(index) {
                continue;
            }
            let (frame, clip) = self.placement(index);
            let clip = clip.intersect(&window);
            if clip.is_empty() {
                continue;
            }
            let hovered = self.hover == Some(index);
            let state = VisualState {
                focused: self.focus == Some(index),
                hovered,
                pressed: hovered && self.capture == Some(index),
                enabled: self.is_active(index),
            };
            self.paint_control(index, pixmap, scale, frame, clip, state, false);
        }
        // Всплывающее окошко — поверх всех контролов.
        if let Some(owner) = self.popup_owner()
            && let Some(frame) = self.popup_rect(owner)
        {
            let clip = frame.intersect(&window);
            if !clip.is_empty() {
                let state = VisualState { enabled: true, ..VisualState::default() };
                self.paint_control(owner, pixmap, scale, frame, clip, state, true);
            }
        }
        self.paint_tooltip(pixmap, scale, window);
    }
}
