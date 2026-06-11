use fltk::{
    app,
    enums::{Align, Event},
    frame::Frame,
    menu::MenuItem,
    prelude::*,
    text::TextEditor,
};
use std::{
    cell::RefCell,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
};

use super::SearchState;

pub type StatusBar = Rc<RefCell<Frame>>;
pub type UpdateStatus = Rc<dyn Fn()>;

pub fn create_status_bar(x: i32, y: i32, w: i32, h: i32) -> StatusBar {
    let mut f = Frame::new(x, y, w, h, "");
    f.set_align(Align::Left | Align::Inside);
    f.set_color(fltk::enums::Color::from_rgb(240, 240, 240));
    Rc::new(RefCell::new(f))
}

fn file_display_name(path: &str) -> String {
    if path.is_empty() {
        return "(untitled)".into();
    }

    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.to_string())
}

fn open_file_location(path: &str) {
    if path.is_empty() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let folder = Path::new(path).parent().unwrap_or_else(|| Path::new(path));
        let _ = Command::new("explorer.exe").arg(folder).spawn();
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = Path::new(path).parent() {
            let _ = Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

fn copy_full_path(path: &str) {
    if path.is_empty() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let copied = Command::new("clip.exe")
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(path.as_bytes())?;
                }
                child.wait()
            })
            .map(|status| status.success())
            .unwrap_or(false);

        if copied {
            return;
        }
    }

    app::copy(path);
}

pub fn attach_status_path_actions(status_bar: &StatusBar, search_state: &Rc<RefCell<SearchState>>) {
    let search_state = Rc::clone(search_state);
    let menu = MenuItem::new(&["Copy Full Path", "Open File Location"]);

    status_bar.borrow_mut().handle(move |_, ev| {
        if ev != Event::Push {
            return false;
        }

        let path = search_state.borrow().filepath.clone();
        if path.is_empty() {
            return false;
        }

        if app::event_mouse_button() != app::MouseButton::Right {
            return false;
        }

        let (x, y) = app::event_coords();
        if let Some(choice) = menu.popup(x, y) {
            if choice.label().as_deref() == Some("Copy Full Path") {
                copy_full_path(&path);
            } else if choice.label().as_deref() == Some("Open File Location") {
                open_file_location(&path);
            }
        }
        true
    });
}

pub fn make_update_status(
    status_bar: &StatusBar,
    editor: &TextEditor,
    search_state: &Rc<RefCell<SearchState>>,
) -> UpdateStatus {
    let status_bar = Rc::clone(status_bar);
    let editor = editor.clone();
    let search_state = Rc::clone(search_state);

    Rc::new(move || {
        let pos = editor.insert_position();
        let line = editor.count_lines(0, pos, false);
        let col = pos - editor.line_start(pos);

        let fp = search_state.borrow().filepath.clone();
        let display = file_display_name(&fp);

        status_bar.borrow_mut().set_label(&format!(
            "Ln {}, Col {}  |  {}",
            line + 1,
            col + 1,
            display
        ));
    })
}

pub fn show_search_controls(search: &mut super::search::SearchControls) {
    search.input.show();
    search.results.borrow_mut().show();
}

pub fn hide_search_controls(search: &mut super::search::SearchControls) {
    search.input.hide();
    search.results.borrow_mut().hide();
}
