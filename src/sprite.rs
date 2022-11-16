use crate::common::{max, min, Size, Vec2};

use std::convert::Into;

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Into<u32> for RGB {
    fn into(self) -> u32 {
        0 | ((self.r as u32) << 16) | ((self.g as u32) << 8) | ((self.b as u32) << 0)
    }
}

pub struct Sprite {
    pub origin: Vec2,
    pub size: Size,
    pub pixels: Vec<RGB>,
}

impl Sprite {
    pub fn new(pos: Vec2, size: Size, pixels: Vec<RGB>) -> Self {
        Self {
            origin: pos,
            size,
            pixels,
        }
    }
    pub fn draw(&self, screen: &mut [u32], screen_width: u32, screen_height: u32) {
        if (self.origin.x < 0.0 || self.origin.y < 0.0) {
            return;
        }
        if (self.origin.x as u32 + self.size.width > screen_width
            || self.origin.y as u32 + self.size.height > screen_height)
        {
            return;
        }

        for y in 0..self.size.height as usize {
            for x in 0..self.size.width as usize {
                let screen_pos_x = x + self.origin.x as usize;
                let screen_pos_y = y + self.origin.y as usize;

                screen[(screen_pos_y * screen_width as usize) + screen_pos_x] =
                    self.pixels[(y * self.size.width as usize) + x].into();
            }
        }
    }
}
