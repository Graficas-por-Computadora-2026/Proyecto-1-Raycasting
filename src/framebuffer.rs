use raylib::prelude::*;

pub struct Framebuffer {
    width: u32,
    height: u32,
    color_buffer: Image,
    display_texture: Option<Texture2D>,
    yamcha_texture: Option<Texture2D>,
    sphere_texture: Option<Texture2D>,
    porunga_texture: Option<Texture2D>,
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
            yamcha_texture: None,
            sphere_texture: None,
            porunga_texture: None,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        // limpien su buffer de colores
        self.color_buffer.clear_background(self.background_color);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        // pongan un pixel en la pantalla, asegúrense de que no se pueda salir del Buffer
        if x < self.width && y < self.height {
            self.color_buffer
                .draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_current_color(&mut self, color: Color) {
        // setten el color
        self.current_color = color;
    }

    pub fn swap_buffers(
        &mut self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        show_welcome: bool,
        selected_level: usize,
        show_success: bool,
        show_defeat: bool,
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

        if self.yamcha_texture.is_none() {
            self.yamcha_texture = window.load_texture(raylib_thread, "assets/yamcha.png").ok();
        }
        if self.sphere_texture.is_none() {
            self.sphere_texture = window.load_texture(raylib_thread, "assets/sphere1.png").ok();
        }
        if self.porunga_texture.is_none() {
            self.porunga_texture = window.load_texture(raylib_thread, "assets/porunga.png").ok();
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

            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.clear_background(Color::BLACK);

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
                draw_dragon_ball_background(&mut renderer, screen_width, screen_height, Color::new(16, 36, 90, 255));
                let center_x = screen_width / 2;
                renderer.draw_text("DROOM BALL SUPER", center_x - 240, screen_height / 3 - 20, 52, Color::ORANGE);
                renderer.draw_text("Selecciona un nivel", center_x - 115, screen_height / 2 - 20, 24, Color::WHITE);
                let level_one_color = if selected_level == 0 { Color::YELLOW } else { Color::LIGHTGRAY };
                let level_two_color = if selected_level == 1 { Color::YELLOW } else { Color::LIGHTGRAY };
                let level_three_color = if selected_level == 2 { Color::YELLOW } else { Color::LIGHTGRAY };
                renderer.draw_text("Nivel 1", center_x - 50, screen_height / 2 + 25, 22, level_one_color);
                renderer.draw_text("Nivel 2", center_x - 50, screen_height / 2 + 55, 22, level_two_color);
                renderer.draw_text("Nivel 3", center_x - 50, screen_height / 2 + 85, 22, level_three_color);
                renderer.draw_text("W/S y ENTER para comenzar", center_x - 145, screen_height / 2 + 145, 20, Color::WHITE);
                renderer.draw_text("W/S: mover | A/D o mouse: girar | ESPACIO: disparar | E: usar | M: vista 2D/3D", center_x - 335, screen_height / 2 + 180, 16, Color::LIGHTGRAY);
            } else if show_success {
                draw_dragon_ball_background(&mut renderer, screen_width, screen_height, Color::new(12, 66, 42, 255));
                if selected_level == 2 {
                    if let Some(texture) = self.porunga_texture.as_ref() {
                        draw_centered_texture(&mut renderer, texture, screen_width, screen_height, 0.62);
                    }
                    renderer.draw_text("PORUNGA HA SIDO INVOCADO", screen_width / 2 - 280, 72, 42, Color::GREEN);
                    renderer.draw_text("HAS REUNIDO LAS ESFERAS DEL DRAGON", screen_width / 2 - 265, screen_height - 112, 26, Color::YELLOW);
                } else {
                    if let Some(texture) = self.sphere_texture.as_ref() {
                        draw_centered_texture(&mut renderer, texture, screen_width, screen_height, 0.36);
                    }
                    renderer.draw_text("HAS RECOLECTADO UNA ESFERA", screen_width / 2 - 285, 82, 42, Color::YELLOW);
                    renderer.draw_text("DEL DRAGON", screen_width / 2 - 100, 128, 32, Color::ORANGE);
                }
                renderer.draw_text("ENTER: elegir nivel", screen_width / 2 - 125, screen_height - 60, 22, Color::WHITE);
            } else if show_defeat {
                draw_dragon_ball_background(&mut renderer, screen_width, screen_height, Color::new(72, 12, 10, 255));
                if let Some(texture) = self.yamcha_texture.as_ref() {
                    draw_centered_texture(&mut renderer, texture, screen_width, screen_height, 0.60);
                }
                renderer.draw_text("HAS SIDO ELIMINADO", screen_width / 2 - 245, 62, 48, Color::YELLOW);
                renderer.draw_text("ENTER: reiniciar nivel", screen_width / 2 - 145, screen_height - 88, 24, Color::WHITE);
                renderer.draw_text("L: elegir nivel", screen_width / 2 - 90, screen_height - 53, 20, Color::LIGHTGRAY);
            }

            renderer.draw_fps(screen_width - 90, screen_height - 30);
        }
    }
}

fn draw_dragon_ball_background<D: RaylibDraw>(
    renderer: &mut D,
    screen_width: i32,
    screen_height: i32,
    color: Color,
) {
    renderer.draw_rectangle(0, 0, screen_width, screen_height, color);
    renderer.draw_rectangle(0, 0, screen_width, 14, Color::ORANGE);
    renderer.draw_rectangle(0, screen_height - 14, screen_width, 14, Color::ORANGE);
    for index in 0..7 {
        let x = 30 + index * 42;
        renderer.draw_circle(x, 34, 14.0, Color::ORANGE);
        renderer.draw_circle(x, 34, 3.0, Color::RED);
    }
}

fn draw_centered_texture<D: RaylibDraw>(
    renderer: &mut D,
    texture: &Texture2D,
    screen_width: i32,
    screen_height: i32,
    max_size: f32,
) {
    let source_width = texture.width() as f32;
    let source_height = texture.height() as f32;
    let scale = (screen_width as f32 * max_size / source_width)
        .min(screen_height as f32 * max_size / source_height);
    let width = source_width * scale;
    let height = source_height * scale;
    renderer.draw_texture_pro(
        texture,
        Rectangle::new(0.0, 0.0, source_width, source_height),
        Rectangle::new(
            (screen_width as f32 - width) / 2.0,
            (screen_height as f32 - height) / 2.0,
            width,
            height,
        ),
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}
