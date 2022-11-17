use std::fs;
use std::io;
use std::path::Path;

use crate::common::{GridPosition, RGB};

const CHAR_WIDTH: usize = 9;
const CHAR_HEIGHT: usize = 14;

struct LetterSprite {
    pub pixels: [RGB; CHAR_WIDTH * CHAR_HEIGHT],
}

impl LetterSprite {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let data = fs::read_to_string(path)?;
        if let Some((header, data)) = data.split_once('\n') {
            let data: Vec<&str> = data.split_ascii_whitespace().collect();
            let mut split = header.split_whitespace();
            let format = split
                .next()
                .expect("Error while parsing ppt sprite: no format!");
            assert!(format == "P3", "Only support P3 ppt version");
            let width = split
                .next()
                .expect("Error while parsing ppt sprite: no sprite width!")
                .parse::<usize>()
                .expect("couldn't parse sprite width");
            let height = split
                .next()
                .expect("Error while parsing ppt sprite: no sprite height!")
                .parse::<usize>()
                .expect("couldn't parse sprite height");
            let colors = split
                .next()
                .expect("Error while parsing ppt sprite: no max colors!");
            assert!(colors == "255", "Not yet support anything but 255 colors");
            assert!(split.next().is_none(), "Unknown additional header fields");

            use std::mem;
            assert!(width == CHAR_WIDTH);
            assert!(height == CHAR_HEIGHT);
            let mut p: [mem::MaybeUninit<RGB>; CHAR_HEIGHT * CHAR_WIDTH] =
                unsafe { mem::MaybeUninit::uninit().assume_init() };
            for y in 0..height {
                for x in 0..width {
                    let r = data[y * width * 3 + x * 3];
                    let g = data[y * width * 3 + x * 3 + 1];
                    let b = data[y * width * 3 + x * 3 + 2];
                    p[y * width + x] = mem::MaybeUninit::new(RGB::new(
                        r.parse::<u8>().unwrap(),
                        g.parse::<u8>().unwrap(),
                        b.parse::<u8>().unwrap(),
                    ));
                }
            }
            let pixels = unsafe { mem::transmute::<_, [RGB; CHAR_HEIGHT * CHAR_WIDTH]>(p) };
            return Ok(Self { pixels });
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Couldn't split file on newline!",
        ))
    }
}

const ASCII: &str = "!\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
struct Font {
    letters: Vec<LetterSprite>,
}

impl Font {
    pub fn new() -> Self {
        let mut letters = Vec::with_capacity(ASCII.len());
        const file_extension: &str = ".ppt";
        const font_folder: &str = "font/";
        for (i, letter) in ASCII.chars().enumerate() {
            let file_name: String = if letter == '/' {
                "slash".to_string()
            } else {
                letter.to_string()
            };
            let file_path = format!("{font_folder}{file_name}{file_extension}");
            letters.push(LetterSprite::new(file_path).unwrap());
        }
        Self { letters }
    }

    pub fn letter(&self, ch: char) -> &LetterSprite {
        &self.letters[ch as usize - 33]
    }
}

pub struct Editor {
    cursor: GridPosition,
    viewport_position: GridPosition,
    viewport_size: (usize, usize),
    font: Font,
    screen_width: usize,
    text: Vec<String>,
}

impl Editor {
    pub fn new(screen_width: usize, screen_height: usize) -> Self {
        Self {
            cursor: GridPosition::new(0, 0),
            viewport_position: GridPosition::new(0, 0),
            viewport_size: (screen_width / CHAR_WIDTH, screen_height / CHAR_HEIGHT),
            font: Font::new(),
            screen_width,
            text: Vec::new(),
        }
    }

    

    pub fn cursor_move_up(&mut self, amount: u32) {
        self.cursor.y = self.cursor.y.saturating_sub(amount);
    }

    pub fn cursor_move_down(&mut self, amount: u32) {
        self.cursor.y = self.cursor.y.saturating_add(amount);
    }

    pub fn cursor_move_left(&mut self, amount: u32) {
        self.cursor.x = self.cursor.x.saturating_sub(amount);
    }

    pub fn cursor_move_right(&mut self, amount: u32) {
        self.cursor.x = self.cursor.x.saturating_add(amount);
    }

    pub fn check_bounds(position: &GridPosition, from: &GridPosition, to: &GridPosition) -> bool {
        position.x >= from.x && position.x < to.x && position.y >= from.y && position.y < to.y
    }

    pub fn draw(&mut self, screen: &mut [u32]) {
        let viewport_end_x = self.viewport_position.x as usize + self.viewport_size.0;
        let viewport_end_y = self.viewport_position.y as usize + self.viewport_size.1;

        let viewport_end = GridPosition::new(viewport_end_x as u32, viewport_end_y as u32);

        let sprite = self.font.letter('_').pixels;
        let positions = vec![self.cursor];

        //filter positions self.check_bounds(position, self.viewport_position, viewport_end)
        let screen_positions: Vec<(usize, usize)> = positions
            .into_iter()
            .map(|pos| (pos.x as usize * CHAR_WIDTH, pos.y as usize * CHAR_HEIGHT))
            .collect();
        for y in 0..CHAR_HEIGHT {
            for x in 0..CHAR_WIDTH {
                let pixel = u32::from(sprite[(y * CHAR_WIDTH) + x]);
                for (screen_x, screen_y) in &screen_positions {
                    screen[((screen_y + y) * self.screen_width as usize) + screen_x + x] = pixel;
                }
            }
        }
    }
}
