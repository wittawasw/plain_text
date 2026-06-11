use fltk::{
    enums::{Align, CallbackTrigger, Event, EventState, Key},
    frame::Frame,
    input::Input,
    prelude::*,
    text::{TextBuffer, TextEditor},
};
use std::{cell::RefCell, rc::Rc};

pub struct SearchState {
    pub results: Vec<(i32, i32)>,
    pub current: usize,
    pub visible: bool,
    pub filepath: String,
    pub recent_files: Vec<String>,
}

pub struct SearchControls {
    pub input: Input,
    pub results: Rc<RefCell<Frame>>,
}

pub fn update_result_status(results: &Rc<RefCell<Frame>>, state: &SearchState) {
    if state.results.is_empty() {
        results.borrow_mut().set_label("");
    } else {
        results
            .borrow_mut()
            .set_label(&format!("{} of {}", state.current + 1, state.results.len()));
    }
}

pub fn create_search_controls(status_bar_w: i32) -> SearchControls {
    let sb_x = status_bar_w - 280;
    let _ = sb_x;

    let results = Rc::new(RefCell::new(Frame::new(sb_x + 5, 5, 70, 20, "")));
    results
        .borrow_mut()
        .set_color(fltk::enums::Color::from_rgb(240, 240, 240));
    results
        .borrow_mut()
        .set_label_color(fltk::enums::Color::from_rgb(100, 100, 100));
    results.borrow_mut().set_align(Align::Left | Align::Inside);

    let mut input = Input::new(sb_x + 80, 5, 200, 20, "");
    input.set_text_color(fltk::enums::Color::Black);
    input.set_text_size(12);
    input.hide();
    results.borrow_mut().hide();

    SearchControls { input, results }
}

pub fn attach_search_logic(
    ui: &mut SearchControls,
    state: Rc<RefCell<SearchState>>,
    buf: Rc<RefCell<TextBuffer>>,
    stylebuf: Rc<RefCell<TextBuffer>>,
    editor: &mut TextEditor,
    update_status: Rc<dyn Fn()>,
) {
    let do_search: Rc<dyn Fn(String, bool) -> Vec<(i32, i32)>> = {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let status = Rc::clone(&ui.results);

        Rc::new(move |pattern: String, cs: bool| -> Vec<(i32, i32)> {
            let text = buf.borrow().text();
            let len = text.len();
            stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));
            if pattern.is_empty() {
                status.borrow_mut().set_label("");
                return vec![];
            }

            let hay = if cs {
                text.clone()
            } else {
                text.to_lowercase()
            };
            let needle = if cs {
                pattern.clone()
            } else {
                pattern.to_lowercase()
            };

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

            if out.is_empty() {
                status.borrow_mut().set_label("");
            } else {
                status.borrow_mut().set_label(&format!("{} of {}", 1, out.len()));
            }
            out
        })
    };

    let goto_match = {
        let buf = Rc::clone(&buf);
        let mut ed = editor.clone();
        let update_status = update_status.clone();
        let status = Rc::clone(&ui.results);
        let state = Rc::clone(&state);

        move |s: i32, e: i32| {
            ed.set_insert_position(s);
            ed.show_insert_position();
            buf.borrow_mut().select(s, e);
            update_result_status(&status, &state.borrow());
            update_status();
        }
    };

    {
        let do_search = Rc::clone(&do_search);
        let state = Rc::clone(&state);
        let mut ed = editor.clone();
        let buf = Rc::clone(&buf);
        let update_status = update_status.clone();
        let status = Rc::clone(&ui.results);

        ui.input.set_trigger(CallbackTrigger::Changed);
        ui.input.set_callback(move |inp| {
            let res = do_search(inp.value(), false);
            let mut st = state.borrow_mut();
            st.results = res;
            st.current = 0;
            update_result_status(&status, &st);

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
        let mut input = ui.input.clone();
        let results = Rc::clone(&ui.results);
        let update_status = update_status.clone();

        ui.input.handle(move |_, ev| match ev {
            Event::KeyDown | Event::Shortcut => {
                let key = fltk::app::event_key();
                let st = fltk::app::event_state();
                let command = st.contains(EventState::Ctrl) || st.contains(EventState::Meta);
                let text = fltk::app::event_text();
                let ctrl_f = command && key == Key::from_char('f');
                let ctrl_j = command && (key == Key::from_char('j') || text == "\n");
                let ctrl_k = command && (key == Key::from_char('k') || text == "\u{b}");

                if ctrl_f {
                    let mut st = state.borrow_mut();
                    st.visible = !st.visible;
                    if st.visible {
                        st.current = 0;
                        input.show();
                        results.borrow_mut().show();
                        input.take_focus().ok();
                    } else {
                        input.hide();
                        results.borrow_mut().hide();
                    }
                    update_status();
                    return true;
                }

                if ctrl_j {
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
                    return true;
                }

                if ctrl_k {
                    let mut st = state.borrow_mut();
                    if !st.results.is_empty() {
                        st.current = (st.current + 1) % st.results.len();
                        let (s, e) = st.results[st.current];
                        goto(s, e);
                    }
                    return true;
                }

                if command {
                    return false;
                }

                if ev != Event::KeyDown {
                    return false;
                }

                if key == Key::Enter {
                    if st.contains(EventState::Shift) {
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
                        return true;
                    } else {
                        let mut st = state.borrow_mut();
                        if !st.results.is_empty() {
                            st.current = (st.current + 1) % st.results.len();
                            let (s, e) = st.results[st.current];
                            goto(s, e);
                        }
                        return true;
                    }
                }

                false
            }
            _ => false,
        });
    }
}
