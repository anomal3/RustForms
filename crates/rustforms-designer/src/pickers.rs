//! Выбор Dock и Anchor картинкой, как в WinForms.

use eframe::egui::{self, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use rustforms_model::{Dock, Side};

/// Квадрат с четырьмя перекладинами. Возвращает сторону, по которой щёлкнули.
pub fn anchor(ui: &mut egui::Ui, on: impl Fn(Side) -> bool) -> Option<Side> {
    let (area, _) = ui.allocate_exact_size(vec2(90.0, 90.0), Sense::hover());
    let line = Stroke::new(1.0, ui.visuals().text_color());
    let rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(area.min + vec2(x, y), vec2(w, h));
    ui.painter().rect_stroke(area, 0.0, Stroke::new(1.0, Color32::GRAY), StrokeKind::Inside);
    ui.painter().rect_filled(rect(30.0, 30.0, 30.0, 30.0), 0.0, Color32::GRAY);
    let mut clicked = None;
    for side in Side::ALL {
        let (bar, hit) = match side.name() {
            "left" => (rect(1.0, 42.0, 29.0, 6.0), rect(0.0, 30.0, 30.0, 30.0)),
            "top" => (rect(42.0, 1.0, 6.0, 29.0), rect(30.0, 0.0, 30.0, 30.0)),
            "right" => (rect(60.0, 42.0, 29.0, 6.0), rect(60.0, 30.0, 30.0, 30.0)),
            _ => (rect(42.0, 60.0, 6.0, 29.0), rect(30.0, 60.0, 30.0, 30.0)),
        };
        let response = ui
            .interact(hit, ui.make_persistent_id(("anchor", side.name())), Sense::click())
            .on_hover_text(format!("Anchor: {}", side.name()));
        let active = on(side);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::CheckBox, true, active, side.name())
        });
        if active {
            ui.painter().rect_filled(bar, 0.0, Color32::from_rgb(0x35, 0x74, 0xf0));
        } else {
            ui.painter().rect_stroke(bar, 0.0, line, StrokeKind::Inside);
        }
        if response.clicked() {
            clicked = Some(side);
        }
    }
    clicked
}

/// Схема: края, середина (Fill) и None.
pub fn dock(ui: &mut egui::Ui, current: &mut Dock) {
    let (area, _) = ui.allocate_exact_size(vec2(96.0, 106.0), Sense::hover());
    for side in Dock::ALL {
        let (x, y, w, h, text) = match side {
            Dock::Top => (0.0, 0.0, 96.0, 18.0, ""),
            Dock::Left => (0.0, 20.0, 18.0, 44.0, ""),
            Dock::Fill => (20.0, 20.0, 56.0, 44.0, "Fill"),
            Dock::Right => (78.0, 20.0, 18.0, 44.0, ""),
            Dock::Bottom => (0.0, 66.0, 96.0, 18.0, ""),
            Dock::None => (0.0, 88.0, 96.0, 18.0, "None"),
        };
        let place = Rect::from_min_size(pos2(area.min.x + x, area.min.y + y), vec2(w, h));
        let button = egui::Button::new(text).selected(*current == side).min_size(vec2(w, h));
        let response = ui.put(place, button).on_hover_text(format!("Dock: {}", side.name()));
        if response.clicked() {
            *current = side;
        }
    }
}
