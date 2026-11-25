use rustforms::{Key, Modifiers, Point, prelude::*};

struct Form {
    group: GroupBox,
    link: LinkLabel,
    number: NumericUpDown,
    track: TrackBar,
    grid: DataGridView,
    tree: TreeView,
    picture: PictureBox,
    log: Vec<String>,
}

fn form() -> Application<Form> {
    let mut state = Form {
        group: GroupBox::new("Группа").at(0.0, 0.0).size(200.0, 60.0),
        link: LinkLabel::new("ссылка").at(10.0, 20.0),
        number: NumericUpDown::new().at(0.0, 70.0),
        track: TrackBar::new().at(0.0, 110.0).size(114.0, 28.0),
        grid: DataGridView::new().at(0.0, 150.0).size(300.0, 90.0),
        tree: TreeView::new().at(310.0, 0.0).size(160.0, 200.0),
        picture: PictureBox::new().at(310.0, 210.0),
        log: Vec::new(),
    };
    state.number.set_range(0, 50);
    state.grid.set_columns(["Имя", "Город"]);
    for n in 0..6 {
        state.grid.add_row([format!("r{n}"), "Самара".to_owned()]);
    }
    state.tree.set_items(["A/B/C", "D", "A/E"]);
    let mut view = View::<Form>::new();
    let group = view.add(|f: &mut Form| &mut f.group);
    let link = view.add_to(group, |f: &mut Form| &mut f.link);
    let number = view.add(|f: &mut Form| &mut f.number);
    let track = view.add(|f: &mut Form| &mut f.track);
    let grid = view.add(|f: &mut Form| &mut f.grid);
    let tree = view.add(|f: &mut Form| &mut f.tree);
    view.add(|f: &mut Form| &mut f.picture);
    view.on_click(link, |f, _, _| f.log.push("link".into()));
    view.on_value_changed(number, |f, _, e| f.log.push(format!("number {}", e.value)));
    view.on_value_changed(track, |f, _, e| f.log.push(format!("track {}", e.value)));
    view.on_selected_changed(grid, |f, _, e| f.log.push(format!("grid {}", e.text)));
    view.on_selected_changed(tree, |f, _, e| f.log.push(format!("tree {}", e.text)));
    Application::new(Ui::new("t", 480.0, 320.0), state, view)
}

fn click(app: &mut Application<Form>, x: f32, y: f32) {
    app.mouse_move(Point::new(x, y));
    app.mouse_down(Modifiers::default());
    app.mouse_up();
}

#[test]
fn link_inside_group_clicks_and_numbers_change() {
    let none = Modifiers::default();
    let mut app = form();
    click(&mut app, 20.0, 30.0);
    // Верхняя и нижняя стрелки числового поля.
    click(&mut app, 90.0, 76.0);
    click(&mut app, 90.0, 92.0);
    click(&mut app, 30.0, 84.0);
    // Набор по цифрам: каждая подходящая по диапазону сразу применяется.
    app.text_input("4");
    app.text_input("2");
    app.key(Key::Up, none);
    app.text_input("9");
    app.text_input("9");
    assert_eq!(app.state.number.value(), 9, "99 вне диапазона и не применяется");
    assert_eq!(
        app.state.log,
        ["link", "number 1", "number 0", "number 4", "number 42", "number 43", "number 9"]
    );
}

#[test]
fn track_bar_follows_mouse_and_keys() {
    let mut app = form();
    app.mouse_move(Point::new(57.0, 124.0));
    app.mouse_down(Modifiers::default());
    app.mouse_move(Point::new(500.0, 124.0));
    app.mouse_up();
    app.key(Key::Left, Modifiers::default());
    assert_eq!(app.state.log, ["track 50", "track 100", "track 99"]);
}

#[test]
fn grid_selects_rows_and_scrolls_to_them() {
    let mut app = form();
    // Заголовок не выбирается.
    click(&mut app, 20.0, 155.0);
    click(&mut app, 20.0, 150.0 + 1.0 + 24.0 + 5.0);
    app.key(Key::End, Modifiers::default());
    assert_eq!(app.state.grid.selected_row().unwrap()[0], "r5");
    assert_eq!(app.state.log, ["grid r0", "grid r5"]);
}

#[test]
fn tree_expands_selects_and_keeps_branches_together() {
    let none = Modifiers::default();
    let mut app = form();
    // Свёрнуто: видны A и D. «+» у A.
    click(&mut app, 310.0 + 9.0, 11.0);
    click(&mut app, 370.0, 1.0 + 22.0 + 11.0);
    assert_eq!(app.state.tree.selected_path(), Some("A/B"));
    app.key(Key::Right, none);
    app.key(Key::Down, none);
    app.key(Key::Down, none);
    app.key(Key::Left, none);
    assert_eq!(app.state.log, ["tree A/B", "tree A/B/C", "tree A/E", "tree A"]);
}

#[test]
fn picture_accepts_only_png() {
    let mut app = form();
    assert!(!app.state.picture.set_image_png(b"not a png"));
    let png = tiny_skia::Pixmap::new(4, 2).unwrap().encode_png().unwrap();
    assert!(app.state.picture.set_image_png(&png));
    assert_eq!(app.state.picture.image_size(), Some((4, 2)));
}
