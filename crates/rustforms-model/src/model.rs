use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{Error, fail};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Button,
    Label,
    TextBox,
    CheckBox,
    Panel,
}

impl Kind {
    pub const ALL: [Kind; 5] =
        [Kind::Button, Kind::Label, Kind::TextBox, Kind::CheckBox, Kind::Panel];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Button => "Button",
            Kind::Label => "Label",
            Kind::TextBox => "TextBox",
            Kind::CheckBox => "CheckBox",
            Kind::Panel => "Panel",
        }
    }

    pub fn default_size(self) -> (f32, f32) {
        match self {
            Kind::Button => (90.0, 28.0),
            Kind::Label => (120.0, 22.0),
            Kind::TextBox => (160.0, 28.0),
            Kind::CheckBox => (140.0, 22.0),
            Kind::Panel => (200.0, 100.0),
        }
    }

    /// События контрола; первое создаётся двойным кликом.
    pub fn events(self) -> &'static [Event] {
        match self {
            Kind::Button | Kind::Label => &[Event::Click],
            Kind::TextBox => &[Event::TextChanged],
            Kind::CheckBox => &[Event::CheckedChanged, Event::Click],
            Kind::Panel => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Load,
    Click,
    TextChanged,
    CheckedChanged,
}

impl Event {
    pub fn name(self) -> &'static str {
        match self {
            Event::Load => "load",
            Event::Click => "click",
            Event::TextChanged => "text_changed",
            Event::CheckedChanged => "checked_changed",
        }
    }

    pub fn args(self) -> &'static str {
        match self {
            Event::Load => "LoadArgs",
            Event::Click => "ClickArgs",
            Event::TextChanged => "TextChangedArgs",
            Event::CheckedChanged => "CheckedChangedArgs",
        }
    }
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
    /// Имя обработчика загрузки.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<String>,
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
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub visible: bool,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub enabled: bool,
    /// Только `CheckBox`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub checked: bool,
    /// Только `Label`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fore_color: Option<String>,
    /// Только `Panel`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_color: Option<String>,
    /// Только `Panel`.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub border: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub events: BTreeMap<Event, String>,
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
        let text = matches!(kind, Kind::Button | Kind::Label | Kind::CheckBox).then(|| name.clone());
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
            visible: true,
            enabled: true,
            checked: false,
            fore_color: None,
            back_color: None,
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
            Kind::Panel => "panel",
        };
        (1..)
            .map(|n| format!("{prefix}{n}"))
            .find(|name| self.controls.iter().all(|c| &c.name != name))
            .expect("свободное имя всегда найдётся")
    }

    /// Обработчики формы вместе с типом события.
    pub fn handlers(&self) -> Vec<(&str, Event)> {
        let load = self.form.load.iter().map(|name| (name.as_str(), Event::Load));
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
            if let Some(parent) = &c.parent
                && seen.get(parent.as_str()) != Some(&Kind::Panel)
            {
                return fail(format!(
                    "`{}`: родитель `{parent}` должен быть панелью, объявленной выше",
                    c.name
                ));
            }
            if let Some(custom) = &c.custom
                && syn::parse_str::<syn::Path>(custom).is_err()
            {
                return fail(format!("`{}`: `{custom}` не похоже на путь к типу", c.name));
            }
            if !(c.x.is_finite() && c.y.is_finite() && c.width >= 0.0 && c.height >= 0.0) {
                return fail(format!("`{}`: неверные координаты или размер", c.name));
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
