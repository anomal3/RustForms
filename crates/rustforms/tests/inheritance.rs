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

struct Controls {
    group: Group,
    button: Deeper,
    digits: DigitsBox,
    clicks: u32,
    text: String,
}

#[test]
fn derived_controls_keep_parent_behaviour() {
    let state = Controls {
        group: Group(Panel::new()).at(10.0, 10.0),
        button: Deeper(CountingButton { base: Button::new("a"), presses: 0 }).at(5.0, 5.0),
        digits: DigitsBox { base: TextBox::new() }.at(10.0, 150.0),
        clicks: 0,
        text: String::new(),
    };
    let mut view = View::<Controls>::new();
    let group = view.add(|f: &mut Controls| &mut f.group);
    let button = view.add_to(group, |f: &mut Controls| &mut f.button);
    let digits = view.add(|f: &mut Controls| &mut f.digits);
    view.on_click(button, |s, _, _| s.clicks += 1);
    view.on_text_changed(digits, |s, _, e| s.text = e.text.clone());
    let mut app = Application::new(Ui::new("t", 300.0, 200.0), state, view);

    app.mouse_move(Point::new(30.0, 25.0));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
    app.key(Key::Space, Modifiers::default());
    assert_eq!(app.state.clicks, 2);
    assert_eq!(app.state.button.presses, 1);
    // свойства родителя доступны напрямую
    assert_eq!(app.state.button.text(), "a");

    app.mouse_move(Point::new(20.0, 160.0));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
    app.text_input("a1б2");
    assert_eq!(app.state.text, "12");
    assert_eq!(app.state.digits.text(), "12");
}

struct BaseForm {
    ok: Button,
    log: Vec<&'static str>,
}

struct Derived {
    base: BaseForm,
}

#[test]
fn base_form_controls_and_handlers_are_included() {
    let state = Derived {
        base: BaseForm { ok: Button::new("OK").at(0.0, 0.0), log: Vec::new() },
    };
    let mut view = View::<Derived>::new();
    let ok = view.include(
        |form| &mut form.base,
        |view| {
            let ok = view.add(|f: &mut BaseForm| &mut f.ok);
            view.on_load(|f: &mut BaseForm, _, _| f.log.push("base load"));
            view.on_click(ok, |f: &mut BaseForm, _, _| f.log.push("base click"));
            ok
        },
    );
    view.on_load(|f, _, _| f.base.log.push("derived load"));
    view.on_click(ok, |f, _, _| f.base.log.push("derived click"));
    let mut app = Application::new(Ui::new("t", 200.0, 100.0), state, view);

    app.load();
    app.mouse_move(Point::new(5.0, 5.0));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
    assert_eq!(app.state.base.log, ["base load", "derived load", "base click", "derived click"]);
}
