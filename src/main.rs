#![windows_subsystem = "windows"]

use fltk::{
    app,
    dialog,
    enums::*,
    menu::{MenuBar, MenuFlag},
    prelude::*,
    text::*,
    window::Window,
};
use std::fs;

fn main() {
    let app = app::App::default();

    let mut win = Window::new(100, 100, 800, 600, "PlainText");
    win.set_color(Color::White);

    let mut menu = MenuBar::new(0, 0, 800, 30, "");

    let buf = TextBuffer::default();
    let mut editor = TextEditor::new(0, 30, 800, 570, "");
    editor.set_buffer(Some(buf.clone()));
    editor.set_scrollbar_size(16);
    editor.wrap_mode(WrapMode::AtBounds, 0);

    {
        let mut buf = buf.clone();
        menu.add(
            "File/Open\t",
            Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = dialog::file_chooser("Open File", "*", ".", false) {
                    if let Ok(content) = fs::read_to_string(path) {
                        buf.set_text(&content);
                    }
                }
            },
        );
    }

    {
        let buf = buf.clone();
        menu.add(
            "File/Save As\t",
            Shortcut::Ctrl | 's',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = dialog::file_chooser("Save File", "*", ".", true) {
                    let text = buf.text();
                    let _ = fs::write(path, text);
                }
            },
        );
    }

    menu.add(
        "File/Quit\t",
        Shortcut::Ctrl | 'q',
        MenuFlag::Normal,
        |_| {
            app::quit();
        },
    );

    {
        let mut buf = buf.clone();

        win.handle(move |_, ev| match ev {
            Event::DndEnter | Event::DndDrag | Event::DndRelease => true,

            Event::Paste => {
                let dropped = app::event_text();

                let path = dropped.trim();

                if let Ok(content) = fs::read_to_string(path) {
                    buf.set_text(&content);
                }

                true
            }

            _ => false,
        });
    }

    win.end();
    win.show();
    app.run().unwrap();
}
