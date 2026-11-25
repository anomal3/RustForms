use std::time::{Duration, Instant};

use rustforms::{Key, Modifiers, Point, prelude::*};

struct Form {
    tabs: TabControl,
    first: TabPage,
    second: TabPage,
    one: Button,
    two: Button,
    timer: Timer,
    log: Vec<String>,
}

fn form() -> Application<Form> {
    let state = Form {
        tabs: TabControl::new().at(0.0, 0.0).size(300.0, 180.0),
        first: TabPage::new("Первая"),
        second: TabPage::new("Вторая"),
        one: Button::new("1").at(10.0, 10.0),
        two: Button::new("2").at(10.0, 10.0),
        timer: Timer::new(100),
        log: Vec::new(),
    };
    let mut view = View::<Form>::new();
    let tabs = view.add(|f: &mut Form| &mut f.tabs);
    let first = view.add_to(tabs, |f: &mut Form| &mut f.first);
    let one = view.add_to(first, |f: &mut Form| &mut f.one);
    let second = view.add_to(tabs, |f: &mut Form| &mut f.second);
    let two = view.add_to(second, |f: &mut Form| &mut f.two);
    let timer = view.add_timer(|f: &mut Form| &mut f.timer);
    view.on_click(one, |f, _, _| f.log.push("one".into()));
    view.on_click(two, |f, _, _| f.log.push("two".into()));
    view.on_selected_changed(tabs, |f, _, e| f.log.push(format!("tab {}", e.text)));
    view.on_tick(timer, |f, _, _| {
        f.log.push("tick".into());
        if f.log.len() == 2 {
            f.timer.stop();
        }
    });
    Application::new(Ui::new("t", 320.0, 200.0), state, view)
}

fn click(app: &mut Application<Form>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn only_the_selected_page_is_visible_and_receives_input() {
    let mut app = form();
    assert_eq!(app.state.first.bounds(), Rect::new(0.0, 28.0, 300.0, 152.0));
    click(&mut app, 30.0, 50.0);
    // Ярлычок второй вкладки.
    let mut pixmap = tiny_skia::Pixmap::new(320, 200).unwrap();
    app.render(&mut pixmap, 1.0);
    let edge = app.state.tabs.tab_edges()[1];
    click(&mut app, edge + 5.0, 14.0);
    click(&mut app, 30.0, 50.0);
    assert_eq!(app.state.log, ["one", "tab Вторая", "two"]);

    // Выбор из кода и стрелками.
    app.state.tabs.set_selected_index(0);
    click(&mut app, 30.0, 50.0);
    assert_eq!(app.state.log.last().unwrap(), "one");
    click(&mut app, 5.0, 14.0);
    app.key(Key::Right, Modifiers::default());
    assert_eq!(app.state.tabs.selected_index(), 1);
}

#[test]
fn timer_fires_on_interval_until_stopped() {
    let mut app = form();
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);
    assert!(!app.tick(at(0)) && app.next_wakeup().is_none(), "остановлен");
    app.state.timer.start();
    assert!(!app.tick(at(0)));
    assert_eq!(app.next_wakeup(), Some(at(100)));
    assert!(!app.tick(at(50)));
    assert!(app.tick(at(120)));
    // Долгий простой даёт одно срабатывание, а не десять.
    assert!(app.tick(at(1000)));
    assert!(!app.state.timer.enabled() && app.next_wakeup().is_none());
    assert!(!app.tick(at(5000)));
    assert_eq!(app.state.log, ["tick", "tick"]);
}
