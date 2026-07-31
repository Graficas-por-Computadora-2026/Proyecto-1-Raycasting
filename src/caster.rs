// caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::player::Player;

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Vec<Vec<char>>,
    player: &Player,
    an: f32,
    block_size: usize,
) {
    let mut d = 0.0;

    framebuffer.set_current_color(Color::GREEN);

    loop {
        let cos = d * an.cos();
        let sin = d * an.sin();

        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;

        // convert pixels to a position in the maze
        let i = x / block_size;
        let j = y / block_size;

        // if the current item is not a space,
        // we have hit a wall and we stop
        if maze[j][i] != ' ' {
            break;
        }

        framebuffer.set_pixel(x as u32, y as u32);

        d += 10.0;
    }
}
