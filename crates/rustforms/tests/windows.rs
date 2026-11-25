use rustforms::{Key, Modifiers, Point, prelude::*};

struct Main {
    menu: MenuStrip,
    result: Label,
    log: Vec<String>,
}

struct Question {
    name: TextBox,
    ok: Button,
    cancel: Button,
}

fn question() -> Application<Question> {
    let state = Question {
        name: TextBox::new().at(0.0, 0.0),
        ok: Button::new("ОК").at(0.0, 40.0),
        cancel: Button::new("Отмена").at(100.0, 40.0),
    };
    let mut view = View::<Question>::new();
    view.add(|f: &mut Question| &mut f.name);
    let ok = view.add(|f: &mut Question| &mut f.ok);
    let cancel = view.add(|f: &mut Question| &mut f.cancel);
    view.on_click(ok, |_, ui, _| ui.accept());
    view.on_click(cancel, |_, ui, _| ui.close());
    Application::new(Ui::new("?", 200.0, 80.0), state, view)
}

fn main_form() -> Application<Main> {
    let mut menu = MenuStrip::new().size(300.0, 26.0).dock(Dock::Top);
    menu.set_items(["Файл/Спросить", "Файл/-", "Файл/Выход", "Справка/О программе"]);
    let state = Main { menu, result: Label::new("").at(0.0, 100.0), log: Vec::new() };
    let mut view = View::<Main>::new();
    let menu = view.add(|f: &mut Main| &mut f.menu);
    view.add(|f: &mut Main| &mut f.result);
    view.on_item_clicked(menu, |me, ui, e| {
        me.log.push(e.item.clone());
        match e.item.as_str() {
            "Файл/Спросить" => {
                ui.show_dialog(question(), |me: &mut Main, dialog: &mut Question, ok, _ui| {
                    let name = if ok { dialog.name.text() } else { "отмена" };
                    me.result.set_text(name);
                });
            }
            "Файл/Выход" => ui.close(),
            _ => {}
        }
    });
    Application::new(Ui::new("main", 460.0, 200.0), state, view)
}

fn click<S: 'static>(app: &mut Application<S>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn menu_opens_selects_and_closes() {
    let mut app = main_form();
    // «Файл», затем пункт «Выход» (после пункта и разделителя).
    click(&mut app, 15.0, 12.0);
    click(&mut app, 30.0, 26.0 + 3.0 + 24.0 + 9.0 + 5.0);
    assert_eq!(app.state.log, ["Файл/Выход"]);
    assert!(app.close_requested());

    // Escape и щелчок мимо закрывают меню без выбора.
    click(&mut app, 15.0, 12.0);
    app.key(Key::Escape, Modifiers::default());
    click(&mut app, 30.0, 34.0);
    click(&mut app, 15.0, 12.0);
    click(&mut app, 250.0, 150.0);
    click(&mut app, 30.0, 34.0);
    assert_eq!(app.state.log.len(), 1);
}

#[test]
fn submenus_open_on_hover_and_report_full_path() {
    let mut app = main_form();
    app.state.menu.set_items(["Файл/Экспорт/Картинка/PNG", "Файл/Экспорт/Текст", "Файл/Выход"]);
    click(&mut app, 15.0, 12.0);
    let row = |n: f32| 26.0 + 3.0 + 24.0 * n + 12.0;
    // «Экспорт» раскрывается наведением, затем «Картинка», затем «PNG».
    app.mouse_move(Point::new(30.0, row(0.0)));
    app.mouse_move(Point::new(180.0, row(0.0)));
    click(&mut app, 320.0, row(0.0));
    assert_eq!(app.state.log, ["Файл/Экспорт/Картинка/PNG"]);

    // Переход на другой пункт сворачивает подменю.
    click(&mut app, 15.0, 12.0);
    app.mouse_move(Point::new(30.0, row(0.0)));
    app.mouse_move(Point::new(30.0, row(1.0)));
    click(&mut app, 30.0, row(1.0));
    assert_eq!(app.state.log[1], "Файл/Выход");
}

#[test]
fn dialog_returns_its_data_to_the_opener() {
    let mut app = main_form();
    for accept in [true, false] {
        click(&mut app, 15.0, 12.0);
        click(&mut app, 30.0, 34.0);
        let mut dialog = app.take_pending().expect("диалог запрошен");
        assert!(dialog.is_modal());
        dialog.state::<Question>().unwrap().name.set_text("Роман");
        if accept {
            dialog.ui().accept();
        } else {
            dialog.ui().close();
        }
        app.finish_pending(dialog);
        assert_eq!(app.state.result.text(), if accept { "Роман" } else { "отмена" });
    }
}

#[test]
fn closing_can_be_cancelled_and_events_fire() {
    struct Form {
        a: Button,
        b: TextBox,
        log: Vec<String>,
        allow: bool,
    }
    let mut state = Form {
        a: Button::new("a").at(0.0, 0.0),
        b: TextBox::new().at(0.0, 40.0),
        log: Vec::new(),
        allow: false,
    };
    state.a.set_tooltip("подсказка");
    state.a.set_tab_index(5);
    state.b.set_read_only(true);
    state.b.set_text("ro");
    let mut view = View::<Form>::new();
    let a = view.add(|f: &mut Form| &mut f.a);
    let b = view.add(|f: &mut Form| &mut f.b);
    view.on_closing(|f, _, e| {
        f.log.push("closing".into());
        if !f.allow {
            e.cancel();
        }
    });
    view.on_resize(|f, _, e| f.log.push(format!("resize {}", e.width)));
    view.on_mouse_enter(a, |f, _, _| f.log.push("enter".into()));
    view.on_mouse_leave(a, |f, _, _| f.log.push("leave".into()));
    view.on_double_click(a, |f, _, e| f.log.push(format!("double {}", e.x)));
    view.on_got_focus(b, |f, _, _| f.log.push("focus b".into()));
    view.on_lost_focus(b, |f, _, _| f.log.push("blur b".into()));
    view.on_key_down(b, |f, _, e| f.log.push(format!("key {}", e.key)));
    let mut app = Application::new(Ui::new("t", 200.0, 100.0), state, view);

    app.mouse_move(Point::new(5.0, 5.0));
    assert!(app.tooltip_deadline().is_some());
    assert!(app.tick(std::time::Instant::now() + std::time::Duration::from_secs(2)));
    for _ in 0..2 {
        app.mouse_down(Modifiers::default());
        app.mouse_up();
    }
    app.mouse_move(Point::new(150.0, 90.0));
    assert_eq!(app.state.log, ["enter", "double 5", "leave"]);

    // tab_index: поле (0) раньше кнопки (5).
    app.state.log.clear();
    // Фокус был на кнопке — последней по порядку, поэтому Tab возвращается к полю.
    app.key(Key::Tab, Modifiers::default());
    assert_eq!(app.focused(), Some(b.id()));
    app.raw_key("F5", Modifiers::default(), true);
    app.text_input("x");
    app.key(Key::Backspace, Modifiers::default());
    assert_eq!(app.state.b.text(), "ro");
    app.key(Key::Tab, Modifiers::default());
    assert_eq!(app.state.log, ["focus b", "key F5", "blur b"]);

    app.state.log.clear();
    app.ui.width = 300.0;
    app.resized();
    app.resized();
    app.ui.close();
    assert!(!app.try_close() && !app.close_requested());
    app.state.allow = true;
    assert!(app.try_close());
    assert_eq!(app.state.log, ["resize 300", "closing", "closing"]);
}

#[test]
fn dialogs_without_a_window_do_not_block() {
    let mut ui = Ui::new("t", 10.0, 10.0);
    ui.message_box("a", "b");
    assert!(!ui.confirm("a", "b"));
    assert!(ui.open_file(&[("x", &["x"])]).is_none());
}
