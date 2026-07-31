mod framebuffer;
mod caster;
mod player;

use caster::cast_ray;
use framebuffer::Framebuffer;
use player::Player;
use raylib::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_maze(filename: &str) -> Vec<Vec<char>> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.unwrap().chars().collect())
        .collect()
}

type Maze = Vec<Vec<char>>;

fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    // pinten un rectangulo de diferente color segun cada char

    let color = match cell {
        '+' | '-' | '|' => Color::BLACK,
        ' ' => Color::WHITE,
        'p' => Color::GREEN,
        'g' => Color::RED,
        _ => Color::MAGENTA,
    };

    framebuffer.set_current_color(color);

    for y in yo..yo + block_size {
        for x in xo..xo + block_size {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Vec<Vec<char>>,
    block_size: usize,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;

            // llamen a su draw cell
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Mundo 3D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(
        window_width as u32,
        window_height as u32,
        Color::BLACK,
    );

    let maze = load_maze("./maze.txt");
    let player = Player {
        pos: Vector2::new(75.0, 75.0),
        a: 0.0,
    };

    while !window.window_should_close() {
        framebuffer.clear();
        render_maze(&mut framebuffer, &maze, 50);
        draw_cell(
            &mut framebuffer,
            (player.pos.x as usize / 50) * 50,
            (player.pos.y as usize / 50) * 50,
            50,
            'p',
        );
        cast_ray(&mut framebuffer, &maze, &player, 50);
        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
