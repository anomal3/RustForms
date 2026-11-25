use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{Error, fail};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Button,
    Label,
    TextBox,
    CheckBox,
    RadioButton,
    ComboBox,
    ListBox,
    ProgressBar,
    MenuStrip,
    Panel,
    GroupBox,
    LinkLabel,
    NumericUpDown,
    TrackBar,
    PictureBox,
    StatusStrip,
    DataGridView,
    TreeView,
    TabControl,
    TabPage,
    Timer,
    DateTimePicker,
    SplitContainer,
    SplitPanel,
    ToolStrip,
}

impl Kind {
    pub const ALL: [Kind; 25] = [
        Kind::Button,
        Kind::Label,
        Kind::TextBox,
        Kind::CheckBox,
        Kind::RadioButton,
        Kind::ComboBox,
        Kind::ListBox,
        Kind::ProgressBar,
        Kind::MenuStrip,
        Kind::Panel,
        Kind::GroupBox,
        Kind::LinkLabel,
        Kind::NumericUpDown,
        Kind::TrackBar,
        Kind::PictureBox,
        Kind::StatusStrip,
        Kind::DataGridView,
        Kind::TreeView,
        Kind::TabControl,
        Kind::TabPage,
        Kind::Timer,
        Kind::DateTimePicker,
        Kind::SplitContainer,
        Kind::SplitPanel,
        Kind::ToolStrip,
    ];

    /// Высота ярлычков вкладок: страницы занимают всё под ними.
    pub const TAB_HEADER: f32 = 28.0;

    /// Внутрь этих можно класть другие контролы (в `TabControl` — только страницы).
    pub fn container(self) -> bool {
        matches!(self, Kind::Panel | Kind::GroupBox | Kind::TabPage | Kind::SplitPanel)
    }

    /// Можно ли положить контрол этого вида в `parent` (`None` — прямо на форму).
    pub fn fits_in(self, parent: Option<Kind>) -> bool {
        match self {
            Kind::TabPage => parent == Some(Kind::TabControl),
            Kind::SplitPanel => parent == Some(Kind::SplitContainer),
            // Таймер не контрол и ни в чём не лежит.
            Kind::Timer => parent.is_none(),
            _ => parent.is_none_or(Kind::container),
        }
    }

    /// У этих есть значение в диапазоне.
    pub fn ranged(self) -> bool {
        matches!(self, Kind::ProgressBar | Kind::NumericUpDown | Kind::TrackBar)
    }

    /// У этих контролов есть подпись, задаваемая конструктором.
    pub fn captioned(self) -> bool {
        matches!(
            self,
            Kind::Button
                | Kind::Label
                | Kind::CheckBox
                | Kind::RadioButton
                | Kind::GroupBox
                | Kind::LinkLabel
                | Kind::StatusStrip
                | Kind::TabPage
        )
    }

    pub fn has_text(self) -> bool {
        self.captioned() || self == Kind::TextBox
    }

    pub fn has_items(self) -> bool {
        matches!(
            self,
            Kind::ComboBox
                | Kind::ListBox
                | Kind::MenuStrip
                | Kind::DataGridView
                | Kind::TreeView
                | Kind::ToolStrip
        )
    }

    /// Списки, в которых есть выбранная строка.
    pub fn selectable(self) -> bool {
        matches!(self, Kind::ComboBox | Kind::ListBox)
    }

    pub fn checkable(self) -> bool {
        matches!(self, Kind::CheckBox | Kind::RadioButton)
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Button => "Button",
            Kind::Label => "Label",
            Kind::TextBox => "TextBox",
            Kind::CheckBox => "CheckBox",
            Kind::RadioButton => "RadioButton",
            Kind::ComboBox => "ComboBox",
            Kind::ListBox => "ListBox",
            Kind::ProgressBar => "ProgressBar",
            Kind::MenuStrip => "MenuStrip",
            Kind::Panel => "Panel",
            Kind::GroupBox => "GroupBox",
            Kind::LinkLabel => "LinkLabel",
            Kind::NumericUpDown => "NumericUpDown",
            Kind::TrackBar => "TrackBar",
            Kind::PictureBox => "PictureBox",
            Kind::StatusStrip => "StatusStrip",
            Kind::DataGridView => "DataGridView",
            Kind::TreeView => "TreeView",
            Kind::TabControl => "TabControl",
            Kind::TabPage => "TabPage",
            Kind::Timer => "Timer",
            Kind::DateTimePicker => "DateTimePicker",
            Kind::SplitContainer => "SplitContainer",
            Kind::SplitPanel => "SplitPanel",
            Kind::ToolStrip => "ToolStrip",
        }
    }

    pub fn default_size(self) -> (f32, f32) {
        match self {
            Kind::Button => (90.0, 28.0),
            Kind::Label => (120.0, 22.0),
            Kind::TextBox => (160.0, 28.0),
            Kind::CheckBox | Kind::RadioButton => (140.0, 22.0),
            Kind::ComboBox => (160.0, 28.0),
            Kind::ListBox => (160.0, 112.0),
            Kind::ProgressBar => (200.0, 18.0),
            Kind::MenuStrip => (300.0, 26.0),
            Kind::GroupBox => (200.0, 100.0),
            Kind::LinkLabel => (120.0, 22.0),
            Kind::NumericUpDown => (100.0, 28.0),
            Kind::TrackBar => (200.0, 28.0),
            Kind::PictureBox => (120.0, 90.0),
            Kind::StatusStrip => (300.0, 24.0),
            Kind::DataGridView => (320.0, 150.0),
            Kind::TreeView => (180.0, 150.0),
            Kind::TabControl => (300.0, 180.0),
            Kind::TabPage => (300.0, 152.0),
            Kind::Timer => (28.0, 28.0),
            Kind::DateTimePicker => (150.0, 28.0),
            Kind::SplitContainer => (300.0, 160.0),
            Kind::SplitPanel => (100.0, 160.0),
            Kind::ToolStrip => (300.0, 30.0),
            Kind::Panel => (200.0, 100.0),
        }
    }

    pub fn focusable(self) -> bool {
        !matches!(
            self,
            Kind::Label
                | Kind::ProgressBar
                | Kind::MenuStrip
                | Kind::Panel
                | Kind::GroupBox
                | Kind::PictureBox
                | Kind::StatusStrip
                | Kind::TabPage
                | Kind::Timer
                | Kind::SplitContainer
                | Kind::SplitPanel
                | Kind::ToolStrip
        )
    }

    /// События контрола; первое создаётся двойным кликом в дизайнере.
    pub fn events(self) -> Vec<Event> {
        if self == Kind::Timer {
            return vec![Event::Tick];
        }
        let mut events = match self {
            Kind::Button | Kind::Label | Kind::LinkLabel | Kind::PictureBox => vec![Event::Click],
            Kind::TextBox => vec![Event::TextChanged],
            Kind::CheckBox | Kind::RadioButton => vec![Event::CheckedChanged, Event::Click],
            Kind::ComboBox | Kind::ListBox | Kind::TreeView | Kind::DateTimePicker => {
                vec![Event::SelectedChanged]
            }
            Kind::DataGridView => vec![Event::SelectedChanged, Event::CellChanged],
            Kind::ToolStrip => vec![Event::ItemClicked],
            Kind::NumericUpDown | Kind::TrackBar => vec![Event::ValueChanged],
            Kind::MenuStrip => vec![Event::ItemClicked],
            Kind::TabControl => vec![Event::SelectedChanged],
            Kind::ProgressBar
            | Kind::Panel
            | Kind::GroupBox
            | Kind::StatusStrip
            | Kind::TabPage
            | Kind::Timer
            | Kind::SplitContainer
            | Kind::SplitPanel => Vec::new(),
        };
        events.extend([
            Event::DoubleClick,
            Event::MouseEnter,
            Event::MouseLeave,
            Event::MouseDown,
            Event::MouseUp,
            Event::ContextItemClicked,
        ]);
        if self.focusable() {
            events.extend([Event::KeyDown, Event::KeyUp, Event::GotFocus, Event::LostFocus]);
        }
        events
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Load,
    Click,
    TextChanged,
    CheckedChanged,
    SelectedChanged,
    ItemClicked,
    DoubleClick,
    MouseEnter,
    MouseLeave,
    MouseDown,
    MouseUp,
    KeyDown,
    KeyUp,
    GotFocus,
    LostFocus,
    Closing,
    Resize,
    ValueChanged,
    Tick,
    CellChanged,
    ContextItemClicked,
}

impl Event {
    pub const ALL: [Event; 21] = [
        Event::Load,
        Event::Click,
        Event::TextChanged,
        Event::CheckedChanged,
        Event::SelectedChanged,
        Event::ItemClicked,
        Event::DoubleClick,
        Event::MouseEnter,
        Event::MouseLeave,
        Event::MouseDown,
        Event::MouseUp,
        Event::KeyDown,
        Event::KeyUp,
        Event::GotFocus,
        Event::LostFocus,
        Event::Closing,
        Event::Resize,
        Event::ValueChanged,
        Event::Tick,
        Event::CellChanged,
        Event::ContextItemClicked,
    ];

    /// События самой формы, а не контрола.
    pub const FORM: [Event; 3] = [Event::Load, Event::Closing, Event::Resize];

    pub fn name(self) -> &'static str {
        match self {
            Event::Load => "load",
            Event::Click => "click",
            Event::TextChanged => "text_changed",
            Event::CheckedChanged => "checked_changed",
            Event::SelectedChanged => "selected_changed",
            Event::ItemClicked => "item_clicked",
            Event::DoubleClick => "double_click",
            Event::MouseEnter => "mouse_enter",
            Event::MouseLeave => "mouse_leave",
            Event::MouseDown => "mouse_down",
            Event::MouseUp => "mouse_up",
            Event::KeyDown => "key_down",
            Event::KeyUp => "key_up",
            Event::GotFocus => "got_focus",
            Event::LostFocus => "lost_focus",
            Event::Closing => "closing",
            Event::Resize => "resize",
            Event::ValueChanged => "value_changed",
            Event::Tick => "tick",
            Event::CellChanged => "cell_changed",
            Event::ContextItemClicked => "context_item_clicked",
        }
    }

    pub fn args(self) -> &'static str {
        match self {
            Event::Load => "LoadArgs",
            Event::Click => "ClickArgs",
            Event::TextChanged => "TextChangedArgs",
            Event::CheckedChanged => "CheckedChangedArgs",
            Event::SelectedChanged => "SelectedChangedArgs",
            Event::ItemClicked => "MenuItemArgs",
            Event::DoubleClick
            | Event::MouseEnter
            | Event::MouseLeave
            | Event::MouseDown
            | Event::MouseUp => "MouseArgs",
            Event::KeyDown | Event::KeyUp => "KeyArgs",
            Event::GotFocus | Event::LostFocus => "FocusArgs",
            Event::Closing => "ClosingArgs",
            Event::Resize => "ResizeArgs",
            Event::ValueChanged => "ValueChangedArgs",
            Event::Tick => "TickArgs",
            Event::CellChanged => "CellChangedArgs",
            Event::ContextItemClicked => "MenuItemArgs",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Left, Side::Top, Side::Right, Side::Bottom];

    pub fn name(self) -> &'static str {
        match self {
            Side::Left => "left",
            Side::Top => "top",
            Side::Right => "right",
            Side::Bottom => "bottom",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dock {
    #[default]
    None,
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}

impl Dock {
    pub const ALL: [Dock; 6] =
        [Dock::None, Dock::Top, Dock::Bottom, Dock::Left, Dock::Right, Dock::Fill];

    pub fn name(self) -> &'static str {
        match self {
            Dock::None => "none",
            Dock::Top => "top",
            Dock::Bottom => "bottom",
            Dock::Left => "left",
            Dock::Right => "right",
            Dock::Fill => "fill",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Align {
    pub const ALL: [Align; 3] = [Align::Left, Align::Center, Align::Right];

    pub fn name(self) -> &'static str {
        match self {
            Align::Left => "left",
            Align::Center => "center",
            Align::Right => "right",
        }
    }
}

pub fn default_anchor() -> BTreeSet<Side> {
    BTreeSet::from([Side::Left, Side::Top])
}

fn is_default_anchor(anchor: &BTreeSet<Side>) -> bool {
    *anchor == default_anchor()
}

fn is_no_dock(dock: &Dock) -> bool {
    *dock == Dock::None
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormProps {
    /// Имя структуры формы, например `MainForm`.
    pub name: String,
    pub title: String,
    pub width: f32,
    pub height: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_color: Option<String>,
    /// Имена обработчиков событий формы.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resize: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    #[serde(rename = "type")]
    pub kind: Kind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Путь к своему контролу на базе `kind`, например `crate::round_button::RoundButton`.
    /// Он должен реализовать `Default`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Стороны родителя, к которым привязан контрол.
    #[serde(default = "default_anchor", skip_serializing_if = "is_default_anchor")]
    pub anchor: BTreeSet<Side>,
    #[serde(default, skip_serializing_if = "is_no_dock")]
    pub dock: Dock,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub visible: bool,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub enabled: bool,
    /// `CheckBox` и `RadioButton`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub checked: bool,
    /// Только `TextBox`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub multiline: bool,
    /// Строки `ComboBox` и `ListBox`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// Меню по правой кнопке: `Пункт`, `Подменю/Пункт`, `-`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<String>,
    /// Только `DateTimePicker`: `ГГГГ-ММ-ДД`; без неё — сегодняшняя.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Только `DataGridView`: разрешить правку ячеек.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub editable: bool,
    /// Только `SplitContainer`: панели друг над другом.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub horizontal: bool,
    /// Только `SplitContainer`: размер первой панели.
    #[serde(default = "hundred_u", skip_serializing_if = "is_hundred_u")]
    pub distance: u32,
    /// Только `Timer`: интервал в миллисекундах.
    #[serde(default = "second", skip_serializing_if = "is_second")]
    pub interval: u32,
    /// Файл PNG для `PictureBox`, путь относительно папки формы.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Выбранная строка (с нуля).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<usize>,
    /// Только `ProgressBar`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub value: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub minimum: i32,
    #[serde(default = "hundred", skip_serializing_if = "is_hundred")]
    pub maximum: i32,
    /// Цвета в виде `#rrggbb`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fore_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub tab_index: i32,
    /// `Button` и `Label`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    /// Только `TextBox`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub password: bool,
    /// Только `Panel`.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub border: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub events: BTreeMap<Event, String>,
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}

fn hundred() -> i32 {
    100
}

fn is_hundred(value: &i32) -> bool {
    *value == 100
}

fn hundred_u() -> u32 {
    100
}

fn is_hundred_u(value: &u32) -> bool {
    *value == 100
}

/// Три числа даты `ГГГГ-ММ-ДД`.
pub(crate) fn parse_date(text: &str) -> Result<(i32, u32, u32), Error> {
    let bad = || Error(format!("дата `{text}` должна быть в виде ГГГГ-ММ-ДД"));
    let mut parts = text.split('-').map(|p| p.parse::<u32>().ok());
    let (Some(Some(y)), Some(Some(m)), Some(Some(d)), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(bad());
    };
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => 0,
    };
    if !(1..=9999).contains(&y) || d == 0 || d > days {
        return Err(bad());
    }
    Ok((y as i32, m, d))
}

fn second() -> u32 {
    1000
}

fn is_second(value: &u32) -> bool {
    *value == 1000
}

fn yes() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

impl Control {
    pub fn new(kind: Kind, name: impl Into<String>, x: f32, y: f32) -> Self {
        let name = name.into();
        let (width, height) = kind.default_size();
        let text = kind.captioned().then(|| name.clone());
        Self {
            kind,
            name,
            parent: None,
            custom: None,
            x,
            y,
            width,
            height,
            text,
            anchor: default_anchor(),
            // Меню по умолчанию прижато к верхнему краю.
            dock: match kind {
                Kind::MenuStrip | Kind::ToolStrip => Dock::Top,
                Kind::StatusStrip => Dock::Bottom,
                _ => Dock::None,
            },
            visible: true,
            // Таймер, как в WinForms, сначала остановлен.
            enabled: kind != Kind::Timer,
            checked: false,
            multiline: false,
            items: match kind {
                Kind::MenuStrip => vec!["Файл/Открыть".into(), "Файл/-".into(), "Файл/Выход".into()],
                Kind::DataGridView => vec!["Столбец 1".into(), "Столбец 2".into()],
                Kind::TreeView => vec!["Узел/Дочерний".into()],
                Kind::ToolStrip => {
                    vec!["Новый".into(), "Открыть".into(), "-".into(), "Сохранить".into()]
                }
                _ => Vec::new(),
            },
            image: None,
            interval: 1000,
            context: Vec::new(),
            date: None,
            editable: false,
            horizontal: false,
            distance: 100,
            selected: None,
            value: 0,
            minimum: 0,
            maximum: 100,
            fore_color: None,
            back_color: None,
            font_size: None,
            bold: false,
            tooltip: None,
            tab_index: 0,
            align: None,
            read_only: false,
            password: false,
            border: true,
            events: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormModel {
    pub form: FormProps,
    /// Порядок важен: это порядок отрисовки и перехода по Tab; родитель идёт раньше детей.
    #[serde(default, rename = "control", skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<Control>,
}

impl FormModel {
    pub fn new(name: &str) -> Self {
        Self {
            form: FormProps {
                name: name.to_owned(),
                title: name.to_owned(),
                width: 480.0,
                height: 320.0,
                back_color: None,
                load: None,
                closing: None,
                resize: None,
            },
            controls: Vec::new(),
        }
    }

    pub fn from_toml(text: &str) -> Result<Self, Error> {
        let model: Self =
            toml::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| Error(e.to_string()))?;
        model.validate()?;
        Ok(model)
    }

    pub fn to_toml(&self) -> Result<String, Error> {
        toml::to_string(self).map_err(|e| Error(e.to_string()))
    }

    /// Свободное имя вида `button1`.
    pub fn free_name(&self, kind: Kind) -> String {
        // Имя поля структуры, поэтому в стиле Rust: `text_box1`.
        let prefix = match kind {
            Kind::Button => "button",
            Kind::Label => "label",
            Kind::TextBox => "text_box",
            Kind::CheckBox => "check_box",
            Kind::RadioButton => "radio_button",
            Kind::ComboBox => "combo_box",
            Kind::ListBox => "list_box",
            Kind::ProgressBar => "progress_bar",
            Kind::MenuStrip => "menu_strip",
            Kind::GroupBox => "group_box",
            Kind::LinkLabel => "link_label",
            Kind::NumericUpDown => "numeric_up_down",
            Kind::TrackBar => "track_bar",
            Kind::PictureBox => "picture_box",
            Kind::StatusStrip => "status_strip",
            Kind::DataGridView => "data_grid_view",
            Kind::TreeView => "tree_view",
            Kind::TabControl => "tab_control",
            Kind::TabPage => "tab_page",
            Kind::Timer => "timer",
            Kind::DateTimePicker => "date_time_picker",
            Kind::SplitContainer => "split_container",
            Kind::SplitPanel => "split_panel",
            Kind::ToolStrip => "tool_strip",
            Kind::Panel => "panel",
        };
        (1..)
            .map(|n| format!("{prefix}{n}"))
            .find(|name| self.controls.iter().all(|c| &c.name != name))
            .expect("свободное имя всегда найдётся")
    }

    /// Обработчики формы вместе с типом события.
    pub fn handlers(&self) -> Vec<(&str, Event)> {
        let form = &self.form;
        let load = [(&form.load, Event::Load), (&form.closing, Event::Closing), (&form.resize, Event::Resize)]
            .into_iter()
            .filter_map(|(name, event)| name.as_deref().map(|n| (n, event)));
        let controls = self
            .controls
            .iter()
            .flat_map(|c| c.events.iter().map(|(&event, name)| (name.as_str(), event)));
        load.chain(controls).collect()
    }

    pub fn validate(&self) -> Result<(), Error> {
        identifier(&self.form.name)?;
        color(&self.form.back_color)?;
        if !(self.form.width > 0.0 && self.form.height > 0.0) {
            return fail("размер формы должен быть положительным");
        }
        let mut seen: HashMap<&str, Kind> = HashMap::new();
        let mut names = HashSet::new();
        for c in &self.controls {
            identifier(&c.name)?;
            if !names.insert(c.name.as_str()) {
                return fail(format!("имя `{}` занято дважды", c.name));
            }
            let parent = match &c.parent {
                Some(parent) => match seen.get(parent.as_str()) {
                    Some(&kind) => Some(kind),
                    None => {
                        return fail(format!("`{}`: `{parent}` должен быть объявлен выше", c.name));
                    }
                },
                None => None,
            };
            if !c.kind.fits_in(parent) {
                return fail(match c.kind {
                    Kind::TabPage => format!("`{}`: страница должна лежать в TabControl", c.name),
                    Kind::SplitPanel => {
                        format!("`{}`: панель должна лежать в SplitContainer", c.name)
                    }
                    Kind::Timer => format!("`{}`: таймер не кладётся внутрь контролов", c.name),
                    _ => format!("`{}`: сюда этот контрол положить нельзя", c.name),
                });
            }
            if let Some(date) = &c.date {
                parse_date(date)?;
            }
            if let Some(custom) = &c.custom
                && syn::parse_str::<syn::Path>(custom).is_err()
            {
                return fail(format!("`{}`: `{custom}` не похоже на путь к типу", c.name));
            }
            if !(c.x.is_finite() && c.y.is_finite() && c.width >= 0.0 && c.height >= 0.0) {
                return fail(format!("`{}`: неверные координаты или размер", c.name));
            }
            if c.kind != Kind::TabControl && c.selected.is_some_and(|i| i >= c.items.len()) {
                return fail(format!("`{}`: выбранной строки нет в списке", c.name));
            }
            if c.font_size.is_some_and(|size| !(4.0..=200.0).contains(&size)) {
                return fail(format!("`{}`: размер шрифта от 4 до 200", c.name));
            }
            if c.maximum < c.minimum {
                return fail(format!("`{}`: максимум меньше минимума", c.name));
            }
            color(&c.fore_color)?;
            color(&c.back_color)?;
            if let Some(event) = c.events.keys().find(|e| !c.kind.events().contains(e)) {
                return fail(format!(
                    "`{}`: у {} нет события {}",
                    c.name,
                    c.kind.name(),
                    event.name()
                ));
            }
            seen.insert(&c.name, c.kind);
        }
        // Эти имена занимает сгенерированный код.
        if self.controls.iter().any(|c| c.name == "data") {
            return fail("имя `data` занято: в нём лежат данные формы");
        }
        if self.handlers().iter().any(|(name, _)| *name == "create") {
            return fail("имя `create` занято: так создаётся форма");
        }
        let mut signatures: HashMap<&str, Event> = HashMap::new();
        for (name, event) in self.handlers() {
            identifier(name)?;
            if signatures.insert(name, event).is_some_and(|other| other != event) {
                return fail(format!("обработчик `{name}` назначен на события разных типов"));
            }
        }
        Ok(())
    }
}

pub(crate) fn identifier(name: &str) -> Result<(), Error> {
    if name.is_ascii() && syn::parse_str::<syn::Ident>(name).is_ok() {
        Ok(())
    } else {
        fail(format!("`{name}` не годится как имя в Rust (латиница, цифры, _)"))
    }
}

/// Разбирает `#rrggbb`.
pub(crate) fn parse_color(text: &str) -> Result<(u8, u8, u8), Error> {
    let hex = text.strip_prefix('#').filter(|h| h.len() == 6 && h.is_ascii());
    let part = |h: &str, i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    hex.and_then(|h| Some((part(h, 0)?, part(h, 2)?, part(h, 4)?)))
        .ok_or_else(|| Error(format!("цвет `{text}` должен быть в виде #rrggbb")))
}

fn color(value: &Option<String>) -> Result<(), Error> {
    value.as_deref().map_or(Ok(()), |v| parse_color(v).map(|_| ()))
}
