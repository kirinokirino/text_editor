use minifb::{Key, KeyRepeat, Window, WindowOptions};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::rc::Rc;

mod common;
mod editor;

use editor::Editor;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let mut window =
        Window::new("game", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{:?}", e);
        });
    let keys_data = KeyVec::new(RefCell::new(Vec::new()));
    let input = Box::new(Input::new(&keys_data));
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.set_input_callback(input);

    let mut editor = Editor::new(WIDTH, HEIGHT);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        editor.draw(&mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        let mut keys = keys_data.borrow_mut();

        for key in window.get_keys_pressed(KeyRepeat::No).iter() {
            match key {
                Key::Left => editor.cursor_move_left(1),
                Key::Right => editor.cursor_move_right(1),
                Key::Down => editor.cursor_move_down(1),
                Key::Up => editor.cursor_move_up(1),
                Key::Enter => editor.newline(),
                _ => (),
            }
        }

        for key in keys.iter() {
            if let Some(ch) = char::from_u32(*key) {
                if (ch.is_ascii()) {
                    editor.type_char(ch);
                }
            }
        }

        keys.clear();
    }
}

type KeyVec = Rc<RefCell<Vec<u32>>>;

struct Input {
    keys: KeyVec,
}

impl Input {
    fn new(data: &KeyVec) -> Input {
        Input { keys: data.clone() }
    }
}

impl minifb::InputCallback for Input {
    fn add_char(&mut self, uni_char: u32) {
        self.keys.borrow_mut().push(uni_char);
    }
}
