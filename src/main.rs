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

type Maze = Vec<Vec<char>>;

let maze = load_maze("./maze.txt");

fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    // pinten un rectangulo de diferente color segun cada char

    let color = match cell {
        '+' | '-' | '|' => Color::BLACK,
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
}D