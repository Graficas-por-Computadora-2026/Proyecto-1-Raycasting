use raylib::prelude::*;
use std::f32::consts::PI;

use crate::player::Player;

pub fn process_events(window: &RaylibHandle, player: &mut Player) {
    const MOVE_SPEED: f32 = 10.0;
    const ROTATION_SPEED: f32 = PI / 10.0;

    if window.is_key_down(KeyboardKey::KEY_LEFT) {
        // rotate the view range to the left
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(KeyboardKey::KEY_RIGHT) {
        // rotate the view range to the right
        player.a += ROTATION_SPEED;
    }

    if window.is_key_down(KeyboardKey::KEY_UP) {
        // increase player position in x and y in the direction of view
        player.pos.x += MOVE_SPEED * player.a.cos();
        player.pos.y += MOVE_SPEED * player.a.sin();
    }

    if window.is_key_down(KeyboardKey::KEY_DOWN) {
        // decrease player position in x and y in the direction of view
        player.pos.x -= MOVE_SPEED * player.a.cos();
        player.pos.y -= MOVE_SPEED * player.a.sin();
    }
}
