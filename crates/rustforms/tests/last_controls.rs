use rustforms::{Key, Modifiers, Point, prelude::*};

struct Form {
    date: DateTimePicker,
    split: SplitContainer,
    left: SplitPanel,
    right: SplitPanel,
    wide: Button,
    tools: ToolStrip,
    grid: DataGridView,
    input: TextBox,
    log: Vec<String>,
}

fn form() -> Application<Form> {
    let mut state = Form {
        date: DateTimePicker::new().at(0.0, 40.0),
        split: SplitContainer::new().at(0.0, 80.0).size(300.0, 100.0),
        left: SplitPanel::new(),
        right: SplitPanel::new().size(195.0, 100.0),
        wide: Button::new("w").at(5.0, 5.0).size(180.0, 28.0),
        tools: ToolStrip::new().size(400.0, 30.0),
        grid: DataGridView::new().at(0.0, 200.0).size(302.0, 120.0),
        input: TextBox::new().at(200.0, 40.0),
        log: Vec::new(),
    };
    state.date.set_value(Date::new(2026, 11, 25));
    state.wide.set_anchor(Anchor::LEFT | Anchor::TOP | Anchor::RIGHT);
    state.tools.set_items(["New", "-", "Save"]);
    state.grid.set_columns(["name", "score"]);
    state.grid.add_row(["b", "10"]);
    state.grid.add_row(["a", "9"]);
    state.grid.add_row(["c", "100"]);
    state.grid.set_editable(true);
    state.left.set_context_menu(["Копировать", "-", "Вид/Крупно"]);
    let mut view = View::<Form>::new();
    let date = view.add(|f: &mut Form| &mut f.date);
    let split = view.add(|f: &mut Form| &mut f.split);
    let left = view.add_to(split, |f: &mut Form| &mut f.left);
    let right = view.add_to(split, |f: &mut Form| &mut f.right);
    view.add_to(right, |f: &mut Form| &mut f.wide);
    let tools = view.add(|f: &mut Form| &mut f.tools);
    let grid = view.add(|f: &mut Form| &mut f.grid);
    view.add(|f: &mut Form| &mut f.input);
    view.on_selected_changed(date, |f, _, e| f.log.push(format!("date {}", e.text)));
    view.on_item_clicked(tools, |f, _, e| f.log.push(format!("tool {}", e.item)));
    view.on_context_item_clicked(left, |f, _, e| f.log.push(format!("context {}", e.item)));
    view.on_cell_changed(grid, |f, _, e| f.log.push(format!("cell {} {} {}", e.row, e.column, e.text)));
    Application::new(Ui::new("t", 420.0, 340.0), state, view)
}

fn click(app: &mut Application<Form>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn dates_are_correct_and_calendar_selects() {
    let d = Date::new(2024, 2, 28);
    assert_eq!(d.add_days(1), Date::new(2024, 2, 29));
    assert_eq!(d.add_months(12), Date::new(2025, 2, 28));
    assert_eq!(Date::new(2026, 11, 25).weekday(), 2, "среда");
    assert_eq!("2026-11-25".parse::<Date>().unwrap().to_string(), "2026-11-25");
    assert!("2026-02-30".parse::<Date>().is_err());
    assert_eq!(Date::from_days(0), Date::new(1970, 1, 1));

    let mut app = form();
    click(&mut app, 20.0, 54.0);
    assert!(app.state.date.is_open());
    // Ноябрь 2026 начинается в воскресенье: первая клетка — 26 октября.
    // Клетка в четвёртой строке, первом столбце — 16 ноября.
    click(&mut app, 6.0 + 15.0, 68.0 + 30.0 + 24.0 + 3.0 * 24.0 + 12.0);
    assert_eq!(app.state.date.value(), Date::new(2026, 11, 16));
    app.key(Key::Right, Modifiers::default());
    assert_eq!(app.state.log, ["date 2026-11-16", "date 2026-11-17"]);
}

#[test]
fn splitter_moves_and_children_follow_anchors() {
    let mut app = form();
    assert_eq!(app.state.right.bounds(), Rect::new(105.0, 0.0, 195.0, 100.0));
    app.mouse_move(Point::new(102.0, 130.0));
    app.mouse_down(Modifiers::default());
    app.mouse_move(Point::new(152.0, 130.0));
    app.mouse_up();
    app.layout();
    assert_eq!(app.state.left.bounds().width, 150.0 - 0.5);
    assert_eq!(app.state.wide.bounds().width, 180.0 - 49.5);
}

#[test]
fn tool_strip_and_context_menus() {
    let mut app = form();
    click(&mut app, 20.0, 15.0);
    app.mouse_move(Point::new(30.0, 100.0));
    app.context_click();
    assert!(app.context_open());
    // Первый пункт меню.
    click(&mut app, 50.0, 100.0 + 3.0 + 12.0);
    assert!(!app.context_open());
    app.mouse_move(Point::new(30.0, 100.0));
    app.context_click();
    app.key(Key::Escape, Modifiers::default());
    assert_eq!(app.state.log, ["tool New", "context Копировать"]);

    // У поля ввода своё меню: «Выделить всё», затем «Вырезать».
    app.state.input.set_text("abc");
    app.mouse_move(Point::new(210.0, 54.0));
    app.context_click();
    click(&mut app, 230.0, 54.0 + 3.0 + 24.0 * 3.0 + 12.0);
    assert_eq!(app.state.input.selected_text(), "abc");
    app.mouse_move(Point::new(210.0, 54.0));
    app.context_click();
    click(&mut app, 230.0, 54.0 + 3.0 + 12.0);
    assert_eq!(app.state.input.text(), "");
}

#[test]
fn grid_sorts_resizes_and_edits() {
    let none = Modifiers::default();
    let mut app = form();
    // Щелчок по заголовку «score»: числа сортируются как числа.
    click(&mut app, 200.0, 212.0);
    let names: Vec<&str> = app.state.grid.rows().iter().map(|r| r[0].as_str()).collect();
    assert_eq!(names, ["a", "b", "c"]);
    click(&mut app, 200.0, 212.0);
    assert_eq!(app.state.grid.rows()[0][1], "100");

    // Повторный щелчок по выбранной ячейке начинает правку.
    let row = 200.0 + 1.0 + 24.0 + 12.0;
    click(&mut app, 40.0, row);
    assert!(!app.state.grid.is_editing());
    click(&mut app, 40.0, row);
    assert!(app.state.grid.is_editing());
    app.text_input("!");
    app.key(Key::Enter, none);
    // Набор начинает правку заново, Escape отменяет.
    app.text_input("zzz");
    app.key(Key::Escape, none);
    // Уход фокуса сохраняет.
    app.text_input("q");
    app.key(Key::Tab, none);
    assert_eq!(app.state.log, ["cell 0 0 c!", "cell 0 0 q"]);

    // Граница столбцов тянется мышью.
    app.mouse_move(Point::new(151.0, 212.0));
    app.mouse_down(none);
    app.mouse_move(Point::new(61.0, 212.0));
    app.mouse_up();
    click(&mut app, 100.0, row);
    click(&mut app, 100.0, row);
    app.text_input("5");
    app.key(Key::Enter, none);
    // Правка открылась с прежним «100», набранное дописано в конец.
    assert_eq!(app.state.grid.cell(0, 1), Some("1005"), "щелчок попал уже во второй столбец");
}
