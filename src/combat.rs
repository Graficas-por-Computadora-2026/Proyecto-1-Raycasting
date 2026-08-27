use raylib::prelude::*;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::level::{cell_position, reachable_cells};
use crate::player::Player;
use crate::sprites::{EnemyKind, Sprite};
use crate::Maze;

pub const MAX_AMMO: i32 = 12;

#[derive(Clone, Copy)]
pub enum PickupKind {
    Health,
    Ammo,
    Switch,
}

pub struct Pickup {
    pub pos: Vector2,
    pub kind: PickupKind,
    pub active: bool,
}

pub struct Projectile {
    pub pos: Vector2,
    pub direction: Vector2,
    pub active: bool,
}

fn spawn_enemy(position: Vector2, kind: EnemyKind, block_size: usize) -> Sprite {
    let health = match kind {
        EnemyKind::Grunt => 3,
        EnemyKind::Brute => 5,
    };

    Sprite {
        pos: position,
        texture: match kind {
            EnemyKind::Grunt => 'g',
            EnemyKind::Brute => 'b',
        },
        size: block_size as f32 * 1.25,
        active: true,
        health,
        attack_cooldown: 0.0,
        kind,
    }
}

pub fn spawn_enemies(level: usize, maze: &Maze, block_size: usize) -> Vec<Sprite> {
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

pub fn spawn_pickups(level: usize, maze: &Maze, block_size: usize) -> Vec<Pickup> {
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
            (Vector2::new(175.0, 75.0), PickupKind::Ammo),
            (Vector2::new(350.0, 75.0), PickupKind::Health),
            (Vector2::new(425.0, 75.0), PickupKind::Switch),
        ],
        1 => vec![
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

pub fn collect_pickups(
    player: &Player,
    pickups: &mut [Pickup],
    health: &mut i32,
    ammo: &mut i32,
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
            PickupKind::Switch => continue,
        }
        pickup.active = false;
        collected = true;
    }

    collected
}

pub fn interact_with_level(
    player: &Player,
    maze: &mut Maze,
    pickups: &mut [Pickup],
    exit_unlocked: &mut bool,
    block_size: usize,
) -> bool {
    const INTERACTION_DISTANCE: f32 = 40.0;

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

pub fn update_enemies(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    sprites: &mut [Sprite],
    projectiles: &mut Vec<Projectile>,
    block_size: usize,
    delta_time: f32,
) -> usize {
    const SIGHT_RANGE: f32 = 1_200.0;
    const ATTACK_RANGE: f32 = 75.0;
    let mut shots_fired = 0;

    for sprite in sprites {
        if !sprite.active {
            continue;
        }

        sprite.attack_cooldown = (sprite.attack_cooldown - delta_time).max(0.0);
        let dx = player.pos.x - sprite.pos.x;
        let dy = player.pos.y - sprite.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance < f32::EPSILON || distance > SIGHT_RANGE {
            continue;
        }
        let angle_from_player = (sprite.pos.y - player.pos.y).atan2(sprite.pos.x - player.pos.x);
        let wall = cast_ray(
            framebuffer,
            maze,
            player,
            angle_from_player,
            block_size,
            false,
        );

        if distance >= wall.distance {
            continue;
        }

        let (speed, attack_delay, projectile_speed) = match sprite.kind {
            EnemyKind::Grunt => (70.0, 1.2, 90.0),
            EnemyKind::Brute => (50.0, 1.8, 70.0),
        };

        if distance <= ATTACK_RANGE && sprite.attack_cooldown == 0.0 {
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

        if distance > 1.0 {
            let step = (speed * delta_time).min(distance - 1.0);
            let next_position = Vector2::new(
                sprite.pos.x + dx / distance * step,
                sprite.pos.y + dy / distance * step,
            );

            if enemy_can_move(next_position, maze, block_size) {
                sprite.pos = next_position;
            }
        }
    }

    shots_fired
}

pub fn update_projectiles(
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
