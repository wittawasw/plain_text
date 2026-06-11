use fltk::prelude::MenuExt;
use fltk::{enums::*, menu::*, prelude::*, text::TextBuffer};
use rfd::FileDialog;
use std::{
    cell::RefCell,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::load_as_utf8;
use super::{
    SearchState,
    search::{SearchControls, update_result_status},
    status::{hide_search_controls, show_search_controls},
};

const MAX_RECENT_FILES: usize = 10;

fn recent_files_store_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return Path::new(&appdata)
                .join("plain_text")
                .join("recent_files.txt");
        }
    }

    if let Ok(home) = env::var("HOME") {
        return Path::new(&home)
            .join(".config")
            .join("plain_text")
            .join("recent_files.txt");
    }

    Path::new("recent_files.txt").to_path_buf()
}

fn save_recent_files(state: &Rc<RefCell<SearchState>>) {
    let path = recent_files_store_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let content = state.borrow().recent_files.join("\n");
    let _ = fs::write(path, content);
}

pub fn load_recent_files_into_state(state: &Rc<RefCell<SearchState>>) {
    let path = recent_files_store_path();
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    let mut recents: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();

    recents.dedup();
    if recents.len() > MAX_RECENT_FILES {
        recents.truncate(MAX_RECENT_FILES);
    }

    state.borrow_mut().recent_files = recents;
}

fn recent_item_label(path: &str, index: usize) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| path.to_string())
        .replace('/', "\\/");
    format!("{}. {}", index + 1, name)
}

fn remember_recent_path(state: &Rc<RefCell<SearchState>>, path: &str) {
    let mut s = state.borrow_mut();
    s.recent_files.retain(|p| p != path);
    s.recent_files.insert(0, path.to_string());
    if s.recent_files.len() > MAX_RECENT_FILES {
        s.recent_files.truncate(MAX_RECENT_FILES);
    }
    drop(s);
    save_recent_files(state);
}

fn open_path_into_editor(
    path: &str,
    buf: &Rc<RefCell<TextBuffer>>,
    stylebuf: &Rc<RefCell<TextBuffer>>,
    state: &Rc<RefCell<SearchState>>,
    update_status: &dyn Fn(),
) -> bool {
    if let Some(text) = load_as_utf8(path) {
        let len = text.len();

        buf.borrow_mut().set_text(&text);
        stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));

        state.borrow_mut().filepath = path.to_string();
        update_status();
        return true;
    }
    false
}

fn refresh_recent_menu<F>(
    menu: &mut MenuBar,
    buf: &Rc<RefCell<TextBuffer>>,
    stylebuf: &Rc<RefCell<TextBuffer>>,
    state: &Rc<RefCell<SearchState>>,
    update_status: &F,
) where
    F: Fn() + Clone + 'static,
{
    let mut submenu_idx = menu.find_index("Recent");
    if submenu_idx < 0 {
        menu.add("Recent", Shortcut::None, MenuFlag::Submenu, |_| {});
        submenu_idx = menu.find_index("Recent");
    }

    if submenu_idx >= 0 {
        let _ = menu.clear_submenu(submenu_idx);
    }

    let recents = state.borrow().recent_files.clone();
    if recents.is_empty() {
        menu.add(
            "Recent/(No recent files)",
            Shortcut::None,
            MenuFlag::Inactive,
            |_| {},
        );
        return;
    }

    for (idx, path) in recents.into_iter().enumerate() {
        let label = recent_item_label(&path, idx);

        let buf = Rc::clone(buf);
        let stylebuf = Rc::clone(stylebuf);
        let state = Rc::clone(state);
        let update_status = update_status.clone();
        let mut menu_ref = menu.clone();

        menu.add(
            &format!("Recent/{}", label),
            Shortcut::None,
            MenuFlag::Normal,
            move |_| {
                if open_path_into_editor(&path, &buf, &stylebuf, &state, &update_status) {
                    remember_recent_path(&state, &path);
                    refresh_recent_menu(&mut menu_ref, &buf, &stylebuf, &state, &update_status);
                }
            },
        );
    }
}

pub fn remember_recent_and_refresh<F>(
    menu: &mut MenuBar,
    buf: &Rc<RefCell<TextBuffer>>,
    stylebuf: &Rc<RefCell<TextBuffer>>,
    state: &Rc<RefCell<SearchState>>,
    update_status: &F,
    path: &str,
) where
    F: Fn() + Clone + 'static,
{
    remember_recent_path(state, path);
    refresh_recent_menu(menu, buf, stylebuf, state, update_status);
}

pub fn add_file_menu_items<F>(
    menu: &mut MenuBar,
    buf: &Rc<RefCell<TextBuffer>>,
    stylebuf: &Rc<RefCell<TextBuffer>>,
    state: &Rc<RefCell<SearchState>>,
    update_status: F,
) where
    F: Fn() + Clone + 'static,
{
    refresh_recent_menu(menu, buf, stylebuf, state, &update_status);

    {
        let buf = Rc::clone(buf);
        let stylebuf = Rc::clone(stylebuf);
        let state = Rc::clone(state);
        let update_status_open = update_status.clone();
        let mut menu_ref = menu.clone();

        menu.add(
            "File/Open\t",
            Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = FileDialog::new().pick_file() {
                    let path = path.to_string_lossy().to_string();
                    if open_path_into_editor(&path, &buf, &stylebuf, &state, &update_status_open) {
                        remember_recent_and_refresh(
                            &mut menu_ref,
                            &buf,
                            &stylebuf,
                            &state,
                            &update_status_open,
                            &path,
                        );
                    }
                }
            },
        );
    }

    {
        let buf = Rc::clone(buf);
        let state = Rc::clone(state);
        let update_status_save = update_status.clone();

        menu.add(
            "File/Save\t",
            Shortcut::Ctrl | 's',
            MenuFlag::Normal,
            move |_| {
                let current_path = state.borrow().filepath.clone();

                let path = if current_path.is_empty() {
                    FileDialog::new()
                        .save_file()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    Some(current_path)
                };

                if let Some(path) = path {
                    let text = buf.borrow().text();
                    let utf8 = text.as_bytes();
                    fs::write(&path, utf8).ok();

                    state.borrow_mut().filepath = path;
                    update_status_save();
                }
            },
        );
    }

    {
        let buf = Rc::clone(buf);
        let state = Rc::clone(state);
        let update_status_saveas = update_status.clone();

        menu.add(
            "File/Save As",
            Shortcut::None,
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = FileDialog::new().save_file() {
                    let text = buf.borrow().text();
                    let utf8 = text.as_bytes();
                    fs::write(&*path.to_string_lossy(), utf8).ok();

                    state.borrow_mut().filepath = path.to_string_lossy().to_string();
                    update_status_saveas();
                }
            },
        );
    }

    {
        let buf = Rc::clone(buf);
        let stylebuf = Rc::clone(stylebuf);
        let state = Rc::clone(state);
        let update_status_new = update_status.clone();

        menu.add(
            "File/New\t",
            Shortcut::Ctrl | 'n',
            MenuFlag::Normal,
            move |_| {
                buf.borrow_mut().set_text("");
                stylebuf.borrow_mut().set_text("A");
                state.borrow_mut().filepath.clear(); // mark as new file
                update_status_new();
            },
        );
    }

    menu.add(
        "File/Quit\t",
        Shortcut::Ctrl | 'q',
        MenuFlag::Normal,
        |_| fltk::app::quit(),
    );
}

pub fn add_search_menu<F>(
    menu: &mut MenuBar,
    state: &Rc<RefCell<SearchState>>,
    controls: &Rc<RefCell<SearchControls>>,
    editor: &fltk::text::TextEditor,
    buf: &Rc<RefCell<fltk::text::TextBuffer>>,
    update_status: F,
) where
    F: Fn() + Clone + 'static,
{
    {
        let state = Rc::clone(state);
        let controls = Rc::clone(controls);
        let update_status = update_status.clone();

        menu.add(
            "Search/Toggle Find\t",
            Shortcut::Ctrl | 'f',
            MenuFlag::Normal,
            move |_| {
                let mut s = state.borrow_mut();
                s.visible = !s.visible;
                if s.visible {
                    s.current = 0;
                    let mut sc = controls.borrow_mut();
                    show_search_controls(&mut sc);
                    sc.input.take_focus().ok();
                } else {
                    hide_search_controls(&mut controls.borrow_mut());
                }
                update_status();
            },
        );
    }

    {
        let state = Rc::clone(state);
        let controls = Rc::clone(controls);
        let mut editor = editor.clone();
        let buf = Rc::clone(buf);
        let update_status = update_status.clone();

        menu.add(
            "Search/Previous Match\t",
            Shortcut::Ctrl | 'j',
            MenuFlag::Normal,
            move |_| {
                let mut s = state.borrow_mut();
                if !s.results.is_empty() {
                    if s.current == 0 {
                        s.current = s.results.len() - 1;
                    } else {
                        s.current -= 1;
                    }
                    let (start, end) = s.results[s.current];
                    editor.set_insert_position(start);
                    editor.show_insert_position();
                    buf.borrow_mut().select(start, end);
                    update_result_status(&controls.borrow().results, &s);
                    update_status();
                }
            },
        );
    }

    {
        let state = Rc::clone(state);
        let controls = Rc::clone(controls);
        let mut editor = editor.clone();
        let buf = Rc::clone(buf);
        let update_status = update_status;

        menu.add(
            "Search/Next Match\t",
            Shortcut::Ctrl | 'k',
            MenuFlag::Normal,
            move |_| {
                let mut s = state.borrow_mut();
                if !s.results.is_empty() {
                    s.current = (s.current + 1) % s.results.len();
                    let (start, end) = s.results[s.current];
                    editor.set_insert_position(start);
                    editor.show_insert_position();
                    buf.borrow_mut().select(start, end);
                    update_result_status(&controls.borrow().results, &s);
                    update_status();
                }
            },
        );
    }
}
