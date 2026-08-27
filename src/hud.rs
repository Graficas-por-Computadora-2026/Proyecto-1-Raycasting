use raylib::prelude::*;

use crate::framebuffer::Framebuffer;

pub fn render_hud(
    framebuffer: &mut Framebuffer,
    health: i32,
    ammo: i32,
    max_ammo: i32,
    enemies_alive: usize,
    total_enemies: usize,
) {
    render_status_bar(framebuffer, 18, health, 100, Color::RED);
    render_status_bar(framebuffer, 38, ammo, max_ammo, Color::YELLOW);
    render_status_bar(
        framebuffer,
        58,
        enemies_alive as i32,
        total_enemies.max(1) as i32,
        Color::SKYBLUE,
    );
}

fn render_status_bar(
    framebuffer: &mut Framebuffer,
    y: u32,
    value: i32,
    maximum: i32,
    color: Color,
) {
    const WIDTH: u32 = 160;
    const HEIGHT: u32 = 14;
    const X: u32 = 18;
    let filled_width = (value.clamp(0, maximum) as u32 * WIDTH) / maximum as u32;

    framebuffer.set_current_color(Color::DARKGRAY);
    for row in y..y + HEIGHT {
        for x in X..X + WIDTH {
            framebuffer.set_pixel(x, row);
        }
    }

    framebuffer.set_current_color(color);
    for row in y..y + HEIGHT {
        for x in X..X + filled_width {
            framebuffer.set_pixel(x, row);
        }
    }
}
