use minifb::{Key, Window, WindowOptions};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::rc::Rc;

mod common;
use common::{Size, Vec2};
mod sprite;
use sprite::{Sprite, RGB};
mod ppt;
use ppt::load_sprite;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window =
        Window::new("game", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{:?}", e);
        });

    let keys_data = KeyVec::new(RefCell::new(Vec::new()));

    let input = Box::new(Input::new(&keys_data));

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.set_input_callback(input);

    let mut font: HashMap<String, Sprite> = HashMap::new();
    for entry in read_dir("font").unwrap() {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().into_string().unwrap();
            if let Some((character, _file_extension)) = file_name.rsplit_once('.') {
                let sprite = load_sprite(entry.path()).unwrap();
                font.insert(character.to_string(), sprite);
            }
        }
    }
    font.insert(
        " ".to_string(),
        Sprite::new(
            Vec2::new(0.0, 0.0),
            Size::new(9, 14),
            vec![RGB::new(0, 0, 0); 9 * 14],
        ),
    );
    font.insert(
        "\t".to_string(),
        Sprite::new(
            Vec2::new(0.0, 0.0),
            Size::new(9, 14),
            vec![RGB::new(0, 0, 0); 9 * 14],
        ),
    );

    let ascii = "!\"#$%&\'()*+,-./\n0123456789:;<=>?\n@ABCDEFGHIJKLMNO\nPQRSTUVWXYZ[\\]^_\n`abcdefghijklmno\npqrstuvwxyz{|}~";
    let text = read_to_string("src/main.rs").unwrap();
    let char_width = 9;
    let char_height = 14;
    let mut sprites: Vec<Sprite> = Vec::with_capacity(16 * 6);
    for (y, line) in ascii.lines().enumerate() {
        for (x, char) in line.chars().enumerate() {
            let pos = Vec2::new((x * char_width) as f32, (y * char_height) as f32);
            let char_sprite = if char == '/' {
                font.get(&"slash".to_string()).unwrap()
            } else {
                font.get(&char.to_string()).unwrap()
            };
            sprites.push(Sprite::new(
                pos,
                Size::new(
                    char_width.try_into().unwrap(),
                    char_height.try_into().unwrap(),
                ),
                char_sprite.pixels.clone(),
            ));
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for sprite in &sprites {
            sprite.draw(&mut buffer, WIDTH as u32, HEIGHT as u32);
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        let mut keys = keys_data.borrow_mut();

        for t in keys.iter() {
            println!("Code point: {},   Character: {:?}", *t, char::from_u32(*t));
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
