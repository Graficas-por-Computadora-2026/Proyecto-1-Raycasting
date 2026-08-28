// caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::Maze;

pub struct Intersect {
    pub distance: f32,
    pub cell_x: usize,
    pub cell_y: usize,
    pub hit_x: f32,
    pub hit_y: f32,
    pub hit_vertical: bool,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let ray_x = a.cos();
    let ray_y = a.sin();
    let block = block_size as f32;
    let mut cell_x = (player.pos.x / block) as i32;
    let mut cell_y = (player.pos.y / block) as i32;

    let delta_x = if ray_x.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        block / ray_x.abs()
    };
    let delta_y = if ray_y.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        block / ray_y.abs()
    };

    let step_x = if ray_x < 0.0 { -1 } else { 1 };
    let step_y = if ray_y < 0.0 { -1 } else { 1 };
    let mut side_x = if ray_x < 0.0 {
        (player.pos.x - cell_x as f32 * block) / ray_x.abs()
    } else {
        ((cell_x + 1) as f32 * block - player.pos.x) / ray_x.abs()
    };
    let mut side_y = if ray_y < 0.0 {
        (player.pos.y - cell_y as f32 * block) / ray_y.abs()
    } else {
        ((cell_y + 1) as f32 * block - player.pos.y) / ray_y.abs()
    };

    framebuffer.set_current_color(Color::GREEN);

    loop {
        let hit_vertical;
        let distance;

        if side_x < side_y {
            side_x += delta_x;
            cell_x += step_x;
            hit_vertical = true;
            distance = side_x - delta_x;
        } else {
            side_y += delta_y;
            cell_y += step_y;
            hit_vertical = false;
            distance = side_y - delta_y;
        }

        let hit_x = player.pos.x + distance * ray_x;
        let hit_y = player.pos.y + distance * ray_y;

        if cell_x < 0
            || cell_y < 0
            || cell_y as usize >= maze.len()
            || cell_x as usize >= maze[cell_y as usize].len()
        {
            return Intersect {
                distance,
                cell_x: cell_x.max(0) as usize,
                cell_y: cell_y.max(0) as usize,
                hit_x,
                hit_y,
                hit_vertical,
            };
        }

        let impact = maze[cell_y as usize][cell_x as usize];
        if impact != ' ' && impact != 'p' {
            if draw_line {
                for d in 0..distance.ceil() as u32 {
                    let x = player.pos.x + d as f32 * ray_x;
                    let y = player.pos.y + d as f32 * ray_y;
                    framebuffer.set_pixel(x as u32, y as u32);
                }
            }

            return Intersect {
                distance,
                cell_x: cell_x as usize,
                cell_y: cell_y as usize,
                hit_x,
                hit_y,
                hit_vertical,
            };
        }
    }
}
