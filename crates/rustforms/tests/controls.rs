use rustforms::{Key, Modifiers, Point, prelude::*};

struct Form {
    group: Panel,
    first: RadioButton,
    second: RadioButton,
    outside: RadioButton,
    list: ListBox,
    combo: ComboBox,
    under: Button,
    memo: TextBox,
    bar: ProgressBar,
    log: Vec<String>,
}

fn form() -> Application<Form> {
    let mut state = Form {
        group: Panel::new().at(0.0, 0.0).size(200.0, 60.0),
        first: RadioButton::new("1").at(5.0, 5.0),
        second: RadioButton::new("2").at(5.0, 30.0),
        outside: RadioButton::new("3").at(210.0, 5.0),
        list: ListBox::new().at(0.0, 70.0).size(100.0, 46.0),
        combo: ComboBox::new().at(110.0, 70.0).size(100.0, 28.0),
        under: Button::new("под списком").at(110.0, 100.0).size(100.0, 28.0),
        memo: TextBox::new().at(0.0, 220.0).size(300.0, 70.0),
        bar: ProgressBar::new().at(0.0, 300.0),
        log: Vec::new(),
    };
    state.list.set_items(["a", "b", "c", "d"]);
    state.combo.set_items(["x", "y", "z"]);
    state.memo.set_multiline(true);
    let mut view = View::<Form>::new();
    let group = view.add(|f: &mut Form| &mut f.group);
    let first = view.add_to(group, |f: &mut Form| &mut f.first);
    let second = view.add_to(group, |f: &mut Form| &mut f.second);
    view.add(|f: &mut Form| &mut f.outside);
    let list = view.add(|f: &mut Form| &mut f.list);
    let combo = view.add(|f: &mut Form| &mut f.combo);
    let under = view.add(|f: &mut Form| &mut f.under);
    view.add(|f: &mut Form| &mut f.memo);
    view.add(|f: &mut Form| &mut f.bar);
    view.on_checked_changed(first, |f, _, e| f.log.push(format!("first {}", e.checked)));
    view.on_checked_changed(second, |f, _, e| f.log.push(format!("second {}", e.checked)));
    view.on_selected_changed(list, |f, _, e| f.log.push(format!("list {}", e.text)));
    view.on_selected_changed(combo, |f, _, e| f.log.push(format!("combo {}", e.text)));
    view.on_click(under, |f, _, _| f.log.push("under".into()));
    Application::new(Ui::new("t", 320.0, 330.0), state, view)
}

fn click(app: &mut Application<Form>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn radio_buttons_are_exclusive_within_their_parent() {
    let mut app = form();
    app.state.outside.set_checked(true);
    click(&mut app, 12.0, 15.0);
    click(&mut app, 12.0, 40.0);
    assert!(!app.state.first.checked() && app.state.second.checked());
    assert!(app.state.outside.checked(), "другая группа не затронута");
    assert_eq!(app.state.log, ["first true", "first false", "second true"]);
}

#[test]
fn list_box_selects_by_mouse_keys_and_scrolls() {
    let mut app = form();
    click(&mut app, 20.0, 100.0);
    assert_eq!(app.state.list.selected_text(), Some("b"));
    app.key(Key::End, Modifiers::default());
    assert_eq!(app.state.list.selected_text(), Some("d"));
    app.mouse_move(Point::new(20.0, 80.0));
    app.wheel(10.0);
    click(&mut app, 20.0, 80.0);
    assert_eq!(app.state.log, ["list b", "list d", "list a"]);
}

#[test]
fn combo_popup_covers_controls_below_it() {
    let mut app = form();
    click(&mut app, 150.0, 84.0);
    assert!(app.state.combo.is_open());
    // Вторая строка списка лежит поверх кнопки.
    click(&mut app, 150.0, 98.0 + 1.0 + 22.0 + 5.0);
    assert!(!app.state.combo.is_open());
    assert_eq!(app.state.log, ["combo y"]);

    // Щелчок мимо закрывает и нажимает то, по чему щёлкнули.
    click(&mut app, 150.0, 84.0);
    click(&mut app, 12.0, 15.0);
    assert!(!app.state.combo.is_open());
    assert_eq!(app.state.log.last().unwrap(), "first true");

    click(&mut app, 150.0, 84.0);
    app.key(Key::Down, Modifiers::default());
    app.key(Key::Enter, Modifiers::default());
    assert_eq!(app.state.combo.selected_text(), Some("z"));
}

#[test]
fn multiline_text_box_edits_lines() {
    let mut app = form();
    let none = Modifiers::default();
    click(&mut app, 20.0, 230.0);
    app.text_input("первая");
    app.key(Key::Enter, none);
    app.text_input("вторая");
    app.key(Key::Up, none);
    app.key(Key::End, none);
    app.text_input("!");
    app.key(Key::Down, none);
    app.key(Key::Home, none);
    app.text_input(">");
    assert_eq!(app.state.memo.text(), "первая!\n>вторая");
    // Щелчок по второй строке ставит курсор в неё.
    click(&mut app, 200.0, 250.0);
    app.text_input("?");
    assert_eq!(app.state.memo.text(), "первая!\n>вторая?");
}

#[test]
fn progress_value_stays_in_range() {
    let mut app = form();
    app.state.bar.set_value(500);
    assert_eq!(app.state.bar.value(), 100);
    app.state.bar.set_range(10, 20);
    assert_eq!(app.state.bar.value(), 20);
}
