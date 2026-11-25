use rustforms::{Key, Modifiers, Point, prelude::*};

#[derive(Default)]
struct State {
    clicks: u32,
    texts: Vec<String>,
    checks: Vec<bool>,
}

type Form = (Application<State>, Handle<Button>, Handle<TextBox>, Handle<CheckBox>, Handle<Panel>);

fn form() -> Form {
    let mut ui = Ui::new("test", 300.0, 200.0);
    let panel = ui.add(Panel::new().at(10.0, 10.0).size(280.0, 180.0));
    let button = ui.add_to(panel, Button::new("OK").at(10.0, 10.0));
    let input = ui.add_to(panel, TextBox::new().at(10.0, 50.0));
    let check = ui.add_to(panel, CheckBox::new("x").at(10.0, 90.0));
    let mut events = Events::<State>::new();
    events.on_click(button, |s, _, _| s.clicks += 1);
    events.on_text_changed(input, |s, _, e| s.texts.push(e.text.clone()));
    events.on_checked_changed(check, |s, _, e| s.checks.push(e.checked));
    (Application::new(ui, State::default(), events), button, input, check, panel)
}

fn click(app: &mut Application<State>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn click_inside_panel_reaches_handler() {
    let (mut app, ..) = form();
    click(&mut app, 30.0, 30.0);
    assert_eq!(app.state.clicks, 1);
}

#[test]
fn release_outside_cancels_click() {
    let (mut app, ..) = form();
    app.mouse_move(Point::new(30.0, 30.0));
    app.mouse_down(Modifiers::default());
    app.mouse_move(Point::new(280.0, 180.0));
    app.mouse_up();
    assert_eq!(app.state.clicks, 0);
}

#[test]
fn disabled_and_hidden_parent_block_input() {
    let (mut app, _, _, _, panel) = form();
    app.ui[panel].set_enabled(false);
    click(&mut app, 30.0, 30.0);
    app.ui[panel].set_enabled(true);
    app.ui[panel].set_visible(false);
    click(&mut app, 30.0, 30.0);
    assert_eq!(app.state.clicks, 0);
}

#[test]
fn keyboard_tab_and_space() {
    let (mut app, button, input, check, _) = form();
    let none = Modifiers::default();
    app.key(Key::Tab, none);
    assert_eq!(app.focused(), Some(button.id()));
    app.key(Key::Space, none);
    app.key(Key::Tab, none);
    assert_eq!(app.focused(), Some(input.id()));
    app.key(Key::Tab, none);
    app.key(Key::Space, none);
    assert_eq!(app.focused(), Some(check.id()));
    assert_eq!((app.state.clicks, app.state.checks.clone()), (1, vec![true]));
    assert!(app.ui[check].checked());
}

#[test]
fn typing_russian_editing_and_clipboard() {
    let (mut app, _, input, ..) = form();
    let none = Modifiers::default();
    let shift = Modifiers { shift: true, command: false };
    click(&mut app, 100.0, 70.0);
    app.text_input("Привет");
    app.key(Key::Backspace, none);
    assert_eq!(app.ui[input].text(), "Приве");
    app.key(Key::Home, none);
    app.key(Key::Right, shift);
    app.key(Key::Right, shift);
    assert_eq!(app.ui[input].selected_text(), "Пр");
    app.key(Key::Cut, none);
    app.key(Key::End, none);
    app.key(Key::Paste, none);
    assert_eq!(app.ui[input].text(), "ивеПр");
    assert_eq!(app.state.texts.last().unwrap(), "ивеПр");
}

#[test]
fn programmatic_change_raises_no_event() {
    let (mut app, _, input, ..) = form();
    app.ui[input].set_text("a\nb");
    assert_eq!(app.ui[input].text(), "ab");
    assert!(app.state.texts.is_empty());
}
