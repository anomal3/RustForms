//! Ядро RustForms: окна, контролы, события, отрисовка.

// Чтобы `#[derive(Control)]` работал и внутри самого крейта.
extern crate self as rustforms;

mod app;
mod control;
mod controls;
mod dialogs;
mod forms;
mod geometry;
mod input;
pub mod layout;
mod paint;
mod ui;
mod view;
mod window;

pub use app::Application;
pub use control::{AnyControl, Control, ControlBase, ControlExt, ControlImpl, EventCx};
pub use rustforms_macros::Control;
pub use controls::{
    Button, CheckBox, ComboBox, Label, ListBox, MenuStrip, Panel, ProgressBar, RadioButton,
    TextBox,
};
pub use dialogs::FileFilter;
pub use forms::PendingForm;
pub use geometry::{Color, Point, Rect};
pub use input::{Key, Modifiers};
pub use layout::{Anchor, Dock};
pub use paint::{Align, Painter, TextSystem, VisualState};
pub use ui::{Container, ControlId, Handle, Ui};
pub use paint::{FontStyle, TextAlign};
pub use view::{
    ClosingArgs, FocusArgs, KeyArgs, MouseArgs, ResizeArgs,
    CheckedChangedArgs, ClickArgs, LoadArgs, MenuItemArgs, RaisesCheckedChanged, RaisesItemClicked, RaisesSelectedChanged,
    RaisesTextChanged, SelectedChangedArgs, TextChangedArgs, View,
};
pub use window::{RunError, run};

/// Всё, что обычно нужно форме.
pub mod prelude {
    pub use crate::{
        ClosingArgs, FocusArgs, KeyArgs, MouseArgs, ResizeArgs, TextAlign, MenuItemArgs, MenuStrip, ComboBox, ListBox, ProgressBar, RadioButton, SelectedChangedArgs, Align, Anchor, Dock, Application, Button, CheckBox, CheckedChangedArgs, ClickArgs, Color, Control,
        ControlBase, ControlExt, ControlImpl, EventCx, Handle, Key, Label, LoadArgs,
        Modifiers, Painter, Panel, Point, Rect, TextBox, TextChangedArgs, Ui, View, run,
    };
}
