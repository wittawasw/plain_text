use fltk::{frame::Frame, prelude::*};
use fltk::text::TextEditor;
use fltk::enums::Align;
use std::{cell::RefCell, rc::Rc};

use super::search::SearchState;

pub type StatusBar = Rc<RefCell<Frame>>;
pub type UpdateStatus = Rc<dyn Fn()>;

pub fn create_status_bar(x: i32, y: i32, w: i32, h: i32) -> StatusBar {
    let mut f = Frame::new(x, y, w, h, "");
    f.set_align(Align::Left | Align::Inside);
    Rc::new(RefCell::new(f))
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
        let display = if fp.is_empty() { "(untitled)".into() } else { fp };

        status_bar.borrow_mut().set_label(&format!(
            "Line {}, Col {}   |   {}",
            line + 1,
            col + 1,
            display
        ));
    })
}
