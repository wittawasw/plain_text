#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use fltk::{
    app,
    dialog,
    enums::*,
    button::*,
    frame::Frame,
    group::Group,
    input::*,
    menu::*,
    prelude::*,
    text::{self, StyleTableEntry, TextBuffer, TextEditor},
    window::Window,
};
use std::{cell::RefCell, fs, rc::Rc};

struct SearchState {
    results: Vec<(i32, i32)>,
    current: usize,
    visible: bool,
    filepath: String,
}

fn main() {
    let app = app::App::default();

    let mut win = Window::new(100, 100, 800, 600, "PlainText");
    win.make_resizable(true);
    win.set_color(Color::White);

    let mut menu = MenuBar::new(0, 0, 800, 30, "");

    let buf = Rc::new(RefCell::new(TextBuffer::default()));

    let mut editor = TextEditor::new(0, 60, 800, 510, "");
    editor.set_buffer(Some(buf.borrow().clone()));
    editor.set_scrollbar_size(16);
    editor.wrap_mode(text::WrapMode::AtBounds, 0);
    editor.set_text_font(Font::Helvetica);
    editor.set_text_size(app::font_size());

    let status_bar = Rc::new(RefCell::new(Frame::new(0, 570, 800, 30, "")));
    {
        let mut sb = status_bar.borrow_mut();
        sb.set_color(Color::from_hex(0xf0f0f0));
        sb.set_frame(FrameType::FlatBox);
        sb.set_label_color(Color::Black);
        sb.set_align(Align::Left | Align::Inside);
    }
    status_bar.borrow_mut().set_label("Ready");

    let search_group = Rc::new(RefCell::new(Group::new(0, 30, 800, 30, "")));

    let mut search_input = Input::new(60, 35, 200, 20, "");
    let mut case_btn = CheckButton::new(270, 35, 90, 20, "Aa");
    let mut prev_btn = Button::new(370, 35, 60, 20, "Prev");
    let mut next_btn = Button::new(440, 35, 60, 20, "Next");
    let status = Rc::new(RefCell::new(Frame::new(510, 35, 200, 20, "")));

    {
        let sg = search_group.borrow();
        Group::end(&*sg);
    }
    search_group.borrow_mut().resize(0, 30, 800, 0);
    search_group.borrow_mut().hide();

    editor.resize(0, 30, 800, 570);

    let stylebuf = Rc::new(RefCell::new(TextBuffer::default()));
    let styles = vec![
        StyleTableEntry {
            color: Color::Black,
            font: Font::Helvetica,
            size: app::font_size(),
        },
        StyleTableEntry {
            color: Color::Black,
            font: Font::HelveticaBold,
            size: app::font_size(),
        },
    ];
    editor.set_highlight_data(stylebuf.borrow().clone(), styles);

    let state = Rc::new(RefCell::new(SearchState {
        results: vec![],
        current: 0,
        visible: false,
        filepath: "".to_string(),
    }));

    let update_status = {
        let status_bar = Rc::clone(&status_bar);
        let state = Rc::clone(&state);
        let editor = editor.clone();

        move || {
            let pos = editor.insert_position();

            let line = editor.count_lines(0, pos, false);
            let col = pos - editor.line_start(pos);

            let filepath = state.borrow().filepath.clone();
            let path_display = if filepath.is_empty() {
                "(untitled)".to_string()
            } else {
                filepath
            };

            status_bar.borrow_mut().set_label(&format!(
                "Line {}, Col {}   |   {}",
                line + 1,
                col + 1,
                path_display
            ));
        }
    };

    {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let state = Rc::clone(&state);
        let update_status = update_status.clone();

        editor.handle(move |_, ev| match ev {
            Event::DndEnter | Event::DndDrag | Event::DndRelease => true,

            Event::Paste => {
                let dropped = app::event_text().trim().to_string();
                if dropped.is_empty() {
                    return false;
                }
                if let Ok(content) = fs::read_to_string(&dropped) {
                    let len = content.len();
                    buf.borrow_mut().set_text(&content);
                    stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));
                    state.borrow_mut().filepath = dropped.clone();
                    update_status();
                    return true;
                }
                false
            }

            Event::KeyDown
            | Event::KeyUp
            | Event::Push
            | Event::Released
            | Event::Drag
            | Event::MouseWheel => {
                update_status();
                false
            }

            _ => false,
        });
    }

    let do_search: Rc<dyn Fn(String, bool) -> Vec<(i32, i32)>> = {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let status = Rc::clone(&status);

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
                    let mut i = *s;
                    while i < *e && (i as usize) < len {
                        sb.replace(i, i + 1, "B");
                        i += 1;
                    }
                }
            }

            status.borrow_mut().set_label(&format!("{} match(es)", out.len()));
            out
        })
    };

    let goto_match = {
        let buf = Rc::clone(&buf);
        let mut editor = editor.clone();
        let update_status = update_status.clone();

        move |s: i32, e: i32| {
            editor.set_insert_position(s);
            editor.show_insert_position();
            buf.borrow_mut().select(s, e);
            update_status();
        }
    };

    {
        let state = Rc::clone(&state);
        let do_search = Rc::clone(&do_search);
        let case_btn = case_btn.clone();
        let buf = Rc::clone(&buf);
        let mut editor = editor.clone();
        let update_status = update_status.clone();

        search_input.set_trigger(CallbackTrigger::Changed);
        search_input.set_callback(move |inp| {
            let res = do_search(inp.value(), case_btn.value());
            let mut st = state.borrow_mut();
            st.results = res;
            st.current = 0;

            if let Some((s, e)) = st.results.get(st.current) {
                editor.set_insert_position(*s);
                editor.show_insert_position();
                buf.borrow_mut().select(*s, *e);
            }

            update_status();
        });
    }

    {
        let state = Rc::clone(&state);
        let mut goto = goto_match.clone();

        next_btn.set_callback(move |_| {
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

        prev_btn.set_callback(move |_| {
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

    {
        let search_group = Rc::clone(&search_group);
        let state = Rc::clone(&state);
        let mut search_input = search_input.clone();
        let buf = Rc::clone(&buf);
        let mut editor = editor.clone();
        let mut win_clone = win.clone();
        let update_status = update_status.clone();

        win.handle(move |_, ev| match ev {
            Event::Shortcut => {
                let key = app::event_key();
                let st = app::event_state();
                let ctrl = st.contains(EventState::Ctrl);
                let cmd = st.contains(EventState::Meta);

                if key == Key::from_char('f') && (ctrl || cmd) {
                    let mut s = state.borrow_mut();
                    s.visible = !s.visible;

                    if s.visible {
                        search_group.borrow_mut().resize(0, 30, 800, 30);
                        search_group.borrow_mut().show();
                        editor.resize(0, 60, 800, 510);
                        search_input.take_focus().ok();
                    } else {
                        search_group.borrow_mut().resize(0, 30, 800, 0);
                        search_group.borrow_mut().hide();
                        editor.resize(0, 30, 800, 540);
                    }

                    win_clone.redraw();
                    return true;
                }
                false
            }

            Event::KeyDown => {
                let key = app::event_key();
                let mut st = state.borrow_mut();

                if st.visible && !st.results.is_empty() {
                    if key == Key::Down {
                        st.current = (st.current + 1) % st.results.len();
                        let (s, e) = st.results[st.current];
                        editor.set_insert_position(s);
                        editor.show_insert_position();
                        buf.borrow_mut().select(s, e);
                        update_status();
                        return true;
                    }

                    if key == Key::Up {
                        if st.current == 0 {
                            st.current = st.results.len() - 1;
                        } else {
                            st.current -= 1;
                        }
                        let (s, e) = st.results[st.current];
                        editor.set_insert_position(s);
                        editor.show_insert_position();
                        buf.borrow_mut().select(s, e);
                        update_status();
                        return true;
                    }
                }
                false
            }

            _ => false,
        });
    }

    {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let state = Rc::clone(&state);
        let update_status = update_status.clone();

        menu.add("File/Open\t", Shortcut::Ctrl | 'o', MenuFlag::Normal, move |_| {
            if let Some(path) = dialog::file_chooser("Open File", "*", ".", false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let len = content.len();
                    buf.borrow_mut().set_text(&content);
                    stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));
                    state.borrow_mut().filepath = path.clone();
                    update_status();
                }
            }
        });
    }

    {
        let buf = Rc::clone(&buf);
        let state = Rc::clone(&state);
        let update_status = update_status.clone();

        menu.add("File/Save As\t", Shortcut::Ctrl | 's', MenuFlag::Normal, move |_| {
            if let Some(path) = dialog::file_chooser("Save File", "*", ".", true) {
                let text = buf.borrow().text();
                let _ = fs::write(&path, text);
                state.borrow_mut().filepath = path.clone();
                update_status();
            }
        });
    }

    menu.add("File/Quit\t", Shortcut::Ctrl | 'q', MenuFlag::Normal, |_| app::quit());

    {
        let update_status = update_status.clone();
        editor.set_callback(move |_| {
            update_status();
        });
    }

    win.end();

    {
        let mut editor = editor.clone();
        let status_bar = Rc::clone(&status_bar);
        let search_group = Rc::clone(&search_group);
        let state = Rc::clone(&state);

        win.resize_callback(move |_win, _x, _y, w, h| {
            let search_h = if state.borrow().visible { 30 } else { 0 };

            let editor_y = 30 + search_h;
            let editor_h = h - editor_y - 30;

            search_group.borrow_mut().resize(0, 30, w, search_h);
            editor.resize(0, editor_y, w, editor_h);
            status_bar.borrow_mut().resize(0, h - 30, w, 30);
        });
    }

    win.show();
    app.run().unwrap();
}
