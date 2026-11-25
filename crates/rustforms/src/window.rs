use std::{collections::HashMap, error::Error, num::NonZeroU32, rc::Rc, time::Instant};

use softbuffer::{Context, Surface};
use tiny_skia::Pixmap;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{Application, Key, Modifiers, Point, forms::PendingForm};

pub type RunError = Box<dyn Error>;

/// Показывает форму и работает, пока её не закроют.
pub fn run<S: 'static>(app: Application<S>) -> Result<(), RunError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut host = Host {
        first: Some(PendingForm { form: Box::new(app), modal: false, done: None }),
        windows: HashMap::new(),
        main: None,
        modifiers: ModifiersState::empty(),
        error: None,
    };
    event_loop.run_app(&mut host)?;
    match host.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct Open {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    form: PendingForm,
    parent: Option<WindowId>,
}

struct Host {
    first: Option<PendingForm>,
    windows: HashMap<WindowId, Open>,
    /// Закрытие этого окна завершает программу.
    main: Option<WindowId>,
    modifiers: ModifiersState,
    error: Option<RunError>,
}

fn paint(open: &mut Open) -> Result<(), RunError> {
    let size = open.window.inner_size();
    let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
    else {
        return Ok(()); // окно свёрнуто
    };
    let mut pixmap = Pixmap::new(width.get(), height.get()).ok_or("слишком большое окно")?;
    open.form.form.render(&mut pixmap, open.window.scale_factor() as f32);
    open.surface.resize(width, height)?;
    let mut buffer = open.surface.buffer_mut()?;
    for (target, p) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
        *target = u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2]);
    }
    buffer.present()?;
    Ok(())
}

impl Host {
    fn create(
        &mut self,
        event_loop: &ActiveEventLoop,
        mut form: PendingForm,
        parent: Option<WindowId>,
    ) -> Result<WindowId, RunError> {
        let owner = parent.and_then(|id| self.windows.get(&id)).map(|o| o.window.clone());
        let ui = form.form.ui();
        let size = LogicalSize::new(ui.width as f64, ui.height as f64);
        let mut attributes =
            Window::default_attributes().with_title(ui.title.clone()).with_inner_size(size);
        if let Some(owner) = &owner {
            // Новая форма — по центру родительской.
            if let Ok(at) = owner.outer_position() {
                let outer = owner.outer_size();
                let wanted = size.to_physical::<f64>(owner.scale_factor());
                attributes = attributes.with_position(PhysicalPosition::new(
                    at.x + ((outer.width as f64 - wanted.width) / 2.0) as i32,
                    at.y + ((outer.height as f64 - wanted.height) / 2.0) as i32,
                ));
            }
            #[cfg(windows)]
            {
                use winit::{
                    platform::windows::WindowAttributesExtWindows,
                    raw_window_handle::{HasWindowHandle, RawWindowHandle},
                };
                // Диалог держится поверх своего владельца.
                if form.modal
                    && let Ok(handle) = owner.window_handle()
                    && let RawWindowHandle::Win32(handle) = handle.as_raw()
                {
                    attributes = attributes.with_owner_window(handle.hwnd.get());
                }
            }
        }
        let window = Rc::new(event_loop.create_window(attributes)?);
        let context = Context::new(window.clone())?;
        let surface = Surface::new(&context, window.clone())?;
        form.form.ui().owner = Some(window.clone());
        form.form.use_system_clipboard();
        form.form.load();
        let id = window.id();
        window.request_redraw();
        self.windows.insert(id, Open { window, surface, form, parent });
        Ok(id)
    }

    /// Диалог, из-за которого окно не принимает ввод.
    fn blocking(&self, id: WindowId) -> Option<Rc<Window>> {
        self.windows
            .values()
            .find(|o| o.parent == Some(id) && o.form.modal)
            .map(|o| o.window.clone())
    }

    fn close(&mut self, event_loop: &ActiveEventLoop, id: WindowId) {
        // Форма может отказаться (событие `closing`).
        if let Some(open) = self.windows.get_mut(&id)
            && !open.form.form.try_close()
        {
            open.window.request_redraw();
            return;
        }
        let children: Vec<WindowId> = self
            .windows
            .iter()
            .filter(|(_, o)| o.parent == Some(id) && o.form.modal)
            .map(|(&child, _)| child)
            .collect();
        for child in children {
            self.windows.remove(&child);
        }
        let Some(mut closed) = self.windows.remove(&id) else { return };
        if Some(id) == self.main {
            self.windows.clear();
            event_loop.exit();
            return;
        }
        if let Some(parent) = closed.parent.and_then(|p| self.windows.get_mut(&p)) {
            if let Some(done) = closed.form.done.take() {
                let accepted = closed.form.form.ui().accepted;
                parent.form.form.finish(done, closed.form.form.state(), accepted);
            }
            parent.window.focus_window();
            parent.window.request_redraw();
        }
    }

    /// То, что обработчики запросили: открыть формы, закрыть, обновить заголовок.
    fn settle(&mut self, event_loop: &ActiveEventLoop) {
        loop {
            let opening = self.windows.iter_mut().find_map(|(&id, o)| {
                let pending = &mut o.form.form.ui().pending;
                (!pending.is_empty()).then(|| (id, pending.remove(0)))
            });
            if let Some((parent, form)) = opening {
                if let Err(error) = self.create(event_loop, form, Some(parent)) {
                    self.error = Some(error);
                    event_loop.exit();
                    return;
                }
                continue;
            }
            let closing = self
                .windows
                .iter_mut()
                .find_map(|(&id, o)| o.form.form.ui().close_requested.then_some(id));
            match closing {
                Some(id) => self.close(event_loop, id),
                None => break,
            }
        }
        for open in self.windows.values_mut() {
            open.window.set_title(&open.form.form.ui().title);
            open.window.request_redraw();
        }
    }

    fn mods(&self) -> Modifiers {
        let command = if cfg!(target_os = "macos") {
            self.modifiers.super_key()
        } else {
            // Ctrl+Alt — это AltGr, им набирают символы.
            self.modifiers.control_key() && !self.modifiers.alt_key()
        };
        Modifiers { shift: self.modifiers.shift_key(), command }
    }
}

/// `KeyA` → `A`, `Digit1` → `1`; остальные имена как есть (`Enter`, `ArrowUp`, `F5`).
fn key_name(code: KeyCode) -> String {
    let name = format!("{code:?}");
    name.strip_prefix("Key").or_else(|| name.strip_prefix("Digit")).unwrap_or(&name).to_owned()
}

/// Сочетания определяются по физической клавише, чтобы работать в русской раскладке.
fn map_key(code: KeyCode, mods: Modifiers) -> Option<Key> {
    Some(match code {
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::Escape => Key::Escape,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,
        KeyCode::Space => Key::Space,
        KeyCode::Tab => Key::Tab,
        KeyCode::KeyA if mods.command => Key::SelectAll,
        KeyCode::KeyC if mods.command => Key::Copy,
        KeyCode::KeyX if mods.command => Key::Cut,
        KeyCode::KeyV if mods.command => Key::Paste,
        _ => return None,
    })
}

impl ApplicationHandler for Host {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Единственное, что происходит без событий, — появление подсказки.
        let now = Instant::now();
        let mut next: Option<Instant> = None;
        for open in self.windows.values_mut() {
            if open.form.form.tick(now) {
                open.window.request_redraw();
            }
            if let Some(at) = open.form.form.tooltip_deadline() {
                next = Some(next.map_or(at, |n| n.min(at)));
            }
        }
        event_loop.set_control_flow(next.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(first) = self.first.take() else { return };
        match self.create(event_loop, first, None) {
            Ok(id) => {
                self.main = Some(id);
                self.settle(event_loop);
            }
            Err(error) => {
                self.error = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let WindowEvent::ModifiersChanged(modifiers) = &event {
            self.modifiers = modifiers.state();
        }
        let mods = self.mods();
        let blocking = self.blocking(id);
        let Some(open) = self.windows.get_mut(&id) else { return };
        // Пока открыт диалог, окно только рисуется и меняет размер.
        if let Some(dialog) = blocking {
            match event {
                WindowEvent::MouseInput { .. }
                | WindowEvent::Focused(true)
                | WindowEvent::CloseRequested => {
                    dialog.focus_window();
                    return;
                }
                WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CursorMoved { .. } => return,
                _ => {}
            }
        }
        let window = open.window.clone();
        let form = &mut open.form.form;
        match event {
            WindowEvent::CloseRequested => {
                self.close(event_loop, id);
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = paint(open) {
                    self.error = Some(error);
                    event_loop.exit();
                }
                return;
            }
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::CursorMoved { position, .. } => {
                let p = position.to_logical::<f32>(window.scale_factor());
                form.mouse_move(Point::new(p.x, p.y));
            }
            WindowEvent::CursorLeft { .. } => form.mouse_leave(),
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => match state {
                ElementState::Pressed => form.mouse_down(mods),
                ElementState::Released => form.mouse_up(),
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 3.0,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 20.0,
                };
                form.wheel(lines);
            }
            WindowEvent::KeyboardInput { event, is_synthetic: false, .. }
                if event.state == ElementState::Released =>
            {
                if let PhysicalKey::Code(code) = event.physical_key {
                    form.raw_key(&key_name(code), mods, false);
                }
            }
            WindowEvent::KeyboardInput { event, is_synthetic: false, .. }
                if event.state == ElementState::Pressed =>
            {
                if let PhysicalKey::Code(code) = event.physical_key {
                    form.raw_key(&key_name(code), mods, true);
                }
                if let PhysicalKey::Code(code) = event.physical_key
                    && let Some(key) = map_key(code, mods)
                {
                    form.key(key, mods);
                }
                if !mods.command
                    && let Some(text) = &event.text
                    && !text.chars().any(char::is_control)
                {
                    form.text_input(text);
                }
            }
            WindowEvent::Resized(size) => {
                let logical = size.to_logical::<f32>(window.scale_factor());
                form.ui().width = logical.width;
                form.ui().height = logical.height;
                form.resized();
            }
            WindowEvent::Focused(false) => form.close_popups(),
            WindowEvent::ScaleFactorChanged { .. } | WindowEvent::Focused(true) => {}
            _ => return,
        }
        self.settle(event_loop);
    }
}
