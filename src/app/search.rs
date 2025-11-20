// search.rs
use fltk::{
    enums::CallbackTrigger,
    button::*,
    frame::Frame,
    group::Group,
    input::*,
    prelude::*,
    text::{TextBuffer, TextEditor},
};
use std::{cell::RefCell, rc::Rc};

pub struct SearchState {
    pub results: Vec<(i32, i32)>,
    pub current: usize,
    pub visible: bool,
    pub filepath: String,
}

pub struct SearchUI {
    pub group: Rc<RefCell<Group>>,
    pub input: Input,
    pub case_btn: CheckButton,
    pub prev_btn: Button,
    pub next_btn: Button,
    pub status: Rc<RefCell<Frame>>,
}

pub fn create_search_ui(x: i32, y: i32, w: i32) -> SearchUI {
    let group = Rc::new(RefCell::new(Group::new(x, y, w, 30, "")));
    let input = Input::new(60, y + 5, 200, 20, "");
    let case_btn = CheckButton::new(270, y + 5, 90, 20, "Aa");
    let prev_btn = Button::new(370, y + 5, 60, 20, "Prev");
    let next_btn = Button::new(440, y + 5, 60, 20, "Next");
    let status = Rc::new(RefCell::new(Frame::new(510, y + 5, 200, 20, "")));

    {
        let g = group.borrow();
        Group::end(&*g);
    }
    group.borrow_mut().hide();

    SearchUI { group, input, case_btn, prev_btn, next_btn, status }
}

pub fn attach_search_logic(
    ui: &mut SearchUI,
    state: Rc<RefCell<SearchState>>,
    buf: Rc<RefCell<TextBuffer>>,
    stylebuf: Rc<RefCell<TextBuffer>>,
    editor: &mut TextEditor,
    update_status: Rc<dyn Fn()>,
) {
    let do_search: Rc<dyn Fn(String, bool) -> Vec<(i32, i32)>> = {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let status = Rc::clone(&ui.status);

        Rc::new(move |pattern: String, cs: bool| -> Vec<(i32, i32)> {
            let text = buf.borrow().text();
            let len = text.len();
            stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));
            if pattern.is_empty() {
                status.borrow_mut().set_label("");
                return vec![];
            }

            let hay = if cs { text.clone() } else { text.to_lowercase() };
            let needle = if cs { pattern.clone() } else { pattern.to_lowercase() };

            let mut out = vec![];
            let mut pos = 0;

            while let Some(found) = hay[pos..].find(&needle) {
                let s = (pos + found) as i32;
                let e = s + pattern.len() as i32;
                out.push((s, e));
                pos = e as usize;
            }

            {
                let mut sb = stylebuf.borrow_mut();
                for (s, e) in &out {
                    for i in *s..*e {
                        if (i as usize) < len {
                            sb.replace(i, i + 1, "B");
                        }
                    }
                }
            }

            status.borrow_mut().set_label(&format!("{} match(es)", out.len()));
            out
        })
    };

    let goto_match = {
        let buf = Rc::clone(&buf);
        let mut ed = editor.clone();
        let update_status = update_status.clone();

        move |s: i32, e: i32| {
            ed.set_insert_position(s);
            ed.show_insert_position();
            buf.borrow_mut().select(s, e);
            update_status();
        }
    };

    {
        let do_search = Rc::clone(&do_search);
        let state = Rc::clone(&state);
        let case_btn = ui.case_btn.clone();
        let mut ed = editor.clone();
        let buf = Rc::clone(&buf);
        let update_status = update_status.clone();

        ui.input.set_trigger(CallbackTrigger::Changed);
        ui.input.set_callback(move |inp| {
            let res = do_search(inp.value(), case_btn.value());
            let mut st = state.borrow_mut();
            st.results = res;
            st.current = 0;

            if let Some((s, e)) = st.results.get(0) {
                ed.set_insert_position(*s);
                ed.show_insert_position();
                buf.borrow_mut().select(*s, *e);
            }

            update_status();
        });
    }

    {
        let state = Rc::clone(&state);
        let mut goto = goto_match.clone();

        ui.next_btn.set_callback(move |_| {
            let mut st = state.borrow_mut();
            if !st.results.is_empty() {
                st.current = (st.current + 1) % st.results.len();
                let (s, e) = st.results[st.current];
                goto(s, e);
            }
        });
    }

    {
        let state = Rc::clone(&state);
        let mut goto = goto_match.clone();

        ui.prev_btn.set_callback(move |_| {
            let mut st = state.borrow_mut();
            if !st.results.is_empty() {
                if st.current == 0 {
                    st.current = st.results.len() - 1;
                } else {
                    st.current -= 1;
                }
                let (s, e) = st.results[st.current];
                goto(s, e);
            }
        });
    }
}
