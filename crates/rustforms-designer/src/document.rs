//! Редактируемая форма: операции над моделью и отмена. Без привязки к окну.

use std::collections::BTreeSet;

use rustforms::{Anchor, Rect, layout};
use rustforms_model::{Control, Dock, FormModel, Kind, Side};

pub type Frame = (f32, f32, f32, f32);

const HISTORY: usize = 200;

pub struct Document {
    pub model: FormModel,
    pub selected: Option<String>,
    undo: Vec<FormModel>,
    redo: Vec<FormModel>,
    /// Подряд идущие правки с одним ключом отменяются за один раз.
    merge: Option<String>,
    /// Модель после прошлого размещения: якоря считают, насколько изменился родитель.
    laid: FormModel,
}

fn anchor_of(sides: &BTreeSet<Side>) -> Anchor {
    Anchor {
        left: sides.contains(&Side::Left),
        top: sides.contains(&Side::Top),
        right: sides.contains(&Side::Right),
        bottom: sides.contains(&Side::Bottom),
    }
}

fn dock_of(dock: Dock) -> rustforms::Dock {
    match dock {
        Dock::None => rustforms::Dock::None,
        Dock::Top => rustforms::Dock::Top,
        Dock::Bottom => rustforms::Dock::Bottom,
        Dock::Left => rustforms::Dock::Left,
        Dock::Right => rustforms::Dock::Right,
        Dock::Fill => rustforms::Dock::Fill,
    }
}

fn size_of(model: &FormModel, parent: Option<&str>) -> Option<(f32, f32)> {
    match parent {
        None => Some((model.form.width, model.form.height)),
        Some(name) => {
            model.controls.iter().find(|c| c.name == name).map(|c| (c.width, c.height))
        }
    }
}

impl Document {
    pub fn new(model: FormModel) -> Self {
        let laid = model.clone();
        let mut document =
            Self { model, selected: None, undo: Vec::new(), redo: Vec::new(), merge: None, laid };
        document.arrange();
        document
    }

    /// Применяет якоря и Dock после изменений: как в дизайнере WinForms, привязанные
    /// контролы следуют за размером формы и панелей.
    pub fn arrange(&mut self) {
        let mut parents: Vec<Option<String>> = vec![None];
        parents.extend(
            self.model.controls.iter().filter(|c| c.kind == Kind::Panel).map(|c| Some(c.name.clone())),
        );
        for parent in parents {
            let Some(size) = size_of(&self.model, parent.as_deref()) else { continue };
            let old = size_of(&self.laid, parent.as_deref()).unwrap_or(size);
            let (dw, dh) = (size.0 - old.0, size.1 - old.1);
            let mut docked = Vec::new();
            for (i, c) in self.model.controls.iter_mut().enumerate() {
                if c.parent != parent {
                    continue;
                }
                let rect = Rect::new(c.x, c.y, c.width, c.height);
                if c.dock != Dock::None {
                    if c.visible {
                        docked.push((i, (dock_of(c.dock), rect)));
                    }
                } else if (dw, dh) != (0.0, 0.0)
                    // Только что перенесённый или добавленный не двигается.
                    && self.laid.controls.iter().any(|o| o.name == c.name && o.parent == c.parent)
                {
                    let moved = layout::anchored(rect, anchor_of(&c.anchor), dw, dh);
                    (c.x, c.y) = (moved.x.round(), moved.y.round());
                    (c.width, c.height) = (moved.width.max(1.0), moved.height.max(1.0));
                }
            }
            let mut rects: Vec<_> = docked.iter().map(|d| d.1).collect();
            layout::dock(size, &mut rects);
            for ((i, _), (_, r)) in docked.into_iter().zip(rects) {
                let c = &mut self.model.controls[i];
                (c.x, c.y, c.width, c.height) = (r.x, r.y, r.width.max(1.0), r.height.max(1.0));
            }
        }
        self.laid = self.model.clone();
    }

    /// Заменяет модель целиком (отмена, чтение с диска): якоря не применяются.
    pub fn reset(&mut self, model: FormModel) {
        self.model = model;
        self.laid = self.model.clone();
        self.arrange();
    }

    /// Запомнить состояние перед правкой.
    pub fn checkpoint(&mut self, merge: Option<&str>) {
        if merge.is_some() && merge == self.merge.as_deref() {
            return;
        }
        self.merge = merge.map(str::to_owned);
        if self.undo.last() != Some(&self.model) {
            self.undo.push(self.model.clone());
            if self.undo.len() > HISTORY {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
    }

    pub fn end_merge(&mut self) {
        self.merge = None;
    }

    /// Правка с проверкой: если модель становится неверной, ничего не меняется.
    pub fn edit(&mut self, change: impl FnOnce(&mut FormModel)) -> Result<(), String> {
        let mut next = self.model.clone();
        change(&mut next);
        if next == self.model {
            return Ok(());
        }
        next.validate().map_err(|e| e.to_string())?;
        self.checkpoint(None);
        self.model = next;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.undo.iter().any(|m| m != &self.model)
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        // Снимки, не отличающиеся от текущего (клик без перемещения), пропускаются.
        while let Some(previous) = self.undo.pop() {
            if previous != self.model {
                self.redo.push(std::mem::replace(&mut self.model, previous));
                break;
            }
        }
        self.after_history();
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.model, next));
        }
        self.after_history();
    }

    fn after_history(&mut self) {
        self.merge = None;
        self.laid = self.model.clone();
        if self.selected.as_deref().is_some_and(|n| self.control(n).is_none()) {
            self.selected = None;
        }
    }

    pub fn control(&self, name: &str) -> Option<&Control> {
        self.model.controls.iter().find(|c| c.name == name)
    }

    pub fn control_mut(&mut self, name: &str) -> Option<&mut Control> {
        self.model.controls.iter_mut().find(|c| c.name == name)
    }

    /// Положение в координатах формы.
    pub fn frame(&self, name: &str) -> Option<Frame> {
        let c = self.control(name)?;
        let (px, py) = self.origin(c.parent.as_deref());
        Some((px + c.x, py + c.y, c.width, c.height))
    }

    fn origin(&self, parent: Option<&str>) -> (f32, f32) {
        parent.and_then(|p| self.frame(p)).map_or((0.0, 0.0), |f| (f.0, f.1))
    }

    /// Верхний контрол под точкой; `skip` и его дети не учитываются.
    pub fn hit(&self, x: f32, y: f32, skip: Option<&str>) -> Option<&str> {
        let skipped = skip.map(|s| self.subtree(s)).unwrap_or_default();
        self.model
            .controls
            .iter()
            .rev()
            .filter(|c| !skipped.contains(&c.name))
            .find(|c| {
                let mut inside = true;
                let mut name = Some(c.name.as_str());
                // Часть, выступающая за родителя, не видна и не кликается.
                while let Some(n) = name {
                    let (fx, fy, w, h) = self.frame(n).unwrap_or_default();
                    inside &= x >= fx && y >= fy && x < fx + w && y < fy + h;
                    name = self.control(n).and_then(|c| c.parent.as_deref());
                }
                inside
            })
            .map(|c| c.name.as_str())
    }

    fn panel_at(&self, x: f32, y: f32, skip: Option<&str>) -> Option<String> {
        let mut name = self.hit(x, y, skip);
        while let Some(n) = name {
            let c = self.control(n)?;
            if c.kind == Kind::Panel {
                return Some(n.to_owned());
            }
            name = c.parent.as_deref();
        }
        None
    }

    /// Контрол и все его потомки (в порядке объявления).
    pub fn subtree(&self, name: &str) -> Vec<String> {
        let mut names = vec![name.to_owned()];
        for c in &self.model.controls {
            if c.parent.as_ref().is_some_and(|p| names.contains(p)) {
                names.push(c.name.clone());
            }
        }
        names
    }

    fn take_subtree(&mut self, name: &str) -> Vec<Control> {
        let names = self.subtree(name);
        let (taken, rest) = std::mem::take(&mut self.model.controls)
            .into_iter()
            .partition(|c| names.contains(&c.name));
        self.model.controls = rest;
        taken
    }

    /// Новый контрол в точке формы; если там панель — внутрь неё.
    pub fn add(&mut self, kind: Kind, x: f32, y: f32) -> String {
        self.checkpoint(None);
        let parent = self.panel_at(x, y, None);
        let (px, py) = self.origin(parent.as_deref());
        let name = self.model.free_name(kind);
        let mut control = Control::new(kind, name.clone(), x - px, y - py);
        control.parent = parent;
        self.model.controls.push(control);
        self.selected = Some(name.clone());
        name
    }

    /// Переносит контрол в панель, оказавшуюся под его центром, сохраняя место на экране.
    pub fn reparent_by_position(&mut self, name: &str) {
        let Some((x, y, w, h)) = self.frame(name) else { return };
        let parent = self.panel_at(x + w / 2.0, y + h / 2.0, Some(name));
        if self.control(name).is_none_or(|c| c.parent == parent) {
            return;
        }
        let (px, py) = self.origin(parent.as_deref());
        // В конец списка: так родитель гарантированно объявлен раньше.
        let mut moved = self.take_subtree(name);
        if let Some(c) = moved.iter_mut().find(|c| c.name == name) {
            c.parent = parent;
            c.x = x - px;
            c.y = y - py;
        }
        self.model.controls.append(&mut moved);
    }

    pub fn delete(&mut self, name: &str) {
        self.checkpoint(None);
        self.take_subtree(name);
        self.selected = None;
    }

    pub fn rename(&mut self, old: &str, new: &str) -> Result<(), String> {
        self.edit(|model| {
            for c in &mut model.controls {
                if c.name == old {
                    c.name = new.to_owned();
                }
                if c.parent.as_deref() == Some(old) {
                    c.parent = Some(new.to_owned());
                }
            }
        })?;
        if self.selected.as_deref() == Some(old) {
            self.selected = Some(new.to_owned());
        }
        Ok(())
    }

    pub fn bring_to_front(&mut self, name: &str) {
        self.checkpoint(None);
        let mut moved = self.take_subtree(name);
        self.model.controls.append(&mut moved);
    }

    pub fn send_to_back(&mut self, name: &str) {
        self.checkpoint(None);
        let parent = self.control(name).and_then(|c| c.parent.clone());
        let moved = self.take_subtree(name);
        let at = parent
            .and_then(|p| self.model.controls.iter().position(|c| c.name == p))
            .map_or(0, |i| i + 1);
        self.model.controls.splice(at..at, moved);
    }

    /// Контрол с потомками в виде текста для буфера обмена.
    pub fn copy(&self, name: &str) -> Option<String> {
        let names = self.subtree(name);
        let mut part = FormModel::new("Clipboard");
        part.controls =
            self.model.controls.iter().filter(|c| names.contains(&c.name)).cloned().collect();
        let first = part.controls.iter_mut().find(|c| c.name == name)?;
        first.parent = None;
        part.to_toml().ok()
    }

    /// Вставляет скопированное с новыми именами. Обработчики не переносятся.
    pub fn paste(&mut self, text: &str) -> bool {
        let Ok(part) = FormModel::from_toml(text) else { return false };
        if part.form.name != "Clipboard" || part.controls.is_empty() {
            return false;
        }
        self.checkpoint(None);
        let target = self
            .selected
            .as_deref()
            .and_then(|n| self.control(n))
            .and_then(|c| c.parent.clone());
        let mut renamed: Vec<(String, String)> = Vec::new();
        for mut c in part.controls {
            let old = c.name.clone();
            if self.control(&old).is_some() {
                c.name = self.model.free_name(c.kind);
            }
            match &c.parent {
                Some(p) => {
                    c.parent = renamed.iter().find(|r| &r.0 == p).map(|r| r.1.clone());
                }
                None => {
                    c.parent = target.clone();
                    c.x += 16.0;
                    c.y += 16.0;
                    self.selected = Some(c.name.clone());
                }
            }
            c.events.clear();
            renamed.push((old, c.name.clone()));
            self.model.controls.push(c);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Document {
        Document::new(FormModel::new("F"))
    }

    #[test]
    fn controls_dropped_on_a_panel_become_its_children() {
        let mut d = document();
        let button = d.add(Kind::Button, 300.0, 200.0);
        let panel = d.add(Kind::Panel, 10.0, 10.0);
        let inside = d.add(Kind::Label, 20.0, 30.0);
        assert_eq!(d.control(&inside).unwrap().parent.as_deref(), Some(panel.as_str()));
        assert_eq!(d.frame(&inside).unwrap().0, 20.0);

        let c = d.control_mut(&button).unwrap();
        (c.x, c.y) = (40.0, 50.0);
        d.reparent_by_position(&button);
        assert_eq!(d.control(&button).unwrap().parent.as_deref(), Some(panel.as_str()));
        assert_eq!(d.frame(&button).unwrap().0, 40.0);
        d.model.validate().unwrap();

        d.send_to_back(&button);
        d.model.validate().unwrap();
        d.delete(&panel);
        assert!(d.model.controls.is_empty());
        d.undo();
        assert_eq!(d.model.controls.len(), 3);
    }

    #[test]
    fn anchored_and_docked_controls_follow_the_form() {
        let mut d = document();
        let right = d.add(Kind::Button, 300.0, 200.0);
        d.control_mut(&right).unwrap().anchor = [Side::Right, Side::Bottom].into();
        let wide = d.add(Kind::TextBox, 8.0, 8.0);
        d.control_mut(&wide).unwrap().anchor = [Side::Left, Side::Top, Side::Right].into();
        let bar = d.add(Kind::Panel, 8.0, 100.0);
        d.control_mut(&bar).unwrap().dock = Dock::Bottom;
        d.arrange();
        let b = d.control(&bar).unwrap();
        assert_eq!((b.x, b.y, b.width, b.height), (0.0, 220.0, 480.0, 100.0));

        d.checkpoint(None);
        d.model.form.width += 100.0;
        d.model.form.height += 40.0;
        d.arrange();
        assert_eq!((d.control(&right).unwrap().x, d.control(&right).unwrap().y), (400.0, 240.0));
        assert_eq!(d.control(&wide).unwrap().width, 260.0);
        assert_eq!(d.control(&bar).unwrap().width, 580.0);

        d.undo();
        d.arrange();
        assert_eq!(d.control(&right).unwrap().x, 300.0);
    }

    #[test]
    fn rename_checks_names_and_keeps_children() {
        let mut d = document();
        let panel = d.add(Kind::Panel, 10.0, 10.0);
        let child = d.add(Kind::Button, 20.0, 20.0);
        assert!(d.rename(&child, &panel).is_err());
        assert!(d.rename(&child, "2bad").is_err());
        d.rename(&panel, "box_panel").unwrap();
        assert_eq!(d.control(&child).unwrap().parent.as_deref(), Some("box_panel"));
    }

    #[test]
    fn paste_copies_the_whole_panel_with_new_names() {
        let mut d = document();
        let panel = d.add(Kind::Panel, 10.0, 10.0);
        d.add(Kind::Button, 20.0, 20.0);
        let text = d.copy(&panel).unwrap();
        d.selected = None;
        assert!(d.paste(&text));
        assert!(!d.paste("обычный текст"));
        assert_eq!(d.model.controls.len(), 4);
        assert_eq!(d.model.controls[3].parent.as_deref(), Some("panel2"));
        d.model.validate().unwrap();
        d.undo();
        assert_eq!(d.model.controls.len(), 2);
        d.redo();
        assert_eq!(d.model.controls.len(), 4);
    }
}
