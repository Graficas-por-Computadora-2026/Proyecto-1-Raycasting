use raylib::prelude::*;
use std::collections::HashMap;
use std::slice;

pub struct TextureManager {
    images: HashMap<char, Image>,
}

impl TextureManager {
    pub fn new(level: usize) -> Self {
        let mut images = HashMap::new();
        let sphere_path = match level {
            0 => "assets/sphere1.png",
            1 => "assets/sphere2.png",
            _ => "assets/sphere3.png",
        };

        let texture_files = vec![
            ('#', "assets/deadline.png"),
            ('w', "assets/nivel1.png"),
            ('c', "assets/sky.png"),
            ('h', "assets/hermitaño.png"),
            ('a', "assets/ki.png"),
            ('x', sphere_path),
            ('g', "assets/porunga.png"),
            ('e', "assets/frieza.png"),
            ('p', "assets/ki blast.png"),
            ('b', "assets/jiren.png"),
        ];

        for (ch, path) in texture_files {
            let image = Image::load_image(path)
                .unwrap_or_else(|_| panic!("Failed to load image {path}"));
            images.insert(ch, image);
        }

        TextureManager { images }
    }

    pub fn get_pixel_color(&self, ch: char, tx: u32, ty: u32) -> Color {
        let image = self.images.get(&ch).or_else(|| self.images.get(&'#'));

        if let Some(image) = image {
            let x = tx.min(image.width as u32 - 1) as i32;
            let y = ty.min(image.height as u32 - 1) as i32;
            get_pixel_color(image, x, y)
        } else {
            Color::WHITE
        }
    }

    pub fn dimensions(&self, ch: char) -> Option<(u32, u32)> {
        self.images
            .get(&ch)
            .or_else(|| self.images.get(&'#'))
            .map(|image| (image.width as u32, image.height as u32))
    }

    pub fn get_cell_pixel_color(
        &self,
        ch: char,
        cell_x: usize,
        cell_y: usize,
        tx: u32,
        ty: u32,
    ) -> Color {
        self.get_pixel_color(self.texture_key(ch, cell_x, cell_y), tx, ty)
    }

    pub fn cell_dimensions(
        &self,
        ch: char,
        cell_x: usize,
        cell_y: usize,
    ) -> Option<(u32, u32)> {
        self.dimensions(self.texture_key(ch, cell_x, cell_y))
    }

    fn texture_key(&self, ch: char, cell_x: usize, cell_y: usize) -> char {
        if matches!(ch, '+' | '-' | '|' | 'D' | '#') {
            if (cell_x + cell_y).is_multiple_of(2) {
                '#'
            } else {
                'w'
            }
        } else {
            ch
        }
    }
}

fn get_pixel_color(image: &Image, x: i32, y: i32) -> Color {
    let width = image.width as usize;
    let height = image.height as usize;

    if x < 0 || y < 0 || x as usize >= width || y as usize >= height {
        return Color::WHITE;
    }

    let x = x as usize;
    let y = y as usize;
    let data_len = width * height * 4;

    unsafe {
        let data = slice::from_raw_parts(image.data as *const u8, data_len);
        let idx = (y * width + x) * 4;

        if idx + 3 >= data_len {
            return Color::WHITE;
        }

        Color::new(data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
    }
}
