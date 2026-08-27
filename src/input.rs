use raylib::prelude::*;
use std::f32::consts::PI;

use crate::player::Player;
use crate::Maze;

pub fn process_events(
    window: &RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
) -> bool {
    const MOVE_SPEED: f32 = 2.0;
    const ROTATION_SPEED: f32 = PI / 90.0;
    const MOUSE_SENSITIVITY: f32 = 0.003;

    player.a += window.get_mouse_delta().x * MOUSE_SENSITIVITY;

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
        move_player(player, MOVE_SPEED, maze, block_size);
    }

    if window.is_key_down(KeyboardKey::KEY_DOWN) {
        // decrease player position in x and y in the direction of view
        move_player(player, -MOVE_SPEED, maze, block_size);
    }

    window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
}

fn move_player(player: &mut Player, distance: f32, maze: &Maze, block_size: usize) {
    let next_x = player.pos.x + distance * player.a.cos();
    let next_y = player.pos.y + distance * player.a.sin();

    if next_x < 0.0 || next_y < 0.0 {
        return;
    }

    let column = next_x as usize / block_size;
    let row = next_y as usize / block_size;

    if row < maze.len()
        && column < maze[row].len()
        && matches!(maze[row][column], ' ' | 'g')
    {
        player.pos.x = next_x;
        player.pos.y = next_y;
    }
}
