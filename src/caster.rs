// caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::Maze;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub cell_x: usize,
    pub cell_y: usize,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let mut d = 0.0;

    framebuffer.set_current_color(Color::GREEN);

    loop {
        let cos = d * a.cos();
        let sin = d * a.sin();

        let world_x = player.pos.x + cos;
        let world_y = player.pos.y + sin;

        if world_x < 0.0 || world_y < 0.0 {
            return Intersect {
                distance: d,
                impact: '#',
                cell_x: 0,
                cell_y: 0,
            };
        }

        let x = world_x as usize;
        let y = world_y as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect {
                distance: d,
                impact: '#',
                cell_x: i,
                cell_y: j,
            };
        }

        if maze[j][i] != ' ' {
            return Intersect {
                distance: d,
                impact: maze[j][i],
                cell_x: i,
                cell_y: j,
            };
        }

        if draw_line {
            framebuffer.set_pixel(x as u32, y as u32);
        }

        d += 1.0;
    }
}
