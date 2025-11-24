use fltk::{
    dialog,
    enums::*,
    menu::*,
    text::TextBuffer,
};
use fltk::prelude::MenuExt;
use std::{cell::RefCell, fs, rc::Rc};

use super::SearchState;

pub fn add_file_menu_items<F>(
    menu: &mut MenuBar,
    buf: &Rc<RefCell<TextBuffer>>,
    stylebuf: &Rc<RefCell<TextBuffer>>,
    state: &Rc<RefCell<SearchState>>,
    update_status: F,
)
where
    F: Fn() + Clone + 'static,
{
    {
        let buf = Rc::clone(buf);
        let stylebuf = Rc::clone(stylebuf);
        let state = Rc::clone(state);
        let update_status_open = update_status.clone();

        menu.add("File/Open\t", Shortcut::Ctrl | 'o', MenuFlag::Normal, move |_| {
            if let Some(path) = dialog::file_chooser("Open File", "*", ".", false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let len = content.len();
                    buf.borrow_mut().set_text(&content);
                    stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));
                    state.borrow_mut().filepath = path.clone();
                    update_status_open();
                }
            }
        });
    }

    {
        let buf = Rc::clone(buf);
        let state = Rc::clone(state);
        let update_status_save = update_status.clone();

        menu.add("File/Save\t", Shortcut::Ctrl | 's', MenuFlag::Normal, move |_| {
            let current_path = state.borrow().filepath.clone();

            // If no existing file, fallback to Save As
            let path = if current_path.is_empty() {
                dialog::file_chooser("Save File", "*", ".", true)
            } else {
                Some(current_path)
            };

            if let Some(path) = path {
                let text = buf.borrow().text();
                let _ = fs::write(&path, text);
                state.borrow_mut().filepath = path.clone();
                update_status_save();
            }
        });
    }

    {
        let buf = Rc::clone(buf);
        let state = Rc::clone(state);
        let update_status_saveas = update_status.clone();

        menu.add("File/Save As", Shortcut::None, MenuFlag::Normal, move |_| {
            if let Some(path) = dialog::file_chooser("Save File", "*", ".", true) {
                let text = buf.borrow().text();
                let _ = fs::write(&path, text);
                state.borrow_mut().filepath = path.clone();
                update_status_saveas();
            }
        });
    }

    {
        let buf = Rc::clone(buf);
        let stylebuf = Rc::clone(stylebuf);
        let state = Rc::clone(state);
        let update_status_new = update_status.clone();

        menu.add("File/New\t", Shortcut::Ctrl | 'n', MenuFlag::Normal, move |_| {
            buf.borrow_mut().set_text("");
            stylebuf.borrow_mut().set_text("A");
            state.borrow_mut().filepath.clear(); // mark as new file
            update_status_new();
        });
    }

    menu.add("File/Quit\t", Shortcut::Ctrl | 'q', MenuFlag::Normal, |_| {
        fltk::app::quit()
    });
}
