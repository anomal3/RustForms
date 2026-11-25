use tiny_skia::Pixmap;

use crate::{
    ControlId, EventCx, Events, Key, Modifiers, Painter, Point, Rect, TextSystem, Ui, VisualState,
    events::Signal,
};

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

/// Форма вместе с состоянием и подписками. Не зависит от окна, поэтому ввод
/// можно подавать из тестов, а кадр рисовать в память.
pub struct Application<S> {
    pub ui: Ui,
    pub state: S,
    events: Events<S>,
    text: TextSystem,
    clipboard: Clipboard,
    focus: Option<ControlId>,
    hover: Option<ControlId>,
    capture: Option<ControlId>,
    mouse: Point,
    loaded: bool,
}

impl<S> Application<S> {
    pub fn new(ui: Ui, state: S, events: Events<S>) -> Self {
        Self {
            ui,
            state,
            events,
            text: TextSystem::new(),
            clipboard: Clipboard { system: None, local: String::new() },
            focus: None,
            hover: None,
            capture: None,
            mouse: Point::default(),
            loaded: false,
        }
    }

    pub(crate) fn use_system_clipboard(&mut self) {
        self.clipboard.system = arboard::Clipboard::new().ok();
    }

    /// Вызывает `on_load` один раз.
    pub fn load(&mut self) {
        if !self.loaded {
            self.loaded = true;
            self.events.load(&mut self.state, &mut self.ui);
        }
    }

    pub fn focused(&self) -> Option<ControlId> {
        self.focus
    }

    pub fn close_requested(&self) -> bool {
        self.ui.close_requested
    }

    fn with_control(
        &mut self,
        id: ControlId,
        action: impl FnOnce(&mut dyn crate::AnyControl, Point, &mut EventCx),
    ) {
        let (rect, _) = self.ui.placement(id);
        let local = Point::new(self.mouse.x - rect.x, self.mouse.y - rect.y);
        let mut signals: Vec<Signal> = Vec::new();
        let mut cx = EventCx {
            text: &mut self.text,
            clipboard: &mut self.clipboard,
            signals: &mut signals,
            size: (rect.width, rect.height),
        };
        action(&mut *self.ui.nodes[id.0].control, local, &mut cx);
        for signal in signals {
            self.events.dispatch(id, signal, &mut self.state, &mut self.ui);
        }
    }

    pub fn mouse_move(&mut self, at: Point) {
        self.mouse = at;
        self.hover = self.ui.hit_test(at);
        if let Some(id) = self.capture {
            self.with_control(id, |c, local, cx| c.on_mouse_drag(local, cx));
        }
    }

    pub fn mouse_leave(&mut self) {
        self.hover = None;
    }

    pub fn mouse_down(&mut self, mods: Modifiers) {
        let Some(id) = self.ui.hit_test(self.mouse) else { return };
        if !self.ui.is_active(id) {
            return;
        }
        if self.ui.can_focus(id) {
            self.focus = Some(id);
        }
        self.capture = Some(id);
        self.with_control(id, |c, local, cx| c.on_mouse_down(local, mods, cx));
    }

    pub fn mouse_up(&mut self) {
        let Some(id) = self.capture.take() else { return };
        let inside = self.ui.hit_test(self.mouse) == Some(id);
        self.with_control(id, |c, local, cx| c.on_mouse_up(local, inside, cx));
    }

    pub fn key(&mut self, key: Key, mods: Modifiers) {
        if key == Key::Tab {
            self.move_focus(mods.shift);
            return;
        }
        if let Some(id) = self.focus.filter(|&id| self.ui.can_focus(id)) {
            self.with_control(id, |c, _, cx| c.on_key(key, mods, cx));
        }
    }

    pub fn text_input(&mut self, text: &str) {
        if let Some(id) = self.focus.filter(|&id| self.ui.can_focus(id)) {
            self.with_control(id, |c, _, cx| c.on_text_input(text, cx));
        }
    }

    fn move_focus(&mut self, backwards: bool) {
        let order: Vec<ControlId> = (0..self.ui.nodes.len())
            .map(ControlId)
            .filter(|&id| self.ui.can_focus(id))
            .collect();
        if order.is_empty() {
            return;
        }
        let current = self.focus.and_then(|f| order.iter().position(|&id| id == f));
        let next = match (current, backwards) {
            (Some(i), false) => (i + 1) % order.len(),
            (Some(i), true) => (i + order.len() - 1) % order.len(),
            (None, false) => 0,
            (None, true) => order.len() - 1,
        };
        self.focus = Some(order[next]);
    }

    /// Рисует форму. `scale` — масштаб экрана (1.0 = 96 DPI).
    pub fn render(&mut self, pixmap: &mut Pixmap, scale: f32) {
        let back = self.ui.back_color;
        pixmap.fill(tiny_skia::Color::from_rgba8(back.r, back.g, back.b, 255));
        for index in 0..self.ui.nodes.len() {
            let id = ControlId(index);
            if !self.ui.is_shown(id) {
                continue;
            }
            let (frame, clip) = self.ui.placement(id);
            let window = Rect::new(
                0.0,
                0.0,
                pixmap.width() as f32 / scale,
                pixmap.height() as f32 / scale,
            );
            let clip = clip.intersect(&window);
            if clip.is_empty() {
                continue;
            }
            let hovered = self.hover == Some(id);
            let state = VisualState {
                focused: self.focus == Some(id),
                hovered,
                pressed: hovered && self.capture == Some(id),
                enabled: self.ui.is_active(id),
            };
            let mut painter = Painter {
                pixmap: &mut *pixmap,
                text: &mut self.text,
                scale,
                frame,
                clip,
                state,
            };
            self.ui.nodes[index].control.on_paint(&mut painter);
        }
    }
}
