mod framebuffer;
mod hud;
mod caster;
mod player;
mod sprites;
mod input;
mod textures;

use caster::cast_ray;
use framebuffer::Framebuffer;
use hud::render_hud;
use input::process_events;
use player::Player;
use raylib::prelude::*;
use sprites::{render_sprite, shoot_sprite, EnemyKind, Sprite};
use textures::TextureManager;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::VecDeque;

pub fn load_maze(filename: &str) -> Vec<Vec<char>> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let source: Vec<Vec<char>> = reader
        .lines()
        .map(|line| line.unwrap().chars().collect())
        .collect();

    if filename.starts_with("maps/") {
        let width = source.iter().map(Vec::len).max().unwrap_or(1) * 2;
        let mut maze = Vec::new();

        for row in source {
            let mut dot_rows = vec![vec!['+'; width]; 4];
            for (column, cell) in row.into_iter().enumerate() {
                let dots = (cell as u32).saturating_sub(0x2800);
                for (bit, x, y) in [
                    (0, 0, 0), (1, 0, 1), (2, 0, 2), (6, 0, 3),
                    (3, 1, 0), (4, 1, 1), (5, 1, 2), (7, 1, 3),
                ] {
                    if dots & (1 << bit) != 0 {
                        dot_rows[y][column * 2 + x] = ' ';
                    }
                }
            }
            maze.extend(dot_rows);
        }
        mark_art_start_and_goal(&mut maze);
        maze
    } else {
        source
    }
}

pub type Maze = Vec<Vec<char>>;

fn is_walkable_cell(cell: char) -> bool {
    matches!(cell, ' ' | 'p' | 'g')
}

fn farthest_cell(maze: &Maze, start: (usize, usize)) -> (usize, usize) {
    let mut distances = vec![vec![None; maze[0].len()]; maze.len()];
    let mut queue = VecDeque::from([start]);
    distances[start.1][start.0] = Some(0usize);
    let mut farthest = start;

    while let Some((x, y)) = queue.pop_front() {
        let distance = distances[y][x].unwrap();
        if distance > distances[farthest.1][farthest.0].unwrap() {
            farthest = (x, y);
        }

        for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
            let next_x = x as isize + dx;
            let next_y = y as isize + dy;
            if next_y < 0
                || next_y as usize >= maze.len()
                || next_x < 0
                || next_x as usize >= maze[next_y as usize].len()
            {
                continue;
            }

            let (next_x, next_y) = (next_x as usize, next_y as usize);
            if distances[next_y][next_x].is_none() && is_walkable_cell(maze[next_y][next_x]) {
                distances[next_y][next_x] = Some(distance + 1);
                queue.push_back((next_x, next_y));
            }
        }
    }

    farthest
}

fn mark_art_start_and_goal(maze: &mut Maze) {
    let mut visited = vec![vec![false; maze[0].len()]; maze.len()];
    let mut largest_component = Vec::new();

    for y in 0..maze.len() {
        for x in 0..maze[y].len() {
            if visited[y][x] || !is_walkable_cell(maze[y][x]) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = VecDeque::from([(x, y)]);
            visited[y][x] = true;
            while let Some((cell_x, cell_y)) = queue.pop_front() {
                component.push((cell_x, cell_y));
                for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                    let next_x = cell_x as isize + dx;
                    let next_y = cell_y as isize + dy;
                    if next_y < 0
                        || next_y as usize >= maze.len()
                        || next_x < 0
                        || next_x as usize >= maze[next_y as usize].len()
                    {
                        continue;
                    }
                    let (next_x, next_y) = (next_x as usize, next_y as usize);
                    if !visited[next_y][next_x] && is_walkable_cell(maze[next_y][next_x]) {
                        visited[next_y][next_x] = true;
                        queue.push_back((next_x, next_y));
                    }
                }
            }

            if component.len() > largest_component.len() {
                largest_component = component;
            }
        }
    }

    if let Some(&cell) = largest_component.first() {
        let start = farthest_cell(maze, cell);
        let goal = farthest_cell(maze, start);
        maze[start.1][start.0] = 'p';
        maze[goal.1][goal.0] = 'g';
    }
}

const BASE_ASPECT_RATIO: f32 = 800.0 / 600.0;
const MAX_AMMO: i32 = 12;

#[derive(Clone, Copy)]
enum PickupKind {
    Health,
    Ammo,
    Key,
    Switch,
}

struct Pickup {
    pos: Vector2,
    kind: PickupKind,
    active: bool,
}

struct Projectile {
    pos: Vector2,
    direction: Vector2,
    active: bool,
}

fn draw_cell(
    framebuffer: &mut Framebuffer,
    textures: &TextureManager,
    x_start: u32,
    y_start: u32,
    x_end: u32,
    y_end: u32,
    cell: char,
) {
    let color = match cell {
        ' ' => Color::BLACK,
        'p' => Color::MAGENTA,
        'g' => Color::RED,
        _ => Color::WHITE,
    };

    for y in y_start..y_end {
        for x in x_start..x_end {
            if matches!(cell, ' ' | 'p' | 'g') {
                framebuffer.set_current_color(color);
            } else if let Some((texture_width, texture_height)) = textures.dimensions(cell) {
                let tx = ((x - x_start) * texture_width / (x_end - x_start).max(1)) as u32;
                let ty = ((y - y_start) * texture_height / (y_end - y_start).max(1)) as u32;
                framebuffer.set_current_color(textures.get_pixel_color(cell, tx, ty));
            }
            framebuffer.set_pixel(x, y);
        }
    }
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Vec<Vec<char>>,
    textures: &TextureManager,
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
                textures,
                x_start,
                y_start,
                x_end,
                y_end,
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
    const MARGIN: u32 = 12;
    const BORDER: u32 = 2;

    let columns = maze.iter().map(Vec::len).max().unwrap_or(1) as u32;
    let rows = maze.len().max(1) as u32;
    let max_width = framebuffer.width() / 4;
    let max_height = framebuffer.height() / 4;
    let target_height = (rows * max_width + columns - 1) / columns;
    let (width, height) = if target_height > max_height {
        (max_height * columns / rows, max_height)
    } else {
        (max_width, target_height)
    };
    let margin_x = framebuffer.width().saturating_sub(width + MARGIN);
    let margin_y = MARGIN;

    framebuffer.set_current_color(Color::WHITE);
    for y in margin_y.saturating_sub(BORDER)..margin_y + height + BORDER {
        for x in margin_x.saturating_sub(BORDER)..margin_x + width + BORDER {
            framebuffer.set_pixel(x, y);
        }
    }

    for (row, cells) in maze.iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            let color = match cell {
                '+' | '-' | '|' | 'D' => Color::BLACK,
                'g' => Color::RED,
                'p' => Color::MAGENTA,
                _ => Color::WHITE,
            };
            framebuffer.set_current_color(color);

            let x_start = margin_x + column as u32 * width / columns;
            let x_end = margin_x + (column as u32 + 1) * width / columns;
            let y_start = margin_y + row as u32 * height / rows;
            let y_end = margin_y + (row as u32 + 1) * height / rows;
            for y in y_start..y_end {
                for x in x_start..x_end {
                    framebuffer.set_pixel(x, y);
                }
            }
        }
    }

    for (marker, color) in [('p', Color::MAGENTA), ('g', Color::RED)] {
        if let Some((row, column)) = maze.iter().enumerate().find_map(|(row, cells)| {
            cells
                .iter()
                .position(|cell| *cell == marker)
                .map(|column| (row, column))
        }) {
            let marker_x = margin_x + (column as u32 * width / columns);
            let marker_y = margin_y + (row as u32 * height / rows);
            framebuffer.set_current_color(color);
            for y in marker_y.saturating_sub(2)..=marker_y + 2 {
                for x in marker_x.saturating_sub(2)..=marker_x + 2 {
                    framebuffer.set_pixel(x, y);
                }
            }
        }
    }

    let player_x = margin_x + (player.pos.x / (block_size as f32 * columns as f32) * width as f32) as u32;
    let player_y = margin_y + (player.pos.y / (block_size as f32 * rows as f32) * height as f32) as u32;
    framebuffer.set_current_color(Color::SKYBLUE);
    for y in player_y.saturating_sub(3)..=player_y + 3 {
        for x in player_x.saturating_sub(3)..=player_x + 3 {
            framebuffer.set_pixel(x, y);
        }
    }
}

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

fn player_start_position(maze: &Maze, block_size: usize) -> Vector2 {
    for (row, cells) in maze.iter().enumerate() {
        if let Some(column) = cells.iter().position(|cell| *cell == 'p') {
            return Vector2::new(
                (column as f32 + 0.5) * block_size as f32,
                (row as f32 + 0.5) * block_size as f32,
            );
        }
    }

    Vector2::new(75.0, 75.0)
}

fn player_start_angle(maze: &Maze) -> f32 {
    for (row, cells) in maze.iter().enumerate() {
        if let Some(column) = cells.iter().position(|cell| *cell == 'p') {
            for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                let next_x = column as isize + dx;
                let next_y = row as isize + dy;
                if next_y >= 0
                    && (next_y as usize) < maze.len()
                    && next_x >= 0
                    && (next_x as usize) < maze[next_y as usize].len()
                    && is_walkable_cell(maze[next_y as usize][next_x as usize])
                {
                    return (dy as f32).atan2(dx as f32);
                }
            }
        }
    }

    0.0
}

fn reachable_cells(maze: &Maze) -> Vec<(usize, usize)> {
    let Some((start_x, start_y)) = maze.iter().enumerate().find_map(|(row, cells)| {
        cells
            .iter()
            .position(|cell| *cell == 'p')
            .map(|column| (column, row))
    }) else {
        return Vec::new();
    };

    let mut cells = Vec::new();
    let mut visited = vec![vec![false; maze[0].len()]; maze.len()];
    let mut queue = VecDeque::from([(start_x, start_y)]);
    visited[start_y][start_x] = true;

    while let Some((x, y)) = queue.pop_front() {
        cells.push((x, y));
        for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
            let next_x = x as isize + dx;
            let next_y = y as isize + dy;
            if next_y < 0
                || next_y as usize >= maze.len()
                || next_x < 0
                || next_x as usize >= maze[next_y as usize].len()
            {
                continue;
            }
            let (next_x, next_y) = (next_x as usize, next_y as usize);
            if !visited[next_y][next_x] && is_walkable_cell(maze[next_y][next_x]) {
                visited[next_y][next_x] = true;
                queue.push_back((next_x, next_y));
            }
        }
    }

    cells
}

fn cell_position(cell: (usize, usize), block_size: usize) -> Vector2 {
    Vector2::new(
        (cell.0 as f32 + 0.5) * block_size as f32,
        (cell.1 as f32 + 0.5) * block_size as f32,
    )
}

fn spawn_enemy(position: Vector2, kind: EnemyKind, block_size: usize) -> Sprite {
    let health = match kind {
        EnemyKind::Grunt => 3,
        EnemyKind::Brute => 5,
    };

    Sprite {
        pos: position,
        texture: 's',
        size: block_size as f32,
        active: true,
        health,
        attack_cooldown: 0.0,
        kind,
    }
}

fn spawn_enemies(level: usize, maze: &Maze, block_size: usize) -> Vec<Sprite> {
    let cells = reachable_cells(maze);
    if !cells.is_empty() {
        let enemy_cells = [cells[cells.len() / 3], cells[cells.len() * 2 / 3]];
        return enemy_cells
            .into_iter()
            .enumerate()
            .map(|(index, cell)| {
                let kind = if index == 0 {
                    EnemyKind::Grunt
                } else {
                    EnemyKind::Brute
                };
                spawn_enemy(cell_position(cell, block_size), kind, block_size)
            })
            .collect();
    }

    let positions = match level {
        0 => vec![
            (Vector2::new(300.0, 75.0), EnemyKind::Grunt),
            (Vector2::new(500.0, 75.0), EnemyKind::Brute),
        ],
        1 => vec![
            (Vector2::new(250.0, 75.0), EnemyKind::Grunt),
            (Vector2::new(425.0, 75.0), EnemyKind::Grunt),
            (Vector2::new(550.0, 75.0), EnemyKind::Brute),
        ],
        _ => Vec::new(),
    };

    positions
        .into_iter()
        .map(|(position, kind)| spawn_enemy(position, kind, block_size))
        .collect()
}

fn spawn_pickups(level: usize, maze: &Maze, block_size: usize) -> Vec<Pickup> {
    let cells = reachable_cells(maze);
    if !cells.is_empty() {
        let pickups = [
            (cells[cells.len() / 6], PickupKind::Ammo),
            (cells[cells.len() / 2], PickupKind::Health),
            (cells[cells.len() * 4 / 5], PickupKind::Switch),
        ];
        return pickups
            .into_iter()
            .map(|(cell, kind)| Pickup {
                pos: cell_position(cell, block_size),
                kind,
                active: true,
            })
            .collect();
    }

    let items = match level {
        0 => vec![
            (Vector2::new(125.0, 75.0), PickupKind::Key),
            (Vector2::new(175.0, 75.0), PickupKind::Ammo),
            (Vector2::new(350.0, 75.0), PickupKind::Health),
            (Vector2::new(425.0, 75.0), PickupKind::Switch),
        ],
        1 => vec![
            (Vector2::new(125.0, 75.0), PickupKind::Key),
            (Vector2::new(150.0, 75.0), PickupKind::Health),
            (Vector2::new(350.0, 75.0), PickupKind::Ammo),
            (Vector2::new(500.0, 75.0), PickupKind::Ammo),
            (Vector2::new(450.0, 75.0), PickupKind::Switch),
        ],
        _ => Vec::new(),
    };

    items
        .into_iter()
        .map(|(pos, kind)| Pickup {
            pos,
            kind,
            active: true,
        })
        .collect()
}

fn collect_pickups(
    player: &Player,
    pickups: &mut [Pickup],
    health: &mut i32,
    ammo: &mut i32,
    has_key: &mut bool,
) -> bool {
    const PICKUP_DISTANCE: f32 = 28.0;
    let mut collected = false;

    for pickup in pickups {
        if !pickup.active || player.pos.distance_to(pickup.pos) > PICKUP_DISTANCE {
            continue;
        }

        match pickup.kind {
            PickupKind::Health => *health = (*health + 25).min(100),
            PickupKind::Ammo => *ammo = (*ammo + 6).min(MAX_AMMO),
            PickupKind::Key => *has_key = true,
            PickupKind::Switch => continue,
        }
        pickup.active = false;
        collected = true;
    }

    collected
}

fn interact_with_level(
    player: &Player,
    maze: &mut Maze,
    pickups: &mut [Pickup],
    has_key: &mut bool,
    exit_unlocked: &mut bool,
    block_size: usize,
) -> bool {
    const INTERACTION_DISTANCE: f32 = 40.0;

    if *has_key {
        for (row, cells) in maze.iter_mut().enumerate() {
            for (column, cell) in cells.iter_mut().enumerate() {
                if *cell != 'D' {
                    continue;
                }

                let door_position = Vector2::new(
                    (column as f32 + 0.5) * block_size as f32,
                    (row as f32 + 0.5) * block_size as f32,
                );
                if player.pos.distance_to(door_position) <= INTERACTION_DISTANCE {
                    *cell = ' ';
                    *has_key = false;
                    return true;
                }
            }
        }
    }

    for pickup in pickups {
        if matches!(pickup.kind, PickupKind::Switch)
            && pickup.active
            && player.pos.distance_to(pickup.pos) <= INTERACTION_DISTANCE
        {
            pickup.active = false;
            *exit_unlocked = true;
            return true;
        }
    }

    false
}

fn enemy_can_move(position: Vector2, maze: &Maze, block_size: usize) -> bool {
    if position.x < 0.0 || position.y < 0.0 {
        return false;
    }

    let column = position.x as usize / block_size;
    let row = position.y as usize / block_size;
    row < maze.len()
        && column < maze[row].len()
        && matches!(maze[row][column], ' ' | 'p' | 'g')
}

fn update_enemies(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    sprites: &mut [Sprite],
    projectiles: &mut Vec<Projectile>,
    block_size: usize,
    delta_time: f32,
) -> usize {
    const ATTACK_RANGE: f32 = 220.0;
    let mut shots_fired = 0;

    for sprite in sprites {
        if !sprite.active {
            continue;
        }

        sprite.attack_cooldown = (sprite.attack_cooldown - delta_time).max(0.0);
        let dx = player.pos.x - sprite.pos.x;
        let dy = player.pos.y - sprite.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let angle_from_player = (sprite.pos.y - player.pos.y).atan2(sprite.pos.x - player.pos.x);
        let wall = cast_ray(
            framebuffer,
            maze,
            player,
            angle_from_player,
            block_size,
            false,
        );

        // The enemy only reacts when the player is in direct line of sight.
        if distance >= wall.distance {
            continue;
        }

        let (speed, attack_delay, projectile_speed) = match sprite.kind {
            EnemyKind::Grunt => (25.0, 1.2, 90.0),
            EnemyKind::Brute => (15.0, 1.8, 70.0),
        };

        if distance > ATTACK_RANGE {
            let step = (speed * delta_time).min(distance - ATTACK_RANGE);
            let next_position = Vector2::new(
                sprite.pos.x + dx / distance * step,
                sprite.pos.y + dy / distance * step,
            );

            if enemy_can_move(next_position, maze, block_size) {
                sprite.pos = next_position;
            }
        } else if sprite.attack_cooldown == 0.0 {
            sprite.attack_cooldown = attack_delay;
            projectiles.push(Projectile {
                pos: sprite.pos,
                direction: Vector2::new(
                    dx / distance * projectile_speed,
                    dy / distance * projectile_speed,
                ),
                active: true,
            });
            shots_fired += 1;
        }
    }

    shots_fired
}

fn update_projectiles(
    projectiles: &mut [Projectile],
    player: &Player,
    maze: &Maze,
    block_size: usize,
    delta_time: f32,
) -> usize {
    const HIT_DISTANCE: f32 = 14.0;
    let mut hits = 0;

    for projectile in projectiles {
        if !projectile.active {
            continue;
        }

        let next_position = Vector2::new(
            projectile.pos.x + projectile.direction.x * delta_time,
            projectile.pos.y + projectile.direction.y * delta_time,
        );

        if next_position.distance_to(player.pos) <= HIT_DISTANCE {
            projectile.active = false;
            hits += 1;
        } else if enemy_can_move(next_position, maze, block_size) {
            projectile.pos = next_position;
        } else {
            projectile.active = false;
        }
    }

    hits
}

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    textures: &TextureManager,
    sprites: &[Sprite],
    pickups: &[Pickup],
    projectiles: &[Projectile],
    time: f32,
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
        let tint = match sprite.kind {
            EnemyKind::Grunt => Color::WHITE,
            EnemyKind::Brute => Color::RED,
        };
        render_sprite(
            framebuffer,
            &render_player,
            sprite,
            textures,
            &z_buffer,
            time,
            tint,
        );
    }

    for pickup in pickups {
        if !pickup.active {
            continue;
        }

        let pickup_sprite = Sprite {
            pos: pickup.pos,
            texture: 's',
            size: block_size as f32 * 0.6,
            active: true,
            health: 0,
            attack_cooldown: 0.0,
            kind: EnemyKind::Grunt,
        };
        let tint = match pickup.kind {
            PickupKind::Health => Color::GREEN,
            PickupKind::Ammo => Color::YELLOW,
            PickupKind::Key => Color::SKYBLUE,
            PickupKind::Switch => Color::PURPLE,
        };
        render_sprite(
            framebuffer,
            &render_player,
            &pickup_sprite,
            textures,
            &z_buffer,
            time,
            tint,
        );
    }

    for projectile in projectiles {
        if !projectile.active {
            continue;
        }

        let projectile_sprite = Sprite {
            pos: projectile.pos,
            texture: 's',
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
            time,
            Color::ORANGE,
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

fn main() {
    let window_width = 1200;
    let window_height = 900;
    let normal_render_width = 1000;
    let fullscreen_render_width = 1600;
    let block_size = 15;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Mundo 3D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    window.disable_cursor();

    let mut framebuffer = Framebuffer::new(
        normal_render_width,
        normal_render_width * window_height as u32 / window_width as u32,
        Color::BLACK,
    );
    let textures = TextureManager::new();
    let audio = RaylibAudio::init_audio_device().expect("Failed to initialize audio");
    audio.set_audio_stream_buffer_size_default(65_536);
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

    let level_files = ["maps/mapa1.txt", "maps/mapa2.txt", "maps/mapa3.txt"];
    let mut selected_level = 0;
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
    let mut has_key = false;
    let mut exit_unlocked = false;
    let mut projectiles = Vec::new();
    let mut shot_flash = 0.0;

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
            if window.is_key_pressed(KeyboardKey::KEY_UP) {
                selected_level = selected_level.saturating_sub(1);
            }
            if window.is_key_pressed(KeyboardKey::KEY_DOWN) {
                selected_level = (selected_level + 1).min(level_files.len() - 1);
            }

            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                maze = load_maze(level_files[selected_level]);
                player.pos = player_start_position(&maze, block_size);
                player.a = player_start_angle(&maze);
                sprites = spawn_enemies(selected_level, &maze, block_size);
                pickups = spawn_pickups(selected_level, &maze, block_size);
                player_health = 100;
                ammo = 6;
                has_key = false;
                exit_unlocked = false;
                projectiles.clear();
                shot_flash = 0.0;
                welcome_screen = false;
            }

            framebuffer.clear();
            framebuffer.swap_buffers(
                &mut window,
                &raylib_thread,
                welcome_screen,
                selected_level,
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
            framebuffer.swap_buffers(&mut window, &raylib_thread, false, selected_level, true, false);
            continue;
        }

        if defeat_screen {
            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                maze = load_maze(level_files[selected_level]);
                player.pos = player_start_position(&maze, block_size);
                player.a = player_start_angle(&maze);
                sprites = spawn_enemies(selected_level, &maze, block_size);
                pickups = spawn_pickups(selected_level, &maze, block_size);
                player_health = 100;
                ammo = 6;
                has_key = false;
                exit_unlocked = false;
                projectiles.clear();
                shot_flash = 0.0;
                defeat_screen = false;
            } else if window.is_key_pressed(KeyboardKey::KEY_L) {
                welcome_screen = true;
                defeat_screen = false;
            }

            framebuffer.clear();
            framebuffer.swap_buffers(&mut window, &raylib_thread, false, selected_level, false, true);
            continue;
        }

        // 1. move the player on user input
        let delta_time = window.get_frame_time();
        shot_flash = (shot_flash - delta_time).max(0.0);
        let shot_fired = process_events(&window, &mut player, &maze, block_size);
        if collect_pickups(
            &player,
            &mut pickups,
            &mut player_health,
            &mut ammo,
            &mut has_key,
        ) {
            hit_sound.play();
        }

        if window.is_key_pressed(KeyboardKey::KEY_E)
            && interact_with_level(
                &player,
                &mut maze,
                &mut pickups,
                &mut has_key,
                &mut exit_unlocked,
                block_size,
            )
        {
            hit_sound.play();
        }

        if player_reached_goal(&player, &maze, &sprites, exit_unlocked, block_size) {
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
                window.get_time() as f32,
                shot_flash > 0.0,
            );
            render_minimap(&mut framebuffer, &maze, &player, block_size);
            render_hud(
                &mut framebuffer,
                player_health,
                ammo,
                MAX_AMMO,
                sprites.iter().filter(|sprite| sprite.active).count(),
                sprites.len(),
            );
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread, false, selected_level, false, false);
    }
}
