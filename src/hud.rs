use raylib::prelude::*;

use crate::framebuffer::Framebuffer;

pub fn render_hud(
    framebuffer: &mut Framebuffer,
    health: i32,
    ammo: i32,
    max_ammo: i32,
    enemies_alive: usize,
) {
    render_status_bar(framebuffer, 14, "VIDA", health, 100, Color::RED);
    render_status_bar(
        framebuffer,
        48,
        "MUNICION",
        ammo,
        max_ammo,
        Color::YELLOW,
    );

    draw_text(framebuffer, "ENEMIGOS:", 18, 84, 2, Color::WHITE);
    draw_text(
        framebuffer,
        &enemies_alive.to_string(),
        92,
        84,
        2,
        Color::SKYBLUE,
    );
}

fn render_status_bar(
    framebuffer: &mut Framebuffer,
    y: u32,
    label: &str,
    value: i32,
    maximum: i32,
    color: Color,
) {
    const X: u32 = 18;
    const WIDTH: u32 = 170;
    const HEIGHT: u32 = 12;
    let maximum = maximum.max(1);
    let filled_width = (value.clamp(0, maximum) as u32 * WIDTH) / maximum as u32;
    let value_text = format!("{}/{}", value.max(0), maximum);

    draw_text(framebuffer, label, X, y, 2, Color::WHITE);
    draw_text(framebuffer, &value_text, X + 112, y, 2, color);
    draw_rectangle(framebuffer, X, y + 12, WIDTH, HEIGHT, Color::DARKGRAY);
    draw_rectangle(framebuffer, X, y + 12, filled_width, HEIGHT, color);
    draw_outline(framebuffer, X, y + 12, WIDTH, HEIGHT, Color::WHITE);
}

fn draw_rectangle(framebuffer: &mut Framebuffer, x: u32, y: u32, width: u32, height: u32, color: Color) {
    framebuffer.set_current_color(color);
    for row in y..y + height {
        for column in x..x + width {
            framebuffer.set_pixel(column, row);
        }
    }
}

fn draw_outline(framebuffer: &mut Framebuffer, x: u32, y: u32, width: u32, height: u32, color: Color) {
    framebuffer.set_current_color(color);
    for column in x..x + width {
        framebuffer.set_pixel(column, y);
        framebuffer.set_pixel(column, y + height - 1);
    }
    for row in y..y + height {
        framebuffer.set_pixel(x, row);
        framebuffer.set_pixel(x + width - 1, row);
    }
}

fn draw_text(framebuffer: &mut Framebuffer, text: &str, x: u32, y: u32, scale: u32, color: Color) {
    for (index, character) in text.chars().enumerate() {
        let offset_x = x + index as u32 * 4 * scale;
        draw_glyph(framebuffer, character, offset_x, y, scale, color);
    }
}

fn draw_glyph(framebuffer: &mut Framebuffer, character: char, x: u32, y: u32, scale: u32, color: Color) {
    let glyph = match character {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'G' => [0b111, 0b100, 0b101, 0b101, 0b111],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
        'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        _ => [0; 5],
    };

    framebuffer.set_current_color(color);
    for (row, bits) in glyph.into_iter().enumerate() {
        for column in 0..3 {
            if bits & (1 << (2 - column)) != 0 {
                draw_rectangle(
                    framebuffer,
                    x + column * scale,
                    y + row as u32 * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}
