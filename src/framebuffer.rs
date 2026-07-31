use raylib::prelude::*;

pub struct Framebuffer {
    width: u32,
    height: u32,
    color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(
            width as i32,
            height as i32,
            background_color,
        );

        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        // limpien su buffer de colores
        self.color_buffer = Image::gen_image_color(
            self.width as i32,
            self.height as i32,
            self.background_color,
        );
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        // pongan un pixel en la pantalla, asegúrense de que no se pueda salir del Buffer
        if x < self.width && y < self.height {
            self.color_buffer
                .draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        // setten el color de fondo
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        // setten el color
        self.current_color = color;
    }

    pub fn render_to_file(&self, file_path: &str) {
        // guarden su framebuffer a un archivo usando export
        self.color_buffer.export_image(file_path);
    }

    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
    ) {
        // we get the "new" data from the new buffer into texture
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {

            // the window currently has the "old" data (previous frame)
            let mut renderer = window.begin_drawing(raylib_thread);

            // we move the "new" data to the window (current frame)
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    }
}
