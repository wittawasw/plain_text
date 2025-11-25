use fltk::{
    app,
    enums::*,
    menu::MenuBar,
    prelude::*,
    text::{self, StyleTableEntry, TextBuffer, TextEditor},
    window::Window,
};
use std::{cell::RefCell, rc::Rc};

mod icon;
mod menu;
mod search;
mod status;
mod encoding;

use search::{create_search_ui, attach_search_logic, SearchState};
use status::{create_status_bar, make_update_status};
use encoding::load_as_utf8;

pub fn run() {
    let app = app::App::default();
    let mut win = Window::new(100, 100, 800, 600, "PlainText");
    let ico = icon::load_app_icon();

    win.set_icon(Some(ico));
    win.make_resizable(true);

    let mut menu = MenuBar::new(0, 0, 800, 30, "");

    let buf = Rc::new(RefCell::new(TextBuffer::default()));

    let mut editor = TextEditor::new(0, 60, 800, 510, "");
    editor.set_buffer(Some(buf.borrow().clone()));
    editor.set_scrollbar_size(16);
    editor.wrap_mode(text::WrapMode::AtBounds, 0);
    win.resizable(&editor);

    let status_bar = create_status_bar(0, 570, 800, 30);

    let stylebuf = Rc::new(RefCell::new(TextBuffer::default()));
    let styles = vec![
        StyleTableEntry { color: fltk::enums::Color::Black, font: fltk::enums::Font::Helvetica, size: app::font_size() },
        StyleTableEntry { color: fltk::enums::Color::Black, font: fltk::enums::Font::HelveticaBold, size: app::font_size() },
    ];
    editor.set_highlight_data(stylebuf.borrow().clone(), styles);

    let search_state = Rc::new(RefCell::new(SearchState {
        results: vec![],
        current: 0,
        visible: false,
        filepath: "".into(),
    }));

    let mut search_ui = create_search_ui(0, 30, 800);

    let update_status = make_update_status(&status_bar, &editor, &search_state);

    {
        let buf = Rc::clone(&buf);
        let stylebuf = Rc::clone(&stylebuf);
        let update_status = update_status.clone();
        let state = Rc::clone(&search_state);

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

                    return true;
                }
                false
            }
            Event::KeyDown | Event::KeyUp | Event::Push | Event::Released | Event::Drag | Event::MouseWheel => {
                update_status();
                false
            }
            _ => false,
        });
    }

    attach_search_logic(
        &mut search_ui,
        Rc::clone(&search_state),
        Rc::clone(&buf),
        Rc::clone(&stylebuf),
        &mut editor,
        update_status.clone(),
    );

    win.handle({
        let search_state = Rc::clone(&search_state);
        let search_ui_group = search_ui.group.clone();
        let mut editor = editor.clone();
        let buf = Rc::clone(&buf);
        let update_status = update_status.clone();
        let mut win_clone = win.clone();

        move |_, ev| match ev {
            Event::Shortcut => {
                let key = app::event_key();
                let st = app::event_state();

                if key == Key::from_char('f') && (st.contains(EventState::Ctrl) || st.contains(EventState::Meta)) {
                    let mut s = search_state.borrow_mut();
                    s.visible = !s.visible;

                    if s.visible {
                        search_ui_group.borrow_mut().resize(0, 30, 800, 30);
                        search_ui_group.borrow_mut().show();
                        editor.resize(0, 60, 800, 510);
                        search_ui.input.take_focus().ok();
                    } else {
                        search_ui_group.borrow_mut().resize(0, 30, 800, 0);
                        search_ui_group.borrow_mut().hide();
                        editor.resize(0, 30, 800, 540);
                    }

                    win_clone.redraw();
                    return true;
                }
                false
            }

            Event::KeyDown => {
                let key = app::event_key();
                let mut st = search_state.borrow_mut();

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
        }
    });

    menu::add_file_menu_items(
        &mut menu,
        &buf,
        &stylebuf,
        &search_state,
        {
            let update_status = update_status.clone();
            move || (update_status)()
        }
    );

    editor.set_callback({
        let update_status = update_status.clone();
        move |_| update_status()
    });

    win.end();

    {
        let mut editor = editor.clone();
        let status_bar = Rc::clone(&status_bar);
        let search_ui_group = search_ui.group.clone();
        let search_state = Rc::clone(&search_state);

        win.resize_callback(move |_win, _x, _y, w, h| {
            let search_h = if search_state.borrow().visible { 30 } else { 0 };
            let editor_y = 30 + search_h;
            let editor_h = h - editor_y - 30;

            search_ui_group.borrow_mut().resize(0, 30, w, search_h);
            editor.resize(0, editor_y, w, editor_h);
            status_bar.borrow_mut().resize(0, h - 30, w, 30);
        });
    }

    win.show();
    app.run().unwrap();
}
