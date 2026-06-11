use fltk::{
    app,
    enums::*,
    menu::MenuBar,
    prelude::*,
    text::{self, StyleTableEntry, TextBuffer, TextEditor},
    window::Window,
};
use std::{cell::RefCell, rc::Rc};

mod encoding;
mod icon;
mod menu;
mod search;
mod status;

use encoding::load_as_utf8;
use search::{SearchState, attach_search_logic, update_result_status};
use status::{
    attach_status_path_actions, create_status_bar, hide_search_controls, make_update_status,
    show_search_controls,
};

pub fn run() {
    let app = app::App::default();
    let mut win = Window::new(100, 100, 800, 600, "PlainText");
    let ico = icon::load_app_icon();

    win.set_icon(Some(ico));
    win.make_resizable(true);

    let mut menu = MenuBar::new(0, 0, 800, 30, "");

    let buf = Rc::new(RefCell::new(TextBuffer::default()));

    let mut editor = TextEditor::new(0, 30, 800, 510, "");
    editor.set_buffer(Some(buf.borrow().clone()));
    editor.set_scrollbar_size(16);
    editor.wrap_mode(text::WrapMode::AtBounds, 0);
    editor.set_text_font(fltk::enums::Font::Courier);
    editor.remove_key_binding(Key::from_char('f'), Shortcut::Ctrl);
    win.resizable(&editor);

    let status_bar = create_status_bar(0, 570, 800, 30);

    let sb = status_bar.borrow();
    let sb_w = sb.w();
    let sb_y = sb.y();
    drop(sb);

    let search_state = Rc::new(RefCell::new(SearchState {
        results: vec![],
        current: 0,
        visible: false,
        filepath: "".into(),
        recent_files: vec![],
    }));
    menu::load_recent_files_into_state(&search_state);

    let search_controls = Rc::new(RefCell::new(search::create_search_controls(sb_y, sb_w)));

    let stylebuf = Rc::new(RefCell::new(TextBuffer::default()));
    let styles = vec![
        StyleTableEntry {
            color: fltk::enums::Color::Black,
            font: fltk::enums::Font::Helvetica,
            size: app::font_size(),
        },
        StyleTableEntry {
            color: fltk::enums::Color::Black,
            font: fltk::enums::Font::HelveticaBold,
            size: app::font_size(),
        },
    ];
    editor.set_highlight_data(stylebuf.borrow().clone(), styles);

    let update_status = make_update_status(&status_bar, &editor, &search_state);
    attach_status_path_actions(&status_bar, &search_state);

    {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let update_status = update_status.clone();
        let state = Rc::clone(&search_state);
        let mut recent_menu = menu.clone();

        editor.handle(move |_, ev| match ev {
            Event::Paste => {
                let dropped = app::event_text().trim().to_string();
                if dropped.is_empty() {
                    return false;
                }
                if let Some(content) = load_as_utf8(&dropped) {
                    let len = content.len();

                    buf.borrow_mut().set_text(&content);
                    stylebuf.borrow_mut().set_text(&"A".repeat(len.max(1)));

                    state.borrow_mut().filepath = dropped.clone();
                    update_status();
                    let update_status_recent = update_status.clone();
                    let recent_status_cb = move || (update_status_recent)();
                    menu::remember_recent_and_refresh(
                        &mut recent_menu,
                        &buf,
                        &stylebuf,
                        &state,
                        &recent_status_cb,
                        &dropped,
                    );

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
    let search_editor = editor.clone();
    let search_buf = buf.clone();
    let search_state_clone = search_state.clone();
    let search_update_status = update_status.clone();

    attach_search_logic(
        &mut search_controls.borrow_mut(),
        Rc::clone(&search_state),
        Rc::clone(&buf),
        Rc::clone(&stylebuf),
        &mut editor,
        update_status.clone(),
    );

    win.handle({
        let search_state = search_state_clone;
        let search_controls = Rc::clone(&search_controls);
        let _status_bar = Rc::clone(&status_bar);
        let mut editor = search_editor;
        let buf = search_buf;
        let update_status = search_update_status;

        move |_, ev| match ev {
            Event::Shortcut | Event::KeyDown => {
                let key = app::event_key();
                let st = app::event_state();
                let command = st.contains(EventState::Ctrl) || st.contains(EventState::Meta);
                let text = app::event_text();
                let ctrl_j = command && (key == Key::from_char('j') || text == "\n");
                let ctrl_k = command && (key == Key::from_char('k') || text == "\u{b}");

                if ctrl_j {
                    let mut s = search_state.borrow_mut();
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
                        update_result_status(&search_controls.borrow().results, &s);
                        update_status();
                    }
                    return true;
                }

                if ctrl_k {
                    let mut s = search_state.borrow_mut();
                    if !s.results.is_empty() {
                        s.current = (s.current + 1) % s.results.len();
                        let (start, end) = s.results[s.current];
                        editor.set_insert_position(start);
                        editor.show_insert_position();
                        buf.borrow_mut().select(start, end);
                        update_result_status(&search_controls.borrow().results, &s);
                        update_status();
                    }
                    return true;
                }

                if ev != Event::KeyDown {
                    return false;
                }

                let key = app::event_key();
                let st = app::event_state();
                let mut s = search_state.borrow_mut();

                if s.visible && !s.results.is_empty() {
                    if key == Key::Down {
                        s.current = (s.current + 1) % s.results.len();
                        let (start, end) = s.results[s.current];
                        editor.set_insert_position(start);
                        editor.show_insert_position();
                        buf.borrow_mut().select(start, end);
                        update_result_status(&search_controls.borrow().results, &s);
                        update_status();
                        return true;
                    }

                    if key == Key::Up {
                        if s.current == 0 {
                            s.current = s.results.len() - 1;
                        } else {
                            s.current -= 1;
                        }
                        let (start, end) = s.results[s.current];
                        editor.set_insert_position(start);
                        editor.show_insert_position();
                        buf.borrow_mut().select(start, end);
                        update_result_status(&search_controls.borrow().results, &s);
                        update_status();
                        return true;
                    }

                    if key == Key::Enter {
                        let shift = st.contains(EventState::Shift);
                        if shift {
                            if s.current == 0 {
                                s.current = s.results.len() - 1;
                            } else {
                                s.current -= 1;
                            }
                        } else {
                            s.current = (s.current + 1) % s.results.len();
                        }
                        let (start, end) = s.results[s.current];
                        editor.set_insert_position(start);
                        editor.show_insert_position();
                        buf.borrow_mut().select(start, end);
                        update_result_status(&search_controls.borrow().results, &s);
                        update_status();
                        return true;
                    }
                }

                false
            }

            _ => false,
        }
    });

    menu::add_file_menu_items(&mut menu, &buf, &stylebuf, &search_state, {
        let update_status = update_status.clone();
        move || (update_status)()
    });

    menu::add_search_menu(&mut menu, &search_state, &search_controls, &editor, &buf, {
        let update_status = update_status.clone();
        move || (update_status)()
    });

    editor.set_callback({
        let update_status = update_status.clone();
        move |_| update_status()
    });

    win.end();

    {
        let mut editor = editor.clone();
        let status_bar = Rc::clone(&status_bar);
        let search_state = Rc::clone(&search_state);

        win.resize_callback(move |_win, _x, _y, w, h| {
            status_bar.borrow_mut().resize(0, h - 30, w, 30);
            editor.resize(0, 30, w, h - 30 - 30);

            let sb = status_bar.borrow();
            let sb_w = sb.w();
            let sb_y = sb.y();
            drop(sb);

            if search_state.borrow().visible {
                let mut sc = search_controls.borrow_mut();
                if w < 300 {
                    hide_search_controls(&mut *sc);
                    search_state.borrow_mut().visible = false;
                } else {
                    show_search_controls(&mut *sc);
                    sc.input.set_pos(sb_w - 200, sb_y + 5);
                    sc.results.borrow_mut().set_pos(sb_w - 275, sb_y + 5);
                }
            }
        });
    }

    win.show();
    app.run().unwrap();
}
