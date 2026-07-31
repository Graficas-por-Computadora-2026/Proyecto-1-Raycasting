// player.rs

use raylib::prelude::*;

pub struct Player {
    pub pos: Vector2,
    pub a: f32, // angle of view
    pub fov: f32, // field of view
}
