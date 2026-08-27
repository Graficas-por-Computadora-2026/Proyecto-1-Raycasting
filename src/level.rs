use raylib::prelude::*;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

pub fn is_walkable_cell(cell: char) -> bool {
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

pub fn player_start_position(maze: &Maze, block_size: usize) -> Vector2 {
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

pub fn player_start_angle(maze: &Maze) -> f32 {
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

pub fn reachable_cells(maze: &Maze) -> Vec<(usize, usize)> {
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

pub fn cell_position(cell: (usize, usize), block_size: usize) -> Vector2 {
    Vector2::new(
        (cell.0 as f32 + 0.5) * block_size as f32,
        (cell.1 as f32 + 0.5) * block_size as f32,
    )
}

