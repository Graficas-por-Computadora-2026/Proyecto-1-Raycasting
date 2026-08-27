use raylib::prelude::*;
use std::collections::HashMap;
use std::slice;

pub struct TextureManager {
    images: HashMap<char, Image>,
}

impl TextureManager {
    pub fn new() -> Self {
        let mut images = HashMap::new();

        // Map maze characters to texture file paths.
        let texture_files = vec![
            ('+', "assets/wall.png"),
            ('-', "assets/wall.png"),
            ('|', "assets/wall.png"),
            ('g', "assets/wall.png"),
            ('#', "assets/wall.png"), // default/fallback
            ('s', "assets/sprite.png"),
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
