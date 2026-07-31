mod framebuffer;
mod caster;
mod player;
mod input;

use caster::cast_ray;
use framebuffer::Framebuffer;
use input::process_events;
use player::Player;
use raylib::prelude::*;
use std::f32::consts::PI;
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

fn render_world(_framebuffer: &mut Framebuffer, _player: &Player) {}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let block_size = 50;

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

    let maze = load_maze("maze.txt");

    let mut player = Player {
        pos: Vector2::new(75.0, 75.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    while !window.window_should_close() {
        // 1. clear framebuffer
        framebuffer.clear();

        // 2. move the player on user input
        process_events(&window, &mut player);

        let mut mode = "2D";

        if window.is_key_down(KeyboardKey::KEY_M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        // Clear the framebuffer
        framebuffer.clear();

        // 3. draw stuff
        if mode == "2D" {
            render_maze(&mut framebuffer, &maze, block_size);
            framebuffer.set_current_color(Color::GREEN);
            framebuffer.set_pixel(player.pos.x as u32, player.pos.y as u32);
        } else {
            render_world(&mut framebuffer, &player);
        }

        // draw what the player sees
        let num_rays = 5;

        for i in 0..num_rays {
            let current_ray = i as f32 / num_rays as f32;
            let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

            cast_ray(
                &mut framebuffer,
                &maze,
                &player,
                a,
                block_size,
            );
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
