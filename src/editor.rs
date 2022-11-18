use std::cmp::min;
use std::fs;
use std::io;
use std::path::Path;

use crate::common::{GridPosition, Rgb};

const CHAR_WIDTH: usize = 9;
const CHAR_HEIGHT: usize = 14;

struct LetterSprite {
    pub pixels: [Rgb; CHAR_WIDTH * CHAR_HEIGHT],
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
            let mut p: [mem::MaybeUninit<Rgb>; CHAR_HEIGHT * CHAR_WIDTH] =
                unsafe { mem::MaybeUninit::uninit().assume_init() };
            for y in 0..height {
                for x in 0..width {
                    let r = data[y * width * 3 + x * 3];
                    let g = data[y * width * 3 + x * 3 + 1];
                    let b = data[y * width * 3 + x * 3 + 2];
                    p[y * width + x] = mem::MaybeUninit::new(Rgb::new(
                        r.parse::<u8>().unwrap(),
                        g.parse::<u8>().unwrap(),
                        b.parse::<u8>().unwrap(),
                    ));
                }
            }
            let pixels = unsafe { mem::transmute::<_, [Rgb; CHAR_HEIGHT * CHAR_WIDTH]>(p) };
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
        const FILE_EXTENSION: &str = ".ppt";
        const FONT_FOLDER: &str = "font/";
        for (_i, letter) in ASCII.chars().enumerate() {
            let file_name: String = if letter == '/' {
                "slash".to_string()
            } else {
                letter.to_string()
            };
            let file_path = format!("{FONT_FOLDER}{file_name}{FILE_EXTENSION}");
            letters.push(LetterSprite::new(file_path).unwrap());
        }
        Self { letters }
    }

    pub fn letter(&self, ch: char) -> &LetterSprite {
        &self.letters[ch as usize - 33]
    }

    pub fn index(&self, ch: char) -> usize {
        ch as usize - 33
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EditorMode {
    normal,
    insert,
}

pub struct Editor {
    cursor: GridPosition,
    viewport_position: GridPosition,
    viewport_size: (usize, usize),
    font: Font,
    screen_width: usize,
    text: Vec<String>,
    pub mode: EditorMode,
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
            mode: EditorMode::normal,
        }
    }

    pub fn type_char(&mut self, ch: char) {
        if (self.mode != EditorMode::insert) {
            return;
        }
        let position = self.cursor;
        while self.text.len() <= position.y as usize {
            self.text.push(String::new());
        }
        while self.text[position.y as usize].len() < position.x as usize {
            self.text[position.y as usize].push(' ');
        }
        self.text[position.y as usize].insert(position.x as usize, ch);
        self.cursor_move_right(1);
    }

    pub fn normal_mode(&mut self) {
        self.mode = EditorMode::normal;
    }

    pub fn insert_mode(&mut self) {
        self.mode = EditorMode::insert;
    }

    pub fn backspace(&mut self) {
        if self.cursor.x >= 1 {
            self.cursor_move_left(1);
            self.delete();
        }
    }

    pub fn delete(&mut self) {
        let position = self.cursor;
        if self.text.len() <= position.y as usize {
            return;
        }
        if self.text[position.y as usize].len() <= position.x as usize {
            return;
        }
        self.text[position.y as usize].remove(position.x as usize);
    }

    pub fn newline(&mut self) {
        self.cursor = GridPosition::new(0, self.cursor.y.saturating_add(1));
    }

    pub fn cursor_move_up(&mut self, amount: u32) {
        self.cursor.y = self.cursor.y.saturating_sub(amount);
        if self.cursor.y < self.viewport_position.y + 3 && self.viewport_position.y > 0 {
            self.viewport_position.y -= 1;
        }
    }

    pub fn cursor_move_down(&mut self, amount: u32) {
        self.cursor.y = self.cursor.y.saturating_add(amount);
        if self.cursor.y > self.viewport_position.y + self.viewport_size.1 as u32 - 3 {
            self.viewport_position.y += 1;
        }
    }

    pub fn cursor_move_left(&mut self, amount: u32) {
        self.cursor.x = self.cursor.x.saturating_sub(amount);
    }

    pub fn cursor_move_right(&mut self, amount: u32) {
        self.cursor.x = self.cursor.x.saturating_add(amount);
    }

    fn viewport_end(&self) -> GridPosition {
        let viewport_end_x = self.viewport_position.x as usize + self.viewport_size.0;
        let viewport_end_y = self.viewport_position.y as usize + self.viewport_size.1;

        GridPosition::new(viewport_end_x as u32, viewport_end_y as u32)
    }

    fn occupied_grid_cells(&self) -> Vec<Vec<(usize, usize)>> {
        let mut ascii = vec![Vec::new(); ASCII.len()];

        for y in self.viewport_position.y..min(self.viewport_end().y, self.text.len() as u32) {
            for x in self.viewport_position.x
                ..min(self.viewport_end().x, self.text[y as usize].len() as u32)
            {
                let ch = self.text[y as usize].as_bytes()[x as usize] as char;
                if ch.is_control() || ch.is_whitespace() {
                    continue;
                }
                ascii[self.font.index(ch)].push((
                    (x - self.viewport_position.x) as usize * CHAR_WIDTH,
                    (y - self.viewport_position.y) as usize * CHAR_HEIGHT,
                ));
            }
        }
        ascii
    }

    pub fn draw_char(&mut self, ch: char, pos: GridPosition, screen: &mut [u32]) {
        if !pos.is_inside(&self.viewport_position, &self.viewport_end()) {
            return;
        }
        let sprite = self.font.letter(ch).pixels;
        for y in 0..CHAR_HEIGHT {
            for x in 0..CHAR_WIDTH {
                let pixel = u32::from(sprite[(y * CHAR_WIDTH) + x]);
                if pixel == 0 {
                    continue;
                };
                screen[((pos.y - self.viewport_position.y) as usize * CHAR_HEIGHT + y)
                    * self.screen_width
                    + ((pos.x - self.viewport_position.x) as usize * CHAR_WIDTH + x)] = pixel;
            }
        }
    }

    pub fn set_text(&mut self, text: String) {
        for line in text.lines() {
            self.text.push(line.to_string());
        }
    }

    pub fn draw(&mut self, screen: &mut [u32]) {
        let occupied = self.occupied_grid_cells();
        for (i, ch) in ASCII.chars().enumerate() {
            let sprite = self.font.letter(ch).pixels;
            let positions = &occupied[i];
            for y in 0..CHAR_HEIGHT {
                for x in 0..CHAR_WIDTH {
                    let pixel = u32::from(sprite[(y * CHAR_WIDTH) + x]);
                    for (screen_x, screen_y) in positions {
                        screen[((screen_y + y) * self.screen_width) + screen_x + x] = pixel;
                    }
                }
            }
        }
        self.draw_char('_', self.cursor, screen);
    }
}
