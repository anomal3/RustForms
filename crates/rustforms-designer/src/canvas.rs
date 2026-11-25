//! Холст: форма, выделение, перетаскивание и изменение размера.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};

use crate::app::{Designer, Request};

pub const GRID: f32 = 8.0;
const TITLE: f32 = 30.0;
const MARGIN: f32 = 24.0;
const KNOB: f32 = 8.0;
const ACCENT: Color32 = Color32::from_rgb(0, 120, 215);

/// Какие стороны тянет ручка: -1 левая/верхняя, 1 правая/нижняя, 0 никакую.
type Sides = (i8, i8);
const KNOBS: [Sides; 8] =
    [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

pub enum Drag {
    Move { name: String, start: (f32, f32) },
    Resize { name: String, sides: Sides, start: (f32, f32, f32, f32) },
    Form { sides: Sides, start: (f32, f32) },
}

fn knob_at(rect: Rect, sides: Sides) -> Rect {
    let pick = |s: i8, low: f32, high: f32| match s {
        -1 => low,
        1 => high,
        _ => (low + high) / 2.0,
    };
    let center = Pos2::new(
        pick(sides.0, rect.left(), rect.right()),
        pick(sides.1, rect.top(), rect.bottom()),
    );
    Rect::from_center_size(center, Vec2::splat(KNOB))
}

fn cursor(sides: Sides) -> CursorIcon {
    match sides {
        (0, _) => CursorIcon::ResizeVertical,
        (_, 0) => CursorIcon::ResizeHorizontal,
        (a, b) if a == b => CursorIcon::ResizeNwSe,
        _ => CursorIcon::ResizeNeSw,
    }
}

impl Designer {
    pub(crate) fn canvas(&mut self, ui: &mut egui::Ui) {
        let Some(open) = &mut self.open else {
            ui.centered_and_justified(|ui| {
                ui.label("Создайте проект или откройте форму (.rfrm)");
            });
            return;
        };
        let document = &mut open.document;
        let size = Vec2::new(document.model.form.width, document.model.form.height);
        let wanted = size + Vec2::new(MARGIN * 2.0, MARGIN * 2.0 + TITLE);
        let (response, painter) =
            ui.allocate_painter(wanted.max(ui.available_size()), Sense::click_and_drag());
        let corner = response.rect.min + Vec2::splat(MARGIN);
        let origin = corner + Vec2::new(0.0, TITLE);
        let client = Rect::from_min_size(origin, size);
        self.origin = origin;

        // Рамка окна вокруг клиентской области.
        let title = Rect::from_min_size(corner, Vec2::new(size.x, TITLE));
        painter.rect_filled(title, 0.0, Color32::from_rgb(32, 32, 40));
        painter.text(
            title.left_center() + Vec2::new(10.0, 0.0),
            Align2::LEFT_CENTER,
            &document.model.form.title,
            FontId::proportional(13.0),
            Color32::WHITE,
        );
        if let Some(texture) = self.preview.texture(ui.ctx(), &document.model) {
            let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            painter.image(texture.id(), client, uv, Color32::WHITE);
        }

        let on_screen = |(x, y, w, h): (f32, f32, f32, f32)| {
            Rect::from_min_size(origin + Vec2::new(x, y), Vec2::new(w, h))
        };
        for control in &document.model.controls {
            let Some(frame) = document.frame(&control.name) else { continue };
            let rect = on_screen(frame);
            if !control.visible {
                let veil = Color32::from_rgba_unmultiplied(255, 255, 255, 170);
                painter.rect_filled(rect, 0.0, veil);
            }
            if let Some(custom) = &control.custom {
                let short = custom.rsplit("::").next().unwrap_or(custom);
                painter.text(
                    rect.right_top(),
                    Align2::RIGHT_BOTTOM,
                    short,
                    FontId::proportional(10.0),
                    ACCENT,
                );
            }
        }

        let selected = document.selected.clone();
        let selection = selected.as_deref().and_then(|n| document.frame(n)).map(on_screen);
        // У формы тянутся только правая и нижняя стороны.
        let knobs: Vec<(Rect, Sides)> = match selection {
            Some(rect) => KNOBS.iter().map(|&s| (knob_at(rect, s), s)).collect(),
            None => [(1, 0), (0, 1), (1, 1)].map(|s| (knob_at(client, s), s)).to_vec(),
        };
        if let Some(rect) = selection {
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, ACCENT), StrokeKind::Outside);
        }
        for (rect, _) in &knobs {
            painter.rect_filled(*rect, 1.0, Color32::WHITE);
            painter.rect_stroke(*rect, 1.0, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
        }

        let (hover, press, alt, released) = ui.input(|i| {
            (
                i.pointer.latest_pos(),
                i.pointer.press_origin(),
                i.modifiers.alt,
                i.pointer.primary_released(),
            )
        });
        let snap = |v: f32| {
            if self.snap && !alt { (v / GRID).round() * GRID } else { v.round() }
        };
        let local = |p: Pos2| (p.x - origin.x, p.y - origin.y);

        if let Some(p) = hover
            && let Some((_, sides)) = knobs.iter().find(|k| k.0.expand(2.0).contains(p))
        {
            ui.ctx().set_cursor_icon(cursor(*sides));
        } else if self.tool.is_some() && response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
        }

        // Инструмент: щелчком по холсту или перетаскиванием из панели.
        if let Some(kind) = self.tool {
            let dropped = released && self.tool_dragged;
            if (response.clicked() || dropped)
                && let Some(p) = hover
                && client.contains(p)
            {
                let (x, y) = local(p);
                document.add(kind, snap(x), snap(y));
                self.tool = None;
            }
            if released {
                self.tool_dragged = false;
            }
            return;
        }

        if response.double_clicked()
            && let Some(p) = response.interact_pointer_pos()
            && client.contains(p)
        {
            let (x, y) = local(p);
            let target = document.hit(x, y, None).map(str::to_owned);
            self.request = Some(Request::Handler { control: target, event: None });
            return;
        }

        if response.clicked() || response.drag_started() {
            let Some(p) = press.or(response.interact_pointer_pos()) else { return };
            let knob = knobs.iter().find(|k| k.0.expand(2.0).contains(p)).map(|k| k.1);
            let drag = if let Some(sides) = knob {
                match (&selected, selected.as_deref().and_then(|n| document.control(n))) {
                    (Some(name), Some(c)) => Some(Drag::Resize {
                        name: name.clone(),
                        sides,
                        start: (c.x, c.y, c.width, c.height),
                    }),
                    _ => Some(Drag::Form { sides, start: (size.x, size.y) }),
                }
            } else {
                let (x, y) = local(p);
                let hit = document.hit(x, y, None).map(str::to_owned);
                document.selected = hit.clone();
                document.end_merge();
                hit.and_then(|name| {
                    let c = document.control(&name)?;
                    Some(Drag::Move { start: (c.x, c.y), name })
                })
            };
            if response.drag_started() && drag.is_some() {
                document.checkpoint(None);
                self.drag = drag;
            }
        }

        if response.dragged()
            && let (Some(drag), Some(now), Some(from)) = (&self.drag, hover, press)
        {
            let delta = now - from;
            match drag {
                Drag::Move { name, start } => {
                    if let Some(c) = document.control_mut(name) {
                        c.x = snap(start.0 + delta.x);
                        c.y = snap(start.1 + delta.y);
                    }
                }
                Drag::Resize { name, sides, start: (x, y, w, h) } => {
                    if let Some(c) = document.control_mut(name) {
                        // Тянутая сторона привязывается к сетке, противоположная стоит.
                        match sides.0 {
                            -1 => {
                                c.x = snap(x + delta.x).min(x + w - KNOB);
                                c.width = x + w - c.x;
                            }
                            1 => c.width = (snap(x + w + delta.x) - x).max(KNOB),
                            _ => {}
                        }
                        match sides.1 {
                            -1 => {
                                c.y = snap(y + delta.y).min(y + h - KNOB);
                                c.height = y + h - c.y;
                            }
                            1 => c.height = (snap(y + h + delta.y) - y).max(KNOB),
                            _ => {}
                        }
                    }
                }
                Drag::Form { sides, start } => {
                    let form = &mut document.model.form;
                    if sides.0 == 1 {
                        form.width = snap(start.0 + delta.x).clamp(80.0, 4000.0);
                    }
                    if sides.1 == 1 {
                        form.height = snap(start.1 + delta.y).clamp(40.0, 4000.0);
                    }
                }
            }
        }

        if response.drag_stopped()
            && let Some(Drag::Move { name, .. }) = self.drag.take()
        {
            document.reparent_by_position(&name);
        }
        if released {
            self.drag = None;
        }
    }
}
