mod framebuffer;
mod hud;
mod map_view;
mod level;
mod combat;
mod world_renderer;
mod caster;
mod player;
mod sprites;
mod input;
mod textures;

use caster::cast_ray;
use framebuffer::Framebuffer;
use hud::render_hud;
use map_view::{draw_map_line, render_maze, render_minimap, world_to_map_position};
use level::{load_maze, player_start_angle, player_start_position};
use world_renderer::render_world;
use combat::{
    collect_pickups, interact_with_level, spawn_enemies, spawn_pickups, update_enemies,
    update_projectiles, PickupKind, MAX_AMMO,
};
pub use level::Maze;
use input::process_events;
use player::Player;
use raylib::prelude::*;
use sprites::{shoot_sprite, Sprite};
use textures::TextureManager;
use std::f32::consts::PI;

fn player_reached_goal(
    player: &Player,
    maze: &Maze,
    sprites: &[Sprite],
    exit_unlocked: bool,
    block_size: usize,
) -> bool {
    let column = player.pos.x as usize / block_size;
    let row = player.pos.y as usize / block_size;

    row < maze.len()
        && column < maze[row].len()
        && maze[row][column] == 'g'
        && sprites.iter().all(|sprite| !sprite.active)
        && exit_unlocked
}

fn main() {
    let window_width = 1200;
    let window_height = 900;
    let normal_render_width = 1000;
    let fullscreen_render_width = 1600;
    let block_size = 15;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Droom Ball Super")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    window.disable_cursor();

    let mut framebuffer = Framebuffer::new(
        normal_render_width,
        normal_render_width * window_height as u32 / window_width as u32,
        Color::BLACK,
    );
    let audio = RaylibAudio::init_audio_device().expect("Failed to initialize audio");
    audio.set_audio_stream_buffer_size_default(65_536);
    let music = audio
        .new_music("assets/holdontight.mp3")
        .expect("Failed to load background music");
    let shoot_sound = audio
        .new_sound("assets/shoot.mp3")
        .expect("Failed to load shoot sound");
    let hit_sound = audio
        .new_sound("assets/hit.mp3")
        .expect("Failed to load hit sound");
    let cure_sound = audio
        .new_sound("assets/cura.mp3")
        .expect("Failed to load cure sound");
    let ki_sound = audio
        .new_sound("assets/ki.mp3")
        .expect("Failed to load ki sound");
    let shenron_sound = audio
        .new_sound("assets/shenron.mp3")
        .expect("Failed to load shenron sound");
    music.set_volume(2.0);
    shoot_sound.set_volume(5.0);
    hit_sound.set_volume(5.0);
    cure_sound.set_volume(5.0);
    ki_sound.set_volume(5.0);
    shenron_sound.set_volume(5.0);
    music.play_stream();

    let level_files = ["maps/mapa1.txt", "maps/mapa3.txt", "maps/mapa2.txt"];
    let mut selected_level = 0;
    let mut textures = TextureManager::new(selected_level);
    let mut highest_unlocked_level = 0;
    let mut maze = load_maze(level_files[selected_level]);

    let mut player = Player {
        pos: player_start_position(&maze, block_size),
        a: player_start_angle(&maze),
        fov: PI / 3.0,
    };
    let mut sprites = spawn_enemies(selected_level, &maze, block_size);
    let mut pickups = spawn_pickups(selected_level, &maze, block_size);
    let mut player_health = 100;
    let mut ammo = 6;
    let mut exit_unlocked = false;
    let mut projectiles = Vec::new();
    let mut shot_flash = 0.0;
    let mut frame_index: usize = 0;

    let mut mode_3d = false;
    let mut m_was_down = false;
    let mut welcome_screen = true;
    let mut success_screen = false;
    let mut defeat_screen = false;

    while !window.window_should_close() {
        let screen_width = window.get_screen_width().max(1) as u32;
        let screen_height = window.get_screen_height().max(1) as u32;
        let render_width = if window.is_window_fullscreen()
            || (screen_width >= 1800 && screen_height >= 1000)
        {
            fullscreen_render_width
        } else {
            normal_render_width
        };
        let render_height = (render_width as f32 * screen_height as f32 / screen_width as f32)
            .round()
            .max(1.0) as u32;
        if framebuffer.width() != render_width || framebuffer.height() != render_height {
            framebuffer = Framebuffer::new(render_width, render_height, Color::BLACK);
        }

        music.update_stream();

        if welcome_screen {
            if window.is_key_pressed(KeyboardKey::KEY_W) {
                selected_level = selected_level.saturating_sub(1);
            }
            if window.is_key_pressed(KeyboardKey::KEY_S) {
                selected_level = (selected_level + 1).min(highest_unlocked_level);
            }

            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                maze = load_maze(level_files[selected_level]);
                textures = TextureManager::new(selected_level);
                player.pos = player_start_position(&maze, block_size);
                player.a = player_start_angle(&maze);
                sprites = spawn_enemies(selected_level, &maze, block_size);
                pickups = spawn_pickups(selected_level, &maze, block_size);
                player_health = 100;
                ammo = 6;
                exit_unlocked = false;
                projectiles.clear();
                shot_flash = 0.0;
                welcome_screen = false;
                success_screen = false;
            }

            framebuffer.clear();
            framebuffer.swap_buffers(
                &mut window,
                &raylib_thread,
                welcome_screen,
                selected_level,
                highest_unlocked_level,
                false,
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
            framebuffer.swap_buffers(
                &mut window,
                &raylib_thread,
                false,
                selected_level,
                highest_unlocked_level,
                true,
                false,
            );
            continue;
        }

        if defeat_screen {
            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                maze = load_maze(level_files[selected_level]);
                textures = TextureManager::new(selected_level);
                player.pos = player_start_position(&maze, block_size);
                player.a = player_start_angle(&maze);
                sprites = spawn_enemies(selected_level, &maze, block_size);
                pickups = spawn_pickups(selected_level, &maze, block_size);
                player_health = 100;
                ammo = 6;
                exit_unlocked = false;
                projectiles.clear();
                shot_flash = 0.0;
                defeat_screen = false;
            } else if window.is_key_pressed(KeyboardKey::KEY_L) {
                welcome_screen = true;
                defeat_screen = false;
            }

            framebuffer.clear();
            framebuffer.swap_buffers(
                &mut window,
                &raylib_thread,
                false,
                selected_level,
                highest_unlocked_level,
                false,
                true,
            );
            continue;
        }

        // 1. move the player on user input
        let delta_time = window.get_frame_time();
        shot_flash = (shot_flash - delta_time).max(0.0);
        let shot_fired = process_events(&window, &mut player, &maze, block_size);
        if let Some(kind) = collect_pickups(
            &player,
            &mut pickups,
            &mut player_health,
            &mut ammo,
        ) {
            match kind {
                PickupKind::Health => cure_sound.play(),
                PickupKind::Ammo => ki_sound.play(),
                PickupKind::Switch => {}
            }
        }

        if window.is_key_pressed(KeyboardKey::KEY_E)
            && interact_with_level(
                &player,
                &mut maze,
                &mut pickups,
                &mut exit_unlocked,
                block_size,
            )
        {
            shenron_sound.play();
        }

        if player_reached_goal(&player, &maze, &sprites, exit_unlocked, block_size) {
            shenron_sound.play();
            highest_unlocked_level = (selected_level + 1).min(level_files.len() - 1);
            success_screen = true;
            continue;
        }

        if shot_fired && mode_3d && ammo > 0 {
            ammo -= 1;
            shoot_sound.play();
            shot_flash = 0.08;

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

        if mode_3d {
            let enemy_shots = update_enemies(
                &mut framebuffer,
                &maze,
                &player,
                &mut sprites,
                &mut projectiles,
                block_size,
                delta_time,
            );
            if enemy_shots > 0 {
                hit_sound.play();
            }

            let projectile_hits =
                update_projectiles(&mut projectiles, &player, &maze, block_size, delta_time);
            if projectile_hits > 0 {
                player_health = (player_health - 10 * projectile_hits as i32).max(0);
                hit_sound.play();
                if player_health == 0 {
                    defeat_screen = true;
                }
            }
            projectiles.retain(|projectile| projectile.active);
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
            render_maze(&mut framebuffer, &maze, &textures);
            framebuffer.set_current_color(Color::SKYBLUE);
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
                &pickups,
                &projectiles,
                frame_index,
                shot_flash > 0.0,
            );
            render_minimap(
                &mut framebuffer,
                &maze,
                &player,
                &sprites,
                &pickups,
                selected_level,
                block_size,
            );
            render_hud(
                &mut framebuffer,
                player_health,
                ammo,
                MAX_AMMO,
                sprites.iter().filter(|sprite| sprite.active).count(),
            );
        }

        framebuffer.swap_buffers(
            &mut window,
            &raylib_thread,
            false,
            selected_level,
            highest_unlocked_level,
            false,
            false,
        );
        frame_index = frame_index.wrapping_add(1);
    }
}
