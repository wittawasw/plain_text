use fltk::prelude::MenuExt;
use fltk::{enums::*, menu::*, prelude::*, text::TextBuffer};
use rfd::FileDialog;
use std::{cell::RefCell, fs, rc::Rc};

use super::load_as_utf8;
use super::{
    SearchState,
    search::{SearchControls, update_result_status},
    status::{hide_search_controls, show_search_controls},
};

pub fn add_file_menu_items<F>(
    menu: &mut MenuBar,
    buf: &Rc<RefCell<TextBuffer>>,
    stylebuf: &Rc<RefCell<TextBuffer>>,
    state: &Rc<RefCell<SearchState>>,
    update_status: F,
) where
    F: Fn() + Clone + 'static,
{
    {
        let buf = Rc::clone(buf);
        let stylebuf = Rc::clone(stylebuf);
        let state = Rc::clone(state);
        let update_status_open = update_status.clone();

        menu.add(
            "File/Open\t",
            Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = FileDialog::new().pick_file() {
                    if let Some(text) = load_as_utf8(&path.to_string_lossy()) {
                        let len = text.len();

                        buf.borrow_mut().set_text(&text);
                        stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));

                        state.borrow_mut().filepath = path.to_string_lossy().to_string();
                        update_status_open();
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
