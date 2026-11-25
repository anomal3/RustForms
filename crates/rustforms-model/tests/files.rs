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
