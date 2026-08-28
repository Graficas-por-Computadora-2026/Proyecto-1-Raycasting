use raylib::prelude::*;

use crate::caster::cast_ray;
use crate::combat::{Pickup, PickupKind, Projectile};
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::sprites::{render_sprite, EnemyKind, Sprite};
use crate::textures::TextureManager;
use crate::Maze;

const BASE_ASPECT_RATIO: f32 = 800.0 / 600.0;

pub fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    textures: &TextureManager,
    sprites: &[Sprite],
    pickups: &[Pickup],
    projectiles: &[Projectile],
    frame_index: usize,
    shot_flash: bool,
) {
    let num_rays = framebuffer.width();
    let mut z_buffer = vec![0.0; num_rays as usize];

    let hw = framebuffer.width() as f32 / 2.0;   // precalculated half width
    let hh = framebuffer.height() as f32 / 2.0;  // precalculated half height
    let vertical_fov = 2.0 * ((player.fov / 2.0).tan() / BASE_ASPECT_RATIO).atan();
    let render_fov = 2.0
        * ((framebuffer.width() as f32 / framebuffer.height() as f32) * (vertical_fov / 2.0).tan())
            .atan();
    let render_player = Player {
        pos: player.pos,
        a: player.a,
        fov: render_fov,
    };

    let horizon = hh as u32;

    if let Some((sky_width, sky_height)) = textures.dimensions('c') {
        for y in 0..horizon {
            let ty = y * sky_height / horizon.max(1);
            for x in 0..framebuffer.width() {
                let tx = x * sky_width / framebuffer.width().max(1);
                framebuffer.set_current_color(textures.get_pixel_color('c', tx, ty));
                framebuffer.set_pixel(x, y);
            }
        }
    } else {
        framebuffer.set_current_color(Color::SKYBLUE);
        for y in 0..horizon {
            for x in 0..framebuffer.width() {
                framebuffer.set_pixel(x, y);
            }
        }
    }

    framebuffer.set_current_color(Color::new(0, 0, 0, 255));
    for y in horizon..framebuffer.height() {
        for x in 0..framebuffer.width() {
            framebuffer.set_pixel(x, y);
        }
    }

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
        let a = render_player.a - (render_player.fov / 2.0) + (render_player.fov * current_ray);

        let intersect = cast_ray(
            framebuffer,
            &maze,
            &render_player,
            a,
            block_size,
            false,
        );

        // Calculate the height of the stake
        let distance_to_wall = intersect.distance * (render_player.a - a).cos(); // fish-eye correction
        let distance_to_projection_plane = hw / (render_player.fov / 2.0).tan(); // distance from the "camera"

        // this ratio doesn't really matter as long as it is a function of distance
        let distance_to_wall = distance_to_wall.max(1.0);
        z_buffer[i as usize] = distance_to_wall;
        let stake_height =
            (block_size as f32 / distance_to_wall) * distance_to_projection_plane;

        // Calculate the position to draw the stake
        let projected_top = hh - (stake_height / 2.0);
        let projected_bottom = hh + (stake_height / 2.0);
        let stake_top = projected_top.max(0.0) as u32;
        let stake_bottom = projected_bottom
            .min(framebuffer.height() as f32) as u32;

        let texture_dimensions = if intersect.impact == 'g' {
            textures.dimensions('g')
        } else {
            textures.wall_dimensions_for_cell(intersect.cell_x, intersect.cell_y)
        };

        if let Some((texture_width, texture_height)) = texture_dimensions {
            let hit_offset = if intersect.hit_vertical {
                intersect.hit_y.rem_euclid(block_size as f32)
            } else {
                intersect.hit_x.rem_euclid(block_size as f32)
            };
            let tx = (hit_offset / block_size as f32 * texture_width as f32) as u32;
            // Draw the stake directly in the framebuffer using texture coordinates.
            for y in stake_top..stake_bottom {
                let ty = ((y as f32 - projected_top) / stake_height
                    * texture_height as f32) as u32;
                let color = if intersect.impact == 'g' {
                    textures.get_pixel_color('g', tx, ty)
                } else {
                    textures.get_wall_pixel_color_for_cell(
                        intersect.cell_x,
                        intersect.cell_y,
                        tx,
                        ty,
                    )
                };
                framebuffer.set_current_color(color);
                framebuffer.set_pixel(i, y);
            }
        }
    }

    for sprite in sprites {
        render_sprite(
            framebuffer,
            &render_player,
            sprite,
            textures,
            &z_buffer,
            frame_index as f32,
            Color::WHITE,
        );
    }

    for pickup in pickups {
        if !pickup.active {
            continue;
        }

        let pickup_sprite = Sprite {
            pos: pickup.pos,
            texture: match pickup.kind {
                PickupKind::Health => 'h',
                PickupKind::Ammo => 'a',
                PickupKind::Switch => 'x',
            },
            size: block_size as f32 * 0.6,
            active: true,
            health: 0,
            attack_cooldown: 0.0,
            kind: EnemyKind::Grunt,
        };
        render_sprite(
            framebuffer,
            &render_player,
            &pickup_sprite,
            textures,
            &z_buffer,
            frame_index as f32,
            Color::WHITE,
        );
    }

    for projectile in projectiles {
        if !projectile.active {
            continue;
        }

        let projectile_sprite = Sprite {
            pos: projectile.pos,
            texture: 'p',
            size: block_size as f32 * 0.2,
            active: true,
            health: 0,
            attack_cooldown: 0.0,
            kind: EnemyKind::Grunt,
        };
        render_sprite(
            framebuffer,
            &render_player,
            &projectile_sprite,
            textures,
            &z_buffer,
            frame_index as f32,
            Color::WHITE,
        );
    }

    let center_x = framebuffer.width() / 2;
    let center_y = framebuffer.height() / 2;
    framebuffer.set_current_color(if shot_flash { Color::YELLOW } else { Color::WHITE });
    for offset in 0..=4 {
        framebuffer.set_pixel(center_x - offset, center_y);
        framebuffer.set_pixel(center_x + offset, center_y);
        framebuffer.set_pixel(center_x, center_y - offset);
        framebuffer.set_pixel(center_x, center_y + offset);
    }
}
