use std::{fs, path::PathBuf};

use rustforms_model::*;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rustforms-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn double_click_adds_handler_once_and_keeps_user_code() {
    let dir = temp("handlers");
    let rfrm = new_form(&dir, "MainForm").unwrap();
    assert!(rfrm.ends_with("main_form.rfrm"));
    assert!(new_form(&dir, "MainForm").is_err());

    let mut model = load(&rfrm).unwrap();
    let name = model.free_name(Kind::Button);
    model.controls.push(Control::new(Kind::Button, name, 10.0, 10.0));
    save(&rfrm, &model).unwrap();

    // Пользователь дописал своё, в том числе с кириллицей и CRLF.
    let code = dir.join("main_form.rs");
    let mine = fs::read_to_string(&code).unwrap() + "\n// мой комментарий\nfn helper() {}\n";
    let mine = format!("\u{feff}{}", mine.replace('\n', "\r\n"));
    fs::write(&code, &mine).unwrap();

    let at = attach_handler(&rfrm, Some("button1"), None).unwrap();
    let after = fs::read_to_string(&code).unwrap();
    let lines: Vec<&str> = after.lines().collect();
    assert_eq!(lines[at.line - 1].trim(), "let _ = (ui, e);");
    assert!(lines[at.line - 2].contains("fn button1_click(&mut self, ui: &mut Ui, e: &ClickArgs)"));
    assert!(after.starts_with('\u{feff}') && after.contains("// мой комментарий\r\nfn helper() {}"));
    assert!(!after.replace("\r\n", "").contains('\n'));

    assert_eq!(attach_handler(&rfrm, Some("button1"), None).unwrap(), at);
    assert_eq!(fs::read_to_string(&code).unwrap(), after);

    attach_handler(&rfrm, None, None).unwrap();
    let designer = fs::read_to_string(dir.join("main_form.designer.rs")).unwrap();
    assert!(designer.contains("view.on_load(Self::main_form_load);"));
    assert!(designer.contains("view.on_click(button1_id, Self::button1_click);"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn broken_user_code_is_not_touched() {
    let dir = temp("broken");
    let rfrm = new_form(&dir, "MainForm").unwrap();
    let before = fs::read_to_string(&rfrm).unwrap();
    fs::write(dir.join("main_form.rs"), "fn {").unwrap();
    assert!(attach_handler(&rfrm, None, None).is_err());
    assert_eq!(fs::read_to_string(dir.join("main_form.rs")).unwrap(), "fn {");
    assert_eq!(fs::read_to_string(&rfrm).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn invalid_models_are_rejected() {
    let base = "[form]\nname='F'\ntitle='t'\nwidth=10.0\nheight=10.0\n";
    let control = |extra: &str| {
        format!("{base}[[control]]\nname='a'\nx=0.0\ny=0.0\nwidth=1.0\nheight=1.0\n{extra}")
    };
    assert!(FormModel::from_toml(&control("type='Button'")).is_ok());
    for bad in [
        "type='Button'\nparent='nope'",
        "type='Panel'\n[control.events]\nclick='x'",
        "type='Button'\n[control.events]\nclick='fn'",
        "type='Label'\nfore_color='red'",
        "type='Button'\ntypo=1",
    ] {
        assert!(FormModel::from_toml(&control(bad)).is_err(), "{bad}");
    }
}

/// Пример `hello` лежит в репозитории уже сгенерированным и не должен расходиться с генератором.
#[test]
fn hello_example_is_up_to_date() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/hello/src");
    let model = load(&dir.join("main_form.rfrm")).unwrap();
    let on_disk = fs::read_to_string(dir.join("main_form.designer.rs")).unwrap();
    assert_eq!(on_disk.replace("\r\n", "\n"), designer_code(&model).unwrap());
}

#[test]
fn every_kind_survives_save_and_generates_setup() {
    let mut model = FormModel::new("F");
    for kind in Kind::ALL {
        let name = model.free_name(kind);
        let mut c = Control::new(kind, name, 1.0, 2.0);
        if kind == Kind::TabPage {
            c.parent = Some("tab_control1".into());
        }
        if kind == Kind::SplitPanel {
            c.parent = Some("split_container1".into());
        }
        if kind == Kind::DateTimePicker {
            c.date = Some("2026-02-28".into());
        }
        if kind == Kind::SplitContainer {
            (c.distance, c.horizontal) = (80, true);
        }
        if kind == Kind::DataGridView {
            c.editable = true;
            c.context = vec!["Удалить".into()];
            c.events.insert(Event::CellChanged, "cell".into());
            c.events.insert(Event::ContextItemClicked, "menu".into());
        }
        if kind == Kind::Timer {
            (c.interval, c.enabled) = (250, true);
            c.events.insert(Event::Tick, "tick".into());
        }
        if kind.has_items() {
            c.items = vec!["a \"b\"".into(), "в".into()];
            c.selected = kind.selectable().then_some(1);
        }
        if kind == Kind::PictureBox {
            c.image = Some("pic\\logo.png".into());
        }
        if matches!(kind, Kind::NumericUpDown | Kind::TrackBar) {
            c.value = 3;
        }
        if kind.checkable() {
            c.checked = true;
        }
        if kind == Kind::TextBox {
            c.multiline = true;
            c.text = Some("1\n2".into());
        }
        if kind == Kind::ProgressBar {
            (c.minimum, c.maximum, c.value) = (5, 50, 7);
        }
        model.controls.push(c);
    }
    assert_eq!(FormModel::from_toml(&model.to_toml().unwrap()).unwrap(), model);
    let code = designer_code(&model).unwrap();
    for part in [
        "set_multiline(true);\n        text_box1.set_text(\"1\\n2\")",
        "combo_box1.set_items([\"a \\\"b\\\"\", \"в\"]);",
        "list_box1.set_selected_index(Some(1));",
        "radio_button1.set_checked(true);",
        "progress_bar1.set_range(5, 50);",
        "menu_strip1.set_dock(Dock::Top);",
        "status_strip1.set_dock(Dock::Bottom);",
        "data_grid_view1.set_columns([",
        "picture_box1.set_image_png(include_bytes!(\"pic/logo.png\"));",
        "track_bar1.set_value(3);",
        "view.add_to(tab_control1_id, |f: &mut Self| &mut f.tab_page1);",
        "let mut timer1 = Timer::new(250);\n        timer1.start();",
        "let timer1_id = view.add_timer(|f: &mut Self| &mut f.timer1);",
        "view.on_tick(timer1_id, Self::tick);",
        "date_time_picker1.set_value(Date::new(2026, 2, 28));",
        "split_container1.set_distance(80.0);",
        "data_grid_view1.set_editable(true);",
        "data_grid_view1.set_context_menu([\"Удалить\"]);",
        "view.on_cell_changed(data_grid_view1_id, Self::cell);",
        "view.on_context_item_clicked(data_grid_view1_id, Self::menu);",
    ] {
        assert!(code.contains(part), "{part}\n{code}");
    }
    let date = model.controls.iter_mut().find(|c| c.name == "date_time_picker1").unwrap();
    date.date = Some("2026-02-30".into());
    assert!(model.validate().is_err());
    model.controls.iter_mut().find(|c| c.name == "date_time_picker1").unwrap().date = None;
    model.controls.iter_mut().find(|c| c.name == "list_box1").unwrap().selected = Some(9);
    assert!(model.validate().is_err());
}

#[test]
fn common_properties_form_events_and_git_projects() {
    let mut model = FormModel::new("F");
    let mut c = Control::new(Kind::Label, "title", 0.0, 0.0);
    c.fore_color = Some("#102030".into());
    c.back_color = Some("#ffffff".into());
    c.font_size = Some(20.0);
    c.bold = true;
    c.tooltip = Some("\"hint\"".into());
    c.tab_index = 3;
    c.align = Some(Align::Right);
    c.events.insert(Event::MouseEnter, "title_enter".into());
    model.controls.push(c);
    model.form.closing = Some("f_closing".into());
    assert_eq!(FormModel::from_toml(&model.to_toml().unwrap()).unwrap(), model);
    let code = designer_code(&model).unwrap();
    for part in [
        "title.set_fore_color(Color::rgb(16, 32, 48));",
        "title.set_font_size(20.0);",
        "title.set_tooltip(\"\\\"hint\\\"\");",
        "title.set_text_align(TextAlign::Right);",
        "view.on_closing(Self::f_closing);",
        "view.on_mouse_enter(title_id, Self::title_enter);",
    ] {
        assert!(code.contains(part), "{part}\n{code}");
    }

    let dir = temp("git").join("app");
    fs::create_dir_all(&dir).unwrap();
    new_project(&dir, Library::Git { url: "https://example.invalid/r.git", tag: "v1" }).unwrap();
    let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(manifest.contains("rustforms = { git = \"https://example.invalid/r.git\", tag = \"v1\" }"));
    assert!(dir.join(".cargo/config.toml").is_file());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap();
}
