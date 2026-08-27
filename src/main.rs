mod framebuffer;
mod caster;
mod player;
mod sprites;
mod input;
mod textures;

use caster::cast_ray;
use framebuffer::Framebuffer;
use input::process_events;
use player::Player;
use raylib::prelude::*;
use sprites::{render_sprite, shoot_sprite, Sprite};
use textures::TextureManager;
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
    x_start: u32,
    y_start: u32,
    x_end: u32,
    y_end: u32,
    row: usize,
    column: usize,
    cell: char,
) {
    // pinten un rectangulo de diferente color segun cada char

    let color = match cell {
        '+' | '-' | '|' => {
            if (row + column) % 2 == 0 {
                Color::BLUE
            } else {
                Color::GREEN
            }
        }
        ' ' => Color::BLACK,
        'p' => Color::WHITE,
        'g' => Color::GRAY,
        _ => Color::SKYBLUE,
    };

    framebuffer.set_current_color(color);

    for y in y_start..y_end {
        for x in x_start..x_end {
            framebuffer.set_pixel(x, y);
        }
    }
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Vec<Vec<char>>,
) {
    let columns = maze.iter().map(Vec::len).max().unwrap_or(1) as u32;
    let rows = maze.len().max(1) as u32;

    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let x_start = col_index as u32 * framebuffer.width() / columns;
            let y_start = row_index as u32 * framebuffer.height() / rows;
            let x_end = (col_index as u32 + 1) * framebuffer.width() / columns;
            let y_end = (row_index as u32 + 1) * framebuffer.height() / rows;

            // llamen a su draw cell
            draw_cell(
                framebuffer,
                x_start,
                y_start,
                x_end,
                y_end,
                row_index,
                col_index,
                cell,
            );
        }
    }
}

fn world_to_map_position(
    position: Vector2,
    framebuffer: &Framebuffer,
    maze: &Maze,
    block_size: usize,
) -> Vector2 {
    let columns = maze.iter().map(Vec::len).max().unwrap_or(1) as f32;
    let rows = maze.len().max(1) as f32;

    Vector2::new(
        position.x / (columns * block_size as f32) * framebuffer.width() as f32,
        position.y / (rows * block_size as f32) * framebuffer.height() as f32,
    )
}

fn draw_map_line(framebuffer: &mut Framebuffer, start: Vector2, end: Vector2) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let steps = dx.abs().max(dy.abs()).ceil() as u32;

    for step in 0..=steps {
        let progress = step as f32 / steps.max(1) as f32;
        framebuffer.set_pixel(
            (start.x + dx * progress) as u32,
            (start.y + dy * progress) as u32,
        );
    }
}

fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    const SCALE: u32 = 8;
    const MARGIN: u32 = 12;
    const BORDER: u32 = 2;

    let width = maze.iter().map(Vec::len).max().unwrap_or(0) as u32 * SCALE;
    let height = maze.len() as u32 * SCALE;

    framebuffer.set_current_color(Color::BLACK);
    for y in MARGIN.saturating_sub(BORDER)..MARGIN + height + BORDER {
        for x in MARGIN.saturating_sub(BORDER)..MARGIN + width + BORDER {
            framebuffer.set_pixel(x, y);
        }
    }

    for (row, cells) in maze.iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            let color = match cell {
                '+' | '-' | '|' => Color::BLUE,
                'g' => Color::GRAY,
                _ => Color::DARKGRAY,
            };
            framebuffer.set_current_color(color);

            for y in 0..SCALE {
                for x in 0..SCALE {
                    framebuffer.set_pixel(MARGIN + column as u32 * SCALE + x, MARGIN + row as u32 * SCALE + y);
                }
            }
        }
    }

    let player_x = MARGIN + (player.pos.x / block_size as f32 * SCALE as f32) as u32;
    let player_y = MARGIN + (player.pos.y / block_size as f32 * SCALE as f32) as u32;
    framebuffer.set_current_color(Color::GREEN);
    for y in player_y.saturating_sub(1)..=player_y + 1 {
        for x in player_x.saturating_sub(1)..=player_x + 1 {
            framebuffer.set_pixel(x, y);
        }
    }
}

fn player_reached_goal(player: &Player, maze: &Maze, block_size: usize) -> bool {
    let column = player.pos.x as usize / block_size;
    let row = player.pos.y as usize / block_size;

    row < maze.len() && column < maze[row].len() && maze[row][column] == 'g'
}

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    textures: &TextureManager,
    sprites: &[Sprite],
    time: f32,
) {
    let num_rays = framebuffer.width();
    let mut z_buffer = vec![0.0; num_rays as usize];

    let hw = framebuffer.width() as f32 / 2.0;   // precalculated half width
    let hh = framebuffer.height() as f32 / 2.0;  // precalculated half height

    let horizon = hh as u32;

    framebuffer.set_current_color(Color::new(135, 206, 235, 255));
    for y in 0..horizon {
        for x in 0..framebuffer.width() {
            framebuffer.set_pixel(x, y);
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

        if let Some((texture_width, texture_height)) = textures.dimensions(intersect.impact) {
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
                framebuffer.set_current_color(textures.get_pixel_color(intersect.impact, tx, ty));
                framebuffer.set_pixel(i, y);
            }
        }
    }

    for sprite in sprites {
        render_sprite(framebuffer, player, sprite, textures, &z_buffer, time);
    }

    let center_x = framebuffer.width() / 2;
    let center_y = framebuffer.height() / 2;
    framebuffer.set_current_color(Color::WHITE);
    for offset in 0..=4 {
        framebuffer.set_pixel(center_x - offset, center_y);
        framebuffer.set_pixel(center_x + offset, center_y);
        framebuffer.set_pixel(center_x, center_y - offset);
        framebuffer.set_pixel(center_x, center_y + offset);
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
    window.disable_cursor();

    let mut framebuffer = Framebuffer::new(
        window_width as u32,
        window_height as u32,
        Color::BLACK,
    );
    let textures = TextureManager::new();
    let audio = RaylibAudio::init_audio_device().expect("Failed to initialize audio");
    let music = audio
        .new_music("assets/guichin.mp3")
        .expect("Failed to load background music");
    let shoot_sound = audio
        .new_sound("assets/shoot.wav")
        .expect("Failed to load shoot sound");
    let hit_sound = audio
        .new_sound("assets/hit.wav")
        .expect("Failed to load hit sound");
    music.set_volume(0.08);
    shoot_sound.set_volume(0.85);
    hit_sound.set_volume(0.85);
    music.play_stream();

    let level_files = ["maze.txt", "maze_level_2.txt"];
    let mut selected_level = 0;
    let mut maze = load_maze(level_files[selected_level]);

    let mut player = Player {
        pos: Vector2::new(75.0, 75.0),
        a: 0.0,
        fov: PI / 3.0,
    };
    let mut sprites = [Sprite {
        pos: Vector2::new(300.0, 75.0),
        texture: 's',
        size: block_size as f32,
        active: true,
    }];

    let mut mode_3d = false;
    let mut m_was_down = false;
    let mut welcome_screen = true;
    let mut success_screen = false;

    while !window.window_should_close() {
        let screen_width = window.get_screen_width().max(1) as u32;
        let screen_height = window.get_screen_height().max(1) as u32;
        if framebuffer.width() != screen_width || framebuffer.height() != screen_height {
            framebuffer = Framebuffer::new(screen_width, screen_height, Color::BLACK);
        }

        music.update_stream();

        if welcome_screen {
            if window.is_key_pressed(KeyboardKey::KEY_UP) {
                selected_level = selected_level.saturating_sub(1);
            }
            if window.is_key_pressed(KeyboardKey::KEY_DOWN) {
                selected_level = (selected_level + 1).min(level_files.len() - 1);
            }

            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                maze = load_maze(level_files[selected_level]);
                player.pos = Vector2::new(75.0, 75.0);
                player.a = 0.0;
                sprites[0].active = true;
                welcome_screen = false;
            }

            framebuffer.clear();
            framebuffer.swap_buffers(
                &mut window,
                &raylib_thread,
                welcome_screen,
                selected_level,
                false,
            );
            continue;
        }

        if success_screen {
            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                welcome_screen = true;
                success_screen = false;
            }

            framebuffer.clear();
            framebuffer.swap_buffers(&mut window, &raylib_thread, false, selected_level, true);
            continue;
        }

        // 1. clear framebuffer
        framebuffer.clear();

        // 2. move the player on user input
        let shot_fired = process_events(&window, &mut player, &maze, block_size);

        if player_reached_goal(&player, &maze, block_size) {
            success_screen = true;
            continue;
        }

        if shot_fired && mode_3d {
            shoot_sound.play();

            let wall = cast_ray(
                &mut framebuffer,
                &maze,
                &player,
                player.a,
                block_size,
                false,
            );

            for sprite in &mut sprites {
                if shoot_sprite(&player, sprite, wall.distance) {
                    hit_sound.play();
                }
            }
        }

        let m_is_down = window.is_key_down(KeyboardKey::KEY_M);
        if m_is_down && !m_was_down {
            mode_3d = !mode_3d;
        }
        m_was_down = m_is_down;

        // Clear the framebuffer
        framebuffer.clear();

        // 3. draw stuff
        if !mode_3d {
            render_maze(&mut framebuffer, &maze);
            framebuffer.set_current_color(Color::GREEN);
            let player_position = world_to_map_position(
                player.pos,
                &framebuffer,
                &maze,
                block_size,
            );
            framebuffer.set_pixel(player_position.x as u32, player_position.y as u32);

            let num_rays = 5;

            for i in 0..num_rays {
                let current_ray = i as f32 / num_rays as f32;
                let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

                let intersect = cast_ray(
                    &mut framebuffer,
                    &maze,
                    &player,
                    a,
                    block_size,
                    false,
                );
                framebuffer.set_current_color(Color::GREEN);
                let impact_position = world_to_map_position(
                    Vector2::new(intersect.hit_x, intersect.hit_y),
                    &framebuffer,
                    &maze,
                    block_size,
                );
                draw_map_line(
                    &mut framebuffer,
                    player_position,
                    impact_position,
                );
            }
        } else {
            render_world(
                &mut framebuffer,
                &maze,
                &player,
                block_size,
                &textures,
                &sprites,
                window.get_time() as f32,
            );
            render_minimap(&mut framebuffer, &maze, &player, block_size);
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread, false, selected_level, false);
    }
}
