/// Клавиши, на которые реагируют контролы. Набор текста приходит отдельно.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Escape,
    Home,
    End,
    Backspace,
    Delete,
    Enter,
    Space,
    Tab,
    SelectAll,
    Copy,
    Cut,
    Paste,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    /// Ctrl, а на macOS — Cmd.
    pub command: bool,
}
