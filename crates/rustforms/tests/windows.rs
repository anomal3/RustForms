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
    Application::new(Ui::new("main", 300.0, 200.0), state, view)
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
fn dialogs_without_a_window_do_not_block() {
    let mut ui = Ui::new("t", 10.0, 10.0);
    ui.message_box("a", "b");
    assert!(!ui.confirm("a", "b"));
    assert!(ui.open_file(&[("x", &["x"])]).is_none());
}
