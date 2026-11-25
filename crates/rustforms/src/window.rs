use std::{error::Error, num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use tiny_skia::Pixmap;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{Application, Key, Modifiers, Point};

pub type RunError = Box<dyn Error>;

/// Показывает форму и работает, пока её не закроют.
pub fn run<S>(mut app: Application<S>) -> Result<(), RunError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    app.use_system_clipboard();
    let mut host = Host { app, view: None, modifiers: ModifiersState::empty(), error: None };
    event_loop.run_app(&mut host)?;
    match host.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct View {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

struct Host<S> {
    app: Application<S>,
    view: Option<View>,
    modifiers: ModifiersState,
    error: Option<RunError>,
}

impl<S> Host<S> {
    fn open(&self, event_loop: &ActiveEventLoop) -> Result<View, RunError> {
        let ui = &self.app.ui;
        let attributes = Window::default_attributes()
            .with_title(ui.title.clone())
            .with_inner_size(LogicalSize::new(ui.width as f64, ui.height as f64));
        let window = Rc::new(event_loop.create_window(attributes)?);
        let context = Context::new(window.clone())?;
        let surface = Surface::new(&context, window.clone())?;
        Ok(View { window, surface })
    }

    fn paint(&mut self) -> Result<(), RunError> {
        let Some(view) = &mut self.view else { return Ok(()) };
        let size = view.window.inner_size();
        let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return Ok(()); // окно свёрнуто
        };
        let mut pixmap = Pixmap::new(width.get(), height.get()).ok_or("слишком большое окно")?;
        self.app.render(&mut pixmap, view.window.scale_factor() as f32);
        view.surface.resize(width, height)?;
        let mut buffer = view.surface.buffer_mut()?;
        for (target, p) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            *target = u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2]);
        }
        buffer.present()?;
        Ok(())
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

/// Сочетания определяются по физической клавише, чтобы работать в русской раскладке.
fn map_key(code: KeyCode, mods: Modifiers) -> Option<Key> {
    Some(match code {
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
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

impl<S> ApplicationHandler for Host<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.view.is_some() {
            return;
        }
        self.app.load();
        match self.open(event_loop) {
            Ok(view) => {
                view.window.request_redraw();
                self.view = Some(view);
            }
            Err(error) => {
                self.error = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.view.as_ref().map(|v| v.window.clone()) else { return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.paint() {
                    self.error = Some(error);
                    event_loop.exit();
                }
                return;
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::CursorMoved { position, .. } => {
                let p = position.to_logical::<f32>(window.scale_factor());
                self.app.mouse_move(Point::new(p.x, p.y));
            }
            WindowEvent::CursorLeft { .. } => self.app.mouse_leave(),
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => match state {
                ElementState::Pressed => self.app.mouse_down(self.mods()),
                ElementState::Released => self.app.mouse_up(),
            },
            WindowEvent::KeyboardInput { event, is_synthetic: false, .. }
                if event.state == ElementState::Pressed =>
            {
                let mods = self.mods();
                if let PhysicalKey::Code(code) = event.physical_key
                    && let Some(key) = map_key(code, mods)
                {
                    self.app.key(key, mods);
                }
                if !mods.command
                    && let Some(text) = &event.text
                    && !text.chars().any(char::is_control)
                {
                    self.app.text_input(text);
                }
            }
            WindowEvent::Resized(size) => {
                let logical = size.to_logical::<f32>(window.scale_factor());
                self.app.ui.width = logical.width;
                self.app.ui.height = logical.height;
            }
            WindowEvent::ScaleFactorChanged { .. } | WindowEvent::Focused(_) => {}
            _ => return,
        }
        if self.app.close_requested() {
            event_loop.exit();
        } else {
            window.set_title(&self.app.ui.title);
            window.request_redraw();
        }
    }
}
