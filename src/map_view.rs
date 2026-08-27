use raylib::prelude::*;

use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::textures::TextureManager;
use crate::Maze;

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

pub fn world_to_map_position(
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

pub fn draw_map_line(framebuffer: &mut Framebuffer, start: Vector2, end: Vector2) {
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

pub fn render_minimap(
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

