use rustforms::{Key, Modifiers, Point, prelude::*};

struct Form {
    panel: Panel,
    button: Button,
    input: TextBox,
    check: CheckBox,
    clicks: u32,
    texts: Vec<String>,
    checks: Vec<bool>,
}

struct Handles {
    button: Handle<Button>,
    input: Handle<TextBox>,
    check: Handle<CheckBox>,
}

fn form() -> (Application<Form>, Handles) {
    let state = Form {
        panel: Panel::new().at(10.0, 10.0).size(280.0, 180.0),
        button: Button::new("OK").at(10.0, 10.0),
        input: TextBox::new().at(10.0, 50.0),
        check: CheckBox::new("x").at(10.0, 90.0),
        clicks: 0,
        texts: Vec::new(),
        checks: Vec::new(),
    };
    let mut view = View::<Form>::new();
    let panel = view.add(|f: &mut Form| &mut f.panel);
    let button = view.add_to(panel, |f: &mut Form| &mut f.button);
    let input = view.add_to(panel, |f: &mut Form| &mut f.input);
    let check = view.add_to(panel, |f: &mut Form| &mut f.check);
    view.on_click(button, |f, _, _| f.clicks += 1);
    view.on_text_changed(input, |f, _, e| f.texts.push(e.text.clone()));
    view.on_checked_changed(check, |f, _, e| f.checks.push(e.checked));
    let app = Application::new(Ui::new("test", 300.0, 200.0), state, view);
    (app, Handles { button, input, check })
}

fn click(app: &mut Application<Form>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn click_inside_panel_reaches_handler() {
    let (mut app, _) = form();
    click(&mut app, 30.0, 30.0);
    assert_eq!(app.state.clicks, 1);
}

#[test]
fn release_outside_cancels_click() {
    let (mut app, _) = form();
    app.mouse_move(Point::new(30.0, 30.0));
    app.mouse_down(Modifiers::default());
    app.mouse_move(Point::new(280.0, 180.0));
    app.mouse_up();
    assert_eq!(app.state.clicks, 0);
}

#[test]
fn disabled_and_hidden_parent_block_input() {
    let (mut app, _) = form();
    app.state.panel.set_enabled(false);
    click(&mut app, 30.0, 30.0);
    app.state.panel.set_enabled(true);
    app.state.panel.set_visible(false);
    click(&mut app, 30.0, 30.0);
    assert_eq!(app.state.clicks, 0);
}

#[test]
fn keyboard_tab_and_space() {
    let (mut app, handles) = form();
    let none = Modifiers::default();
    app.key(Key::Tab, none);
    assert_eq!(app.focused(), Some(handles.button.id()));
    app.key(Key::Space, none);
    app.key(Key::Tab, none);
    assert_eq!(app.focused(), Some(handles.input.id()));
    app.key(Key::Tab, none);
    app.key(Key::Space, none);
    assert_eq!(app.focused(), Some(handles.check.id()));
    assert_eq!((app.state.clicks, app.state.checks.clone()), (1, vec![true]));
    assert!(app.state.check.checked());
}

#[test]
fn typing_russian_editing_and_clipboard() {
    let (mut app, _) = form();
    let none = Modifiers::default();
    let shift = Modifiers { shift: true, command: false };
    click(&mut app, 100.0, 70.0);
    app.text_input("Привет");
    app.key(Key::Backspace, none);
    assert_eq!(app.state.input.text(), "Приве");
    app.key(Key::Home, none);
    app.key(Key::Right, shift);
    app.key(Key::Right, shift);
    assert_eq!(app.state.input.selected_text(), "Пр");
    app.key(Key::Cut, none);
    app.key(Key::End, none);
    app.key(Key::Paste, none);
    assert_eq!(app.state.input.text(), "ивеПр");
    assert_eq!(app.state.texts.last().unwrap(), "ивеПр");
}

#[test]
fn programmatic_change_raises_no_event() {
    let (mut app, _) = form();
    app.state.input.set_text("a\nb");
    assert_eq!(app.state.input.text(), "ab");
    assert!(app.state.texts.is_empty());
}

#[test]
fn anchors_and_dock_follow_window_size() {
    let (mut app, _) = form();
    app.state.button.set_anchor(Anchor::RIGHT | Anchor::BOTTOM);
    app.state.input.set_anchor(Anchor::LEFT | Anchor::RIGHT);
    app.state.panel.set_anchor(Anchor::LEFT | Anchor::TOP | Anchor::RIGHT | Anchor::BOTTOM);
    app.ui.width += 100.0;
    app.ui.height += 50.0;
    app.layout();
    app.layout();
    // Панель выросла вместе с окном, а дети — вместе с панелью.
    assert_eq!(app.state.panel.bounds(), Rect::new(10.0, 10.0, 380.0, 230.0));
    assert_eq!(app.state.button.bounds().x, 110.0);
    assert_eq!(app.state.button.bounds().y, 60.0);
    assert_eq!(app.state.input.bounds().width, 260.0);
    // Без якорей по вертикали — смещение на половину.
    assert_eq!(app.state.input.bounds().y, 75.0);

    app.state.panel.set_dock(Dock::Fill);
    app.state.button.set_dock(Dock::Top);
    app.state.check.set_dock(Dock::Left);
    app.layout();
    assert_eq!(app.state.panel.bounds(), Rect::new(0.0, 0.0, 400.0, 250.0));
    assert_eq!(app.state.button.bounds(), Rect::new(0.0, 0.0, 400.0, 28.0));
    assert_eq!(app.state.check.bounds(), Rect::new(0.0, 28.0, 140.0, 222.0));
}
