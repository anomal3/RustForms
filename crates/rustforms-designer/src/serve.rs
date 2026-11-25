//! Фоновый режим для плагинов IDE: дизайнер без окна. Плагин рисует интерфейс своими
//! средствами и шлёт команды — по одному JSON в строке stdin, ответ — строка в stdout.
//! Каждое изменение сразу записывается на диск.

use std::{
    fs,
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use base64::Engine;
use rustforms_model::{Dock, Event, FormModel, Kind, Side, attach_handler, load, save};
use serde_json::{Map, Value, json};

use crate::{document::Document, render::Renderer};

pub struct Server {
    path: PathBuf,
    document: Document,
    renderer: Renderer,
    scale: f32,
    /// Содержимое .rfrm, каким мы его видели: отличие значит, что файл правили снаружи.
    disk: String,
}

fn event_named(name: &str) -> Option<Event> {
    Event::ALL
        .into_iter()
        .find(|e| e.name() == name)
}

impl Server {
    pub fn open(path: &Path) -> Result<Self, String> {
        let path = std::path::absolute(path).map_err(|e| e.to_string())?;
        let model = load(&path).map_err(|e| e.to_string())?;
        let disk = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        Ok(Self {
            path,
            document: Document::new(model),
            renderer: Renderer::default(),
            scale: 1.0,
            disk,
        })
    }

    fn reload_if_changed(&mut self) -> Result<(), String> {
        let now = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        if now != self.disk {
            let model = load(&self.path).map_err(|e| e.to_string())?;
            self.document.checkpoint(None);
            self.document.reset(model);
            self.disk = now;
        }
        Ok(())
    }

    fn persist(&mut self) -> Result<(), String> {
        save(&self.path, &self.document.model).map_err(|e| format!("Не сохранено: {e}"))?;
        self.disk = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Выполняет команду и возвращает ответ вместе с новым состоянием.
    pub fn handle(&mut self, request: &Value) -> Value {
        let mut reply = Map::new();
        let result = self.reload_if_changed().and_then(|()| self.apply(request, &mut reply));
        if let Err(message) = result {
            reply.insert("error".into(), message.into());
        }
        reply.insert("state".into(), self.state());
        Value::Object(reply)
    }

    fn apply(&mut self, request: &Value, reply: &mut Map<String, Value>) -> Result<(), String> {
        let text = |key: &str| request.get(key).and_then(Value::as_str);
        let number = |key: &str| {
            request.get(key).and_then(Value::as_f64).map(|v| v as f32).filter(|v| v.is_finite())
        };
        let need = |value: Option<f32>| value.ok_or("не хватает числа в команде");
        let name = text("name").map(str::to_owned);
        let existing = || {
            name.clone()
                .filter(|n| self.document.control(n).is_some())
                .ok_or_else(|| "такого контрола нет".to_owned())
        };
        if let Some(scale) = number("scale") {
            self.scale = scale.clamp(0.5, 4.0);
        }
        let op = text("op").unwrap_or_default();
        let before = self.document.model.clone();
        match op {
            "state" => {}
            "add" => {
                let kind = Kind::ALL
                    .into_iter()
                    .find(|k| Some(k.name()) == text("kind"))
                    .ok_or("неизвестный тип контрола")?;
                let added = self.document.add(kind, need(number("x"))?, need(number("y"))?);
                reply.insert("select".into(), added.into());
            }
            "bounds" => {
                let target = existing()?;
                let (x, y) = (need(number("x"))?, need(number("y"))?);
                let (w, h) = (need(number("width"))?, need(number("height"))?);
                self.document.checkpoint(None);
                if let Some(c) = self.document.control_mut(&target) {
                    (c.x, c.y, c.width, c.height) = (x, y, w.max(1.0), h.max(1.0));
                }
                // После перемещения контрол может оказаться в другой панели.
                if request.get("moved").and_then(Value::as_bool) == Some(true) {
                    self.document.reparent_by_position(&target);
                }
            }
            "formSize" => {
                let (w, h) = (need(number("width"))?, need(number("height"))?);
                self.document.edit(|m| {
                    m.form.width = w.clamp(80.0, 4000.0);
                    m.form.height = h.clamp(40.0, 4000.0);
                })?;
            }
            "set" => {
                let property = text("property").ok_or("не указано свойство")?;
                let value = request.get("value").cloned().unwrap_or(Value::Null);
                match name.as_deref() {
                    Some(old) if property == "name" => {
                        let new = value.as_str().unwrap_or_default();
                        existing()?;
                        self.document.rename(old, new)?;
                        reply.insert("select".into(), new.into());
                    }
                    Some(_) => {
                        let target = existing()?;
                        self.set_control(&target, property, &value)?;
                    }
                    None => self.set_form(property, &value)?,
                }
            }
            "delete" => {
                let target = existing()?;
                self.document.delete(&target);
            }
            "front" => {
                let target = existing()?;
                self.document.bring_to_front(&target);
            }
            "back" => {
                let target = existing()?;
                self.document.send_to_back(&target);
            }
            "undo" => self.document.undo(),
            "redo" => self.document.redo(),
            "copy" => {
                let target = existing()?;
                reply.insert("text".into(), self.document.copy(&target).into());
            }
            "paste" => {
                self.document.selected = name.clone();
                if self.document.paste(text("text").unwrap_or_default()) {
                    reply.insert("select".into(), self.document.selected.clone().into());
                }
            }
            "handler" => {
                let event = match text("event") {
                    None => None,
                    Some(e) => Some(event_named(e).ok_or("неизвестное событие")?),
                };
                self.persist()?;
                let at = attach_handler(&self.path, name.as_deref(), event)
                    .map_err(|e| e.to_string())?;
                self.document.checkpoint(None);
                self.document.reset(load(&self.path).map_err(|e| e.to_string())?);
                self.disk = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
                reply.insert(
                    "open".into(),
                    json!({ "file": at.file.display().to_string(), "line": at.line }),
                );
            }
            other => return Err(format!("неизвестная команда `{other}`")),
        }
        self.document.arrange();
        if self.document.model != before && op != "handler" {
            self.persist()?;
        }
        Ok(())
    }

    fn set_form(&mut self, property: &str, value: &Value) -> Result<(), String> {
        let text = value.as_str().unwrap_or_default().to_owned();
        let optional = (!text.is_empty()).then(|| text.clone());
        let number = value.as_f64().map(|v| v as f32);
        let mut known = true;
        self.document.edit(|m: &mut FormModel| match property {
            "name" => m.form.name = text,
            "title" => m.form.title = text,
            "width" => m.form.width = number.map_or(m.form.width, |v| v.clamp(80.0, 4000.0)),
            "height" => m.form.height = number.map_or(m.form.height, |v| v.clamp(40.0, 4000.0)),
            "back_color" => m.form.back_color = optional,
            "load" => m.form.load = optional,
            _ => known = false,
        })?;
        known.then_some(()).ok_or_else(|| format!("у формы нет свойства `{property}`"))
    }

    fn set_control(&mut self, name: &str, property: &str, value: &Value) -> Result<(), String> {
        let text = value.as_str().unwrap_or_default().to_owned();
        let optional = (!text.is_empty()).then(|| text.clone());
        let number = value.as_f64().map(|v| v as f32).filter(|v| v.is_finite());
        let flag = value.as_bool();
        let list: Option<Vec<String>> = value
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect());
        let event = property.strip_prefix("event:").and_then(event_named);
        let mut known = true;
        self.document.edit(|m| {
            let Some(c) = m.controls.iter_mut().find(|c| c.name == name) else { return };
            match property {
                "text" => c.text = Some(text),
                "x" => c.x = number.unwrap_or(c.x),
                "y" => c.y = number.unwrap_or(c.y),
                "width" => c.width = number.map_or(c.width, |v| v.max(1.0)),
                "height" => c.height = number.map_or(c.height, |v| v.max(1.0)),
                "visible" => c.visible = flag.unwrap_or(c.visible),
                "enabled" => c.enabled = flag.unwrap_or(c.enabled),
                "checked" => c.checked = flag.unwrap_or(c.checked),
                "multiline" => c.multiline = flag.unwrap_or(c.multiline),
                "items" => {
                    // Массивом или строкой с разделителями `;` и переводом строки.
                    c.items = match list.as_ref() {
                        Some(items) => items.clone(),
                        None => text
                            .split([';', '\n'])
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .collect(),
                    };
                    c.selected = c.selected.filter(|&i| i < c.items.len());
                }
                "selected" => {
                    c.selected = number.filter(|&v| v >= 0.0).map(|v| v as usize);
                }
                "value" => c.value = number.map_or(c.value, |v| v as i32),
                "minimum" => c.minimum = number.map_or(c.minimum, |v| v as i32),
                "maximum" => c.maximum = number.map_or(c.maximum, |v| v as i32),
                "border" => c.border = flag.unwrap_or(c.border),
                "fore_color" => c.fore_color = optional,
                "back_color" => c.back_color = optional,
                "custom" => c.custom = optional,
                "dock" => match Dock::ALL.into_iter().find(|d| d.name() == text) {
                    Some(dock) => c.dock = dock,
                    None => known = false,
                },
                _ if property.starts_with("anchor:") => {
                    match Side::ALL.into_iter().find(|s| property.ends_with(s.name())) {
                        Some(side) if flag == Some(true) => {
                            c.anchor.insert(side);
                        }
                        Some(side) => {
                            c.anchor.remove(&side);
                        }
                        None => known = false,
                    }
                }
                _ => match (event, optional) {
                    (Some(event), Some(handler)) => {
                        c.events.insert(event, handler);
                    }
                    (Some(event), None) => {
                        c.events.remove(&event);
                    }
                    (None, _) => known = false,
                },
            }
        })?;
        known.then_some(()).ok_or_else(|| format!("нет свойства `{property}`"))
    }

    fn state(&mut self) -> Value {
        let model = &self.document.model;
        let controls: Vec<Value> = model
            .controls
            .iter()
            .map(|c| {
                let (x, y, w, h) = self.document.frame(&c.name).unwrap_or_default();
                let events: Vec<Value> = c
                    .kind
                    .events()
                    .iter()
                    .map(|e| json!({ "event": e.name(), "handler": c.events.get(e) }))
                    .collect();
                json!({
                    "name": c.name, "kind": c.kind.name(), "parent": c.parent, "custom": c.custom,
                    "x": c.x, "y": c.y, "width": c.width, "height": c.height,
                    "frame": [x, y, w, h], "text": c.text, "visible": c.visible,
                    "dock": c.dock.name(), "multiline": c.multiline, "items": c.items,
                    "selected": c.selected, "value": c.value, "minimum": c.minimum,
                    "maximum": c.maximum,
                    "anchor": c.anchor.iter().map(|s| s.name()).collect::<Vec<_>>(),
                    "enabled": c.enabled, "checked": c.checked, "border": c.border,
                    "fore_color": c.fore_color, "back_color": c.back_color, "events": events,
                })
            })
            .collect();
        let form = &model.form;
        let image = self
            .renderer
            .render(model, self.scale)
            .and_then(|p| p.encode_png().ok())
            .map(|png| base64::engine::general_purpose::STANDARD.encode(png));
        json!({
            "form": {
                "name": form.name, "title": form.title, "width": form.width,
                "height": form.height, "back_color": form.back_color, "load": form.load,
            },
            "controls": controls,
            "canUndo": self.document.can_undo(),
            "canRedo": self.document.can_redo(),
            "image": image,
        })
    }
}

pub fn run(path: &Path) -> ExitCode {
    let mut server = match Server::open(path) {
        Ok(server) => server,
        Err(message) => {
            println!("{}", json!({ "error": message }));
            return ExitCode::FAILURE;
        }
    };
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        // .NET может начать поток с метки порядка байтов.
        let reply = match serde_json::from_str::<Value>(line.trim_start_matches('\u{feff}')) {
            Ok(request) => server.handle(&request),
            Err(error) => json!({ "error": error.to_string() }),
        };
        // Плагин закрыл канал — работать больше не для кого.
        if writeln!(out, "{reply}").and_then(|()| out.flush()).is_err() {
            break;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustforms_model::new_form;

    #[test]
    fn commands_edit_files_and_report_state() {
        let dir = std::env::temp_dir().join(format!("rustforms-serve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let rfrm = new_form(&dir, "MainForm").unwrap();
        let mut server = Server::open(&rfrm).unwrap();
        let mut send = |request: Value| server.handle(&request);

        send(json!({"op": "add", "kind": "Panel", "x": 16, "y": 16}));
        let reply = send(json!({"op": "add", "kind": "Button", "x": 32, "y": 32}));
        assert_eq!(reply["select"], "button1");
        assert_eq!(reply["state"]["controls"][1]["parent"], "panel1");
        assert!(reply["state"]["image"].as_str().unwrap().starts_with("iVBOR"));

        let reply = send(json!({"op": "set", "name": "button1", "property": "name", "value": "ok"}));
        assert_eq!(reply["select"], "ok");
        let reply = send(json!({"op": "set", "name": "ok", "property": "name", "value": "panel1"}));
        assert!(reply["error"].is_string());

        let reply = send(json!({"op": "handler", "name": "ok"}));
        assert!(reply["open"]["file"].as_str().unwrap().ends_with("main_form.rs"));
        let code = fs::read_to_string(dir.join("main_form.designer.rs")).unwrap();
        assert!(code.contains("view.on_click(ok_id, Self::ok_click);"));

        send(json!({"op": "bounds", "name": "ok", "x": 300, "y": 200, "width": 90,
                    "height": 28, "moved": true}));
        let reply = send(json!({"op": "undo"}));
        assert_eq!(reply["state"]["controls"][1]["x"], 16.0);

        send(json!({"op": "set", "name": "ok", "property": "anchor:right", "value": true}));
        send(json!({"op": "set", "name": "ok", "property": "anchor:left", "value": false}));
        let reply = send(json!({"op": "formSize", "width": 580, "height": 320}));
        assert_eq!(reply["state"]["controls"][1]["x"], 16.0, "кнопка в панели не двигается");
        let reply = send(json!({"op": "set", "name": "panel1", "property": "dock", "value": "fill"}));
        assert_eq!(reply["state"]["controls"][0]["width"], 580.0);
        // Панель растянулась на 380 — правый якорь внутри неё отработал.
        assert_eq!(reply["state"]["controls"][1]["x"], 396.0);
        let code = fs::read_to_string(dir.join("main_form.designer.rs")).unwrap();
        assert!(code.contains("set_dock(Dock::Fill)") && code.contains("set_anchor(Anchor::TOP | Anchor::RIGHT)"));

        // Правка .rfrm снаружи подхватывается.
        let text = fs::read_to_string(&rfrm).unwrap().replace("title = \"MainForm\"", "title = \"Z\"");
        fs::write(&rfrm, text).unwrap();
        assert_eq!(send(json!({"op": "state"}))["state"]["form"]["title"], "Z");
        fs::remove_dir_all(dir).unwrap();
    }
}
