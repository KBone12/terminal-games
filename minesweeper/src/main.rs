use clap::{crate_authors, crate_description, crate_name, crate_version, Arg};
use rand::{
    self,
    distributions::{Bernoulli, Distribution},
};

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Empty { count: i8, hidden: bool },
    Bomb { hidden: bool },
}

impl CellState {
    pub fn is_empty(&self) -> bool {
        matches!(self, CellState::Empty { .. })
    }
    pub fn is_bomb(&self) -> bool {
        matches!(self, CellState::Bomb { .. })
    }
}

fn generate_board(width: usize, height: usize, rate: f64, max_bombs: usize) -> Vec<Vec<CellState>> {
    let rng = rand::thread_rng();
    let mut samples = Bernoulli::new(rate).unwrap().sample_iter(rng);
    let mut bombs = 0;
    let mut board = vec![
        vec![
            CellState::Empty {
                count: 0,
                hidden: true
            };
            width
        ];
        height
    ];

    while bombs < max_bombs {
        for y in 0..height {
            for x in 0..width {
                if board[y][x].is_bomb() {
                    continue;
                }
                if bombs < max_bombs && samples.next().unwrap() {
                    board[y][x] = CellState::Bomb { hidden: true };
                    bombs += 1;
                }
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            if board[y][x].is_bomb() {
                continue;
            }
            let mut tmp = 0;
            let (x, y, width, height) = (x as isize, y as isize, width as isize, height as isize);
            for i in -1..=1 {
                for j in -1..=1 {
                    if i == 0 && j == 0 {
                        continue;
                    }
                    if y + i >= 0
                        && y + i < height
                        && x + j >= 0
                        && x + j < width
                        && board[(y + i) as usize][(x + j) as usize].is_bomb()
                    {
                        tmp += 1;
                    }
                }
            }
            let x = x as usize;
            let y = y as usize;
            match &mut board[y][x] {
                CellState::Empty { count, .. } => *count = tmp,
                _ => {}
            }
        }
    }

    board
}

fn print_board<R: AsRef<[CellState]>>(board: &[R]) {
    board.iter().for_each(|row| {
        row.as_ref().iter().for_each(|cell| match cell {
            CellState::Empty { count, hidden } => {
                /*
                if *hidden {
                    print!("*");
                } else {
                    print!("{}", count);
                }
                */
                print!("{}", count);
            }
            CellState::Bomb { hidden } => {
                /*
                if *hidden {
                    print!("*");
                } else {
                    print!("X");
                }
                */
                print!("X");
            }
        });
        println!("");
    });
}

fn main() {
    let arguments = clap::app_from_crate!()
        .arg(
            Arg::with_name("width")
                .long("width")
                .help("the board's width")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .long("height")
                .help("the board's height")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bombs")
                .long("bombs")
                .help("the number of bombs")
                .takes_value(true),
        )
        .get_matches();

    let width = arguments
        .value_of("width")
        .and_then(|width| width.parse::<usize>().ok())
        .unwrap_or(8);
    let width = width.max(1);
    let height = arguments
        .value_of("height")
        .and_then(|height| height.parse::<usize>().ok())
        .unwrap_or(8);
    let height = height.max(1);
    let max_bombs = arguments
        .value_of("bombs")
        .and_then(|bombs| bombs.parse::<usize>().ok())
        .unwrap_or(4);
    let max_bombs = max_bombs.min(width * height - 1);
    let bomb_rate = max_bombs as f64 / (width * height) as f64;
    let board = generate_board(width, height, bomb_rate, max_bombs);

    print_board(&board);
}
