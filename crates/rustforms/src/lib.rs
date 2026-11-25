//! Ядро RustForms: окна, контролы, события, отрисовка.

// Чтобы `#[derive(Control)]` работал и внутри самого крейта.
extern crate self as rustforms;

mod app;
mod control;
mod controls;
mod events;
mod geometry;
mod input;
mod paint;
mod ui;
mod window;

pub use app::Application;
pub use control::{AnyControl, Control, ControlBase, ControlExt, ControlImpl, EventCx};
pub use rustforms_macros::Control;
pub use controls::{Button, CheckBox, Label, Panel, TextBox};
pub use events::{
    CheckedChangedArgs, ClickArgs, Events, LoadArgs, RaisesCheckedChanged, RaisesTextChanged,
    TextChangedArgs,
};
pub use geometry::{Color, Point, Rect};
pub use input::{Key, Modifiers};
pub use paint::{Align, Painter, TextSystem, VisualState};
pub use ui::{Container, ControlId, Handle, Ui};
pub use window::{RunError, run};

/// Всё, что обычно нужно форме.
pub mod prelude {
    pub use crate::{
        Align, Application, Button, CheckBox, CheckedChangedArgs, ClickArgs, Color, Control,
        ControlBase, ControlExt, ControlImpl, EventCx, Events, Handle, Key, Label, LoadArgs,
        Modifiers, Painter, Panel, Point, Rect, TextBox, TextChangedArgs, Ui, run,
    };
}
