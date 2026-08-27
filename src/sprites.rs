use raylib::prelude::*;

use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::textures::TextureManager;

#[derive(Clone, Copy)]
pub enum EnemyKind {
    Grunt,
    Brute,
}

pub struct Sprite {
    pub pos: Vector2,
    pub texture: char,
    pub size: f32,
    pub active: bool,
    pub health: i32,
    pub attack_cooldown: f32,
    pub kind: EnemyKind,
}

pub fn shoot_sprite(player: &Player, sprite: &mut Sprite, wall_distance: f32) -> bool {
    if !sprite.active {
        return false;
    }

    let dx = sprite.pos.x - player.pos.x;
    let dy = sprite.pos.y - player.pos.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let angle_difference = normalize_angle(dy.atan2(dx) - player.a);
    let angular_radius = (sprite.size / 2.0 / distance.max(1.0)).atan();

    if angle_difference.abs() <= angular_radius && distance < wall_distance {
        sprite.health -= 1;
        if sprite.health <= 0 {
            sprite.active = false;
        }
        return true;
    }

    false
}

pub fn render_sprite(
    framebuffer: &mut Framebuffer,
    player: &Player,
    sprite: &Sprite,
    textures: &TextureManager,
    z_buffer: &[f32],
    _time: f32,
    tint: Color,
) {
    if !sprite.active {
        return;
    }

    let dx = sprite.pos.x - player.pos.x;
    let dy = sprite.pos.y - player.pos.y;
    let angle_to_sprite = dy.atan2(dx);
    let angle_difference = normalize_angle(angle_to_sprite - player.a);

    let distance = (dx * dx + dy * dy).sqrt();
    let angular_radius = (sprite.size / 2.0 / distance.max(1.0)).atan();

    if angle_difference - angular_radius > player.fov / 2.0
        || angle_difference + angular_radius < -player.fov / 2.0
    {
        return;
    }

    let corrected_distance = (distance * angle_difference.cos()).max(1.0);
    let projection_plane = framebuffer.width() as f32 / 2.0 / (player.fov / 2.0).tan();
    let sprite_height = sprite.size / corrected_distance * projection_plane;

    let Some((texture_width, texture_height)) = textures.dimensions(sprite.texture) else {
        return;
    };

    let sprite_width = sprite_height * texture_width as f32 / texture_height as f32;
    let center_x = framebuffer.width() as f32 / 2.0
        + angle_difference / (player.fov / 2.0) * framebuffer.width() as f32 / 2.0;
    // Move the base into the projected floor; this keeps sprites and pickups
    // visually grounded instead of hanging at the horizon.
    let bottom = framebuffer.height() as f32 / 2.0 + sprite_height * 0.35;
    let top = bottom - sprite_height;
    let left = center_x - sprite_width / 2.0;
    let right = center_x + sprite_width / 2.0;

    let start_x = left.clamp(0.0, framebuffer.width() as f32) as u32;
    let end_x = right.clamp(0.0, framebuffer.width() as f32) as u32;
    let start_y = top.clamp(0.0, framebuffer.height() as f32) as u32;
    let end_y = bottom.clamp(0.0, framebuffer.height() as f32) as u32;

    for screen_x in start_x..end_x {
        if corrected_distance >= z_buffer[screen_x as usize] {
            continue;
        }

        let tx = ((screen_x as f32 - left) / sprite_width * texture_width as f32) as u32;

        for screen_y in start_y..end_y {
            let ty = ((screen_y as f32 - top) / sprite_height * texture_height as f32) as u32;
            let color = textures.get_pixel_color(sprite.texture, tx, ty);

            if color.a > 0 {
                framebuffer.set_current_color(Color::new(
                    (color.r as u16 * tint.r as u16 / 255) as u8,
                    (color.g as u16 * tint.g as u16 / 255) as u8,
                    (color.b as u16 * tint.b as u16 / 255) as u8,
                    color.a,
                ));
                framebuffer.set_pixel(screen_x, screen_y);
            }
        }
    }
}

fn normalize_angle(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI
}
