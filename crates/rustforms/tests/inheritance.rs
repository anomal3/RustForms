use rustforms::{Key, Modifiers, Point, prelude::*};

/// Кнопка, считающая свои нажатия: мышь переопределена, клавиатура — родительская.
#[derive(Control)]
struct CountingButton {
    #[base]
    base: Button,
    presses: u32,
}

impl ControlImpl for CountingButton {
    fn on_mouse_down(&mut self, at: Point, mods: Modifiers, cx: &mut EventCx) {
        self.presses += 1;
        self.base.on_mouse_down(at, mods, cx);
    }
}

/// Два уровня «наследования».
#[derive(Control)]
struct Deeper(#[base] CountingButton);
impl ControlImpl for Deeper {}

#[derive(Control)]
struct DigitsBox {
    #[base]
    base: TextBox,
}

impl ControlImpl for DigitsBox {
    fn on_text_input(&mut self, text: &str, cx: &mut EventCx) {
        let digits: String = text.chars().filter(char::is_ascii_digit).collect();
        self.base.on_text_input(&digits, cx);
    }
}

#[derive(Control)]
struct Group(#[base] Panel);
impl ControlImpl for Group {}

#[derive(Default)]
struct BaseForm {
    log: Vec<&'static str>,
}

struct Derived {
    base: BaseForm,
}

#[test]
fn derived_controls_keep_parent_behaviour() {
    let mut ui = Ui::new("t", 300.0, 200.0);
    let group = ui.add(Group(Panel::new()).at(10.0, 10.0));
    let button = ui.add_to(
        group,
        Deeper(CountingButton { base: Button::new("a"), presses: 0 }).at(5.0, 5.0),
    );
    let digits = ui.add(DigitsBox { base: TextBox::new() }.at(10.0, 150.0));

    #[derive(Default)]
    struct S {
        clicks: u32,
        text: String,
    }
    let mut events = Events::<S>::new();
    events.on_click(button, |s, _, _| s.clicks += 1);
    events.on_text_changed(digits, |s, _, e| s.text = e.text.clone());
    let mut app = Application::new(ui, S::default(), events);

    app.mouse_move(Point::new(30.0, 25.0));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
    app.key(Key::Space, Modifiers::default());
    assert_eq!(app.state.clicks, 2);
    assert_eq!(app.ui[button].presses, 1);
    // свойства родителя доступны напрямую
    assert_eq!(app.ui[button].text(), "a");

    app.mouse_move(Point::new(20.0, 160.0));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
    app.text_input("a1б2");
    assert_eq!(app.state.text, "12");
    assert_eq!(app.ui[digits].text(), "12");
}

#[test]
fn base_form_handlers_run_before_derived() {
    let mut ui = Ui::new("t", 200.0, 100.0);
    let mut events = Events::<Derived>::new();
    let ok = events.include(
        |form| &mut form.base,
        |events| {
            let ok = ui.add(Button::new("OK").at(0.0, 0.0));
            events.on_load(|f: &mut BaseForm, _, _| f.log.push("base load"));
            events.on_click(ok, |f: &mut BaseForm, _, _| f.log.push("base click"));
            ok
        },
    );
    events.on_load(|f, _, _| f.base.log.push("derived load"));
    events.on_click(ok, |f, _, _| f.base.log.push("derived click"));
    let mut app = Application::new(ui, Derived { base: BaseForm::default() }, events);

    app.load();
    app.mouse_move(Point::new(5.0, 5.0));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
    assert_eq!(app.state.base.log, ["base load", "derived load", "base click", "derived click"]);
}
