//! Сценарий «собрать форму мышкой» на настоящем интерфейсе, без окна.

use std::fs;

use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::{Harness, kittest::Queryable};
use rustforms_model::{load, new_form};

use crate::app::Designer;

type Screen = Harness<'static, Designer>;

fn button(screen: &mut Screen, at: Pos2, pressed: bool) {
    screen.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
    screen.step();
}

fn hover(screen: &mut Screen, at: Pos2) {
    screen.event(Event::PointerMoved(at));
    screen.step();
}

fn click(screen: &mut Screen, at: Pos2) {
    // Пауза, чтобы соседние клики не склеились в двойной.
    screen.run_steps(30);
    hover(screen, at);
    button(screen, at, true);
    button(screen, at, false);
    screen.step();
}

fn double_click(screen: &mut Screen, at: Pos2) {
    click(screen, at);
    button(screen, at, true);
    button(screen, at, false);
    screen.run_steps(3);
}

fn drag(screen: &mut Screen, from: Pos2, to: Pos2) {
    screen.run_steps(30);
    hover(screen, from);
    button(screen, from, true);
    for step in 1..=5 {
        hover(screen, from + (to - from) * (step as f32 / 5.0));
    }
    button(screen, to, false);
    screen.step();
}

fn tool(screen: &Screen, name: &str) -> Pos2 {
    screen.get_by_label(name).rect().center()
}

#[test]
fn build_a_form_with_the_mouse() {
    let dir = std::env::temp_dir().join(format!("rustforms-ui-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let rfrm = new_form(&dir, "MainForm").unwrap();

    let file = rfrm.clone();
    let mut screen: Screen = Harness::builder()
        .with_size(Vec2::new(1100.0, 700.0))
        .with_step_dt(1.0 / 60.0)
        .build_eframe(move |_| Designer::new(Some(file), false));
    screen.run_steps(3);
    let origin = screen.state().origin;
    let at = |x: f32, y: f32| origin + Vec2::new(x, y);

    // Панель щелчком, кнопка щелчком внутрь неё.
    let p = tool(&screen, "Panel");
    click(&mut screen, p);
    click(&mut screen, at(24.0, 24.0));
    let p = tool(&screen, "Button");
    click(&mut screen, p);
    click(&mut screen, at(40.0, 40.0));
    // Надпись перетаскиванием из списка элементов.
    let p = tool(&screen, "Label");
    drag(&mut screen, p, at(264.0, 40.0));
    // Поле ввода мимо панели, затем перемещение и растягивание за правую ручку.
    let p = tool(&screen, "TextBox");
    click(&mut screen, p);
    click(&mut screen, at(40.0, 200.0));
    drag(&mut screen, at(60.0, 212.0), at(156.0, 236.0));
    drag(&mut screen, at(296.0, 238.0), at(344.0, 238.0));

    {
        let model = &screen.state().open.as_ref().unwrap().document.model;
        let get = |name: &str| model.controls.iter().find(|c| c.name == name).unwrap();
        assert_eq!(model.controls.len(), 4);
        let b = get("button1");
        assert_eq!((b.parent.as_deref(), b.x, b.y), (Some("panel1"), 16.0, 16.0));
        assert_eq!(get("label1").parent, None);
        let t = get("text_box1");
        assert_eq!((t.x, t.y, t.width), (136.0, 224.0, 208.0));
    }

    // Двойной клик по кнопке и по пустому месту формы.
    double_click(&mut screen, at(60.0, 52.0));
    assert!(screen.state().status.contains("main_form.rs"), "{}", screen.state().status);
    double_click(&mut screen, at(440.0, 290.0));

    let model = load(&rfrm).unwrap();
    assert_eq!(model.form.load.as_deref(), Some("main_form_load"));
    let code = fs::read_to_string(dir.join("main_form.rs")).unwrap();
    assert!(code.contains("fn button1_click(") && code.contains("fn main_form_load("));
    let designer = fs::read_to_string(dir.join("main_form.designer.rs")).unwrap();
    assert!(designer.contains("ui.add_to(panel1, control)"));

    // Клавиатура: выбрать, подвинуть стрелкой, удалить, вернуть.
    click(&mut screen, at(160.0, 236.0));
    screen.key_press(Key::ArrowRight);
    screen.run_steps(2);
    screen.key_press(Key::Delete);
    screen.run_steps(2);
    assert_eq!(screen.state().open.as_ref().unwrap().document.model.controls.len(), 3);
    screen.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    screen.run_steps(2);
    let document = &screen.state().open.as_ref().unwrap().document;
    assert_eq!(document.control("text_box1").unwrap().x, 137.0);

    // Снимок для просмотра глазами.
    if let Ok(path) = std::env::var("RUSTFORMS_SNAPSHOT") {
        click(&mut screen, at(60.0, 52.0));
        screen.run_steps(3);
        screen.render().unwrap().save(path).unwrap();
    }
    let _ = fs::remove_dir_all(dir);
}
