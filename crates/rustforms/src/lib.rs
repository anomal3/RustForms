//! Ядро RustForms: окна, контролы, события, отрисовка.

// Чтобы `#[derive(Control)]` работал и внутри самого крейта.
extern crate self as rustforms;

mod app;
mod control;
mod controls;
mod date;
mod dialogs;
mod forms;
mod geometry;
mod input;
pub mod layout;
mod paint;
mod timer;
mod ui;
mod view;
mod window;

pub use app::Application;
pub use control::{AnyControl, Control, ControlBase, ControlExt, ControlImpl, EventCx};
pub use rustforms_macros::Control;
pub use controls::{
    Button, CheckBox, ComboBox, DataGridView, GroupBox, Label, LinkLabel, ListBox, MenuStrip,
    NumericUpDown, Panel, PictureBox, ProgressBar, RadioButton, SizeMode, StatusStrip, TextBox,
    TrackBar, TreeView, TAB_HEADER, TabControl, TabPage,
    DateTimePicker, SPLITTER, SplitContainer, SplitPanel, ToolStrip,
};
pub use dialogs::FileFilter;
pub use forms::PendingForm;
pub use geometry::{Color, Point, Rect};
pub use input::{Key, Modifiers};
pub use layout::{Anchor, Dock};
pub use paint::{Align, Painter, TextSystem, VisualState};
pub use date::Date;
pub use timer::Timer;
pub use ui::{Container, ControlId, Handle, Ui};
pub use paint::{FontStyle, TextAlign};
pub use view::{
    ClosingArgs, FocusArgs, KeyArgs, MouseArgs, ResizeArgs,
    CheckedChangedArgs, ClickArgs, LoadArgs, MenuItemArgs, RaisesCellChanged, RaisesCheckedChanged, RaisesItemClicked, RaisesSelectedChanged,
    RaisesTextChanged, RaisesValueChanged, CellChangedArgs, TickArgs, ValueChangedArgs, SelectedChangedArgs, TextChangedArgs, View,
};
pub use window::{RunError, run};

/// Всё, что обычно нужно форме.
pub mod prelude {
    pub use crate::{
        CellChangedArgs, Date, DateTimePicker, SplitContainer, SplitPanel, ToolStrip, TabControl, TabPage, TickArgs, Timer, DataGridView, GroupBox, LinkLabel, NumericUpDown, PictureBox, SizeMode, StatusStrip,
        TrackBar, TreeView, ValueChangedArgs, ClosingArgs, FocusArgs, KeyArgs, MouseArgs, ResizeArgs, TextAlign, MenuItemArgs, MenuStrip, ComboBox, ListBox, ProgressBar, RadioButton, SelectedChangedArgs, Align, Anchor, Dock, Application, Button, CheckBox, CheckedChangedArgs, ClickArgs, Color, Control,
        ControlBase, ControlExt, ControlImpl, EventCx, Handle, Key, Label, LoadArgs,
        Modifiers, Painter, Panel, Point, Rect, TextBox, TextChangedArgs, Ui, View, run,
    };
}
