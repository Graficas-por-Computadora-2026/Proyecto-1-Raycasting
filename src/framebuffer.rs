use raylib::prelude::*;

pub struct Framebuffer {
    width: u32,
    height: u32,
    color_buffer: Image,
    display_texture: Option<Texture2D>,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

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
            display_texture: None,
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
        &mut self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        show_welcome: bool,
        selected_level: usize,
        show_success: bool,
    ) {
        // La textura de presentación se crea una sola vez por tamaño de framebuffer.
        if self.display_texture.is_none() {
            let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer)
            else {
                return;
            };
            texture.set_texture_filter(raylib_thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            self.display_texture = Some(texture);
        }

        let pixel_count = (self.width * self.height * 4) as usize;
        let pixels = unsafe {
            std::slice::from_raw_parts(self.color_buffer.data() as *const u8, pixel_count)
        };

        if let Some(texture) = self.display_texture.as_mut() {
            if texture.update_texture(pixels).is_err() {
                return;
            }
            let screen_width = window.get_screen_width();
            let screen_height = window.get_screen_height();

            // the window currently has the "old" data (previous frame)
            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.clear_background(Color::BLACK);

            // we move the "new" data to the window (current frame)
            renderer.draw_texture_pro(
                &texture,
                Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
                Rectangle::new(
                    0.0,
                    0.0,
                    screen_width as f32,
                    screen_height as f32,
                ),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );

            if show_welcome {
                renderer.draw_rectangle(
                    0,
                    0,
                    screen_width,
                    screen_height,
                    Color::new(12, 18, 35, 255),
                );
                let center_x = screen_width / 2;
                renderer.draw_text("MUNDO 3D", center_x - 130, screen_height / 3 - 20, 52, Color::SKYBLUE);
                renderer.draw_text("Selecciona un nivel", center_x - 115, screen_height / 2 - 20, 24, Color::WHITE);
                let level_one_color = if selected_level == 0 { Color::SKYBLUE } else { Color::LIGHTGRAY };
                let level_two_color = if selected_level == 1 { Color::SKYBLUE } else { Color::LIGHTGRAY };
                renderer.draw_text("Nivel 1", center_x - 50, screen_height / 2 + 25, 22, level_one_color);
                renderer.draw_text("Nivel 2", center_x - 50, screen_height / 2 + 55, 22, level_two_color);
                renderer.draw_text("Flechas arriba/abajo y ENTER para comenzar", center_x - 215, screen_height / 2 + 120, 20, Color::WHITE);
                renderer.draw_text("Mouse: girar | Clic izquierdo: disparar | M: vista 2D/3D", center_x - 265, screen_height / 2 + 155, 16, Color::LIGHTGRAY);
            } else if show_success {
                renderer.draw_rectangle(
                    0,
                    0,
                    screen_width,
                    screen_height,
                    Color::new(12, 35, 20, 255),
                );
                renderer.draw_text("NIVEL COMPLETADO", screen_width / 2 - 220, screen_height / 2 - 50, 48, Color::GREEN);
                renderer.draw_text("Presiona ENTER para elegir otro nivel", screen_width / 2 - 195, screen_height / 2 + 50, 24, Color::WHITE);
            }

            renderer.draw_fps(screen_width - 90, screen_height - 30);
        }
    }
}
