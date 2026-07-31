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

pub type Maze = Vec<Vec<char>>;

fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    // pinten un rectangulo de diferente color segun cada char

    let color = match cell {
        '+' | '-' | '|' => {
            if (xo / block_size + yo / block_size) % 2 == 0 {
                Color::BLACK
            } else {
                Color::DARKGRAY
            }
        }
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

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    let num_rays = framebuffer.width();

    let hw = framebuffer.width() as f32 / 2.0;   // precalculated half width
    let hh = framebuffer.height() as f32 / 2.0;  // precalculated half height

    let horizon = hh as u32;

    framebuffer.set_current_color(Color::new(135, 206, 235, 255));
    for y in 0..horizon {
        for x in 0..framebuffer.width() {
            framebuffer.set_pixel(x, y);
        }
    }

    framebuffer.set_current_color(Color::new(90, 70, 50, 255));
    for y in horizon..framebuffer.height() {
        for x in 0..framebuffer.width() {
            framebuffer.set_pixel(x, y);
        }
    }

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

        let intersect = cast_ray(
            framebuffer,
            &maze,
            player,
            a,
            block_size,
            false,
        );

        // Calculate the height of the stake
        let distance_to_wall = intersect.distance * (player.a - a).cos(); // fish-eye correction
        let distance_to_projection_plane = hw / (player.fov / 2.0).tan(); // distance from the "camera"

        if intersect.impact == 'g' {
            framebuffer.set_current_color(Color::RED);
        } else if (intersect.cell_x + intersect.cell_y) % 2 == 0 {
            framebuffer.set_current_color(Color::BLACK);
        } else {
            framebuffer.set_current_color(Color::DARKGRAY);
        }

        // this ratio doesn't really matter as long as it is a function of distance
        let distance_to_wall = distance_to_wall.max(1.0);
        let stake_height =
            (block_size as f32 / distance_to_wall) * distance_to_projection_plane;

        // Calculate the position to draw the stake
        let stake_top = (hh - (stake_height / 2.0)).max(0.0) as u32;
        let stake_bottom = (hh + (stake_height / 2.0))
            .min(framebuffer.height() as f32) as u32;

        // Draw the stake directly in the framebuffer
        for y in stake_top..stake_bottom {
            framebuffer.set_pixel(i, y);
        }
    }
}

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
        a: 0.0,
        fov: PI / 3.0,
    };

    let mut mode_3d = false;
    let mut m_was_down = false;

    while !window.window_should_close() {
        // 1. clear framebuffer
        framebuffer.clear();

        // 2. move the player on user input
        process_events(&window, &mut player, &maze, block_size);

        let m_is_down = window.is_key_down(KeyboardKey::KEY_M);
        if m_is_down && !m_was_down {
            mode_3d = !mode_3d;
        }
        m_was_down = m_is_down;

        // Clear the framebuffer
        framebuffer.clear();

        // 3. draw stuff
        if !mode_3d {
            render_maze(&mut framebuffer, &maze, block_size);
            framebuffer.set_current_color(Color::GREEN);
            framebuffer.set_pixel(player.pos.x as u32, player.pos.y as u32);

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
                    true,
                );
            }
        } else {
            render_world(&mut framebuffer, &maze, &player, block_size);
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
