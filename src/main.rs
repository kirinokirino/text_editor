use minifb::{Key, KeyRepeat, Window, WindowOptions};

use std::cell::RefCell;
use std::env::current_dir;
use std::fs;
use std::rc::Rc;

mod cli;
mod common;
mod editor;

use cli::Arguments;
use editor::{Editor, EditorMode};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let args = Arguments::new();
    let mut path = current_dir().unwrap();
    let mut starting_text = String::new();
    if args.unnamed.len() == 1 {
        path.push(args.unnamed[0].clone());
        if let Ok(contents) = fs::read_to_string(path) {
            starting_text = contents;
        }
    }
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
    editor.set_text(starting_text);
    'running: while window.is_open() {
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
                Key::I => editor.insert_mode(),
                Key::C => {
                    if window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl) {
                        break 'running;
                    }
                }
                Key::Apostrophe => {
                    if editor.mode == EditorMode::Normal {
                        break 'running;
                    }
                }
                Key::Period => {
                    if editor.mode == EditorMode::Normal {
                        editor.cursor_move_up(1)
                    }
                }
                Key::Escape => editor.normal_mode(),
                Key::E => {
                    if editor.mode == EditorMode::Normal {
                        editor.cursor_move_down(1)
                    }
                }
                Key::O => {
                    if editor.mode == EditorMode::Normal {
                        editor.cursor_move_left(1)
                    }
                }
                Key::U => {
                    if editor.mode == EditorMode::Normal {
                        editor.cursor_move_right(1)
                    }
                }
                Key::Enter => editor.newline(),
                Key::Delete => editor.delete(),
                Key::Backspace => editor.backspace(),
                key => {
                    dbg!(key);
                }
            }
        }

        for key in keys.iter() {
            if let Some(ch) = char::from_u32(*key) {
                if ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch == ' ' {
                    editor.type_char(ch);
                } else {
                    dbg!(ch);
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
