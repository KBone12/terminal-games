use rand;

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Empty { count: i8, hidden: bool },
    Bomb { hidden: bool },
}

fn main() {
    let width = 8;
    let height = 8;
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
    let max_bombs = 4;
    let bomb_rate = max_bombs as f64 / (width * height) as f64;
    let mut bomb_count = 0;
    while bomb_count < max_bombs {
        for y in 0..height {
            for x in 0..width {
                if matches!(board[y][x], CellState::Bomb { .. }) {
                    continue;
                }
                if bomb_count < max_bombs && rand::random::<f64>() < bomb_rate {
                    board[y][x] = CellState::Bomb { hidden: true };
                    bomb_count += 1;
                }
            }
        }
    }
    for y in 0..height {
        for x in 0..width {
            if matches!(board[y][x], CellState::Bomb { .. }) {
                continue;
            }
            let mut tmp = 0;
            let x = x as isize;
            let y = y as isize;
            for i in -1..=1 {
                for j in -1..=1 {
                    if i == 0 && j == 0 {
                        continue;
                    }
                    if y + i >= 0
                        && y + i < height as isize
                        && x + j >= 0
                        && x + j < width as isize
                        && matches!(board[(y + i) as usize][(x + j) as usize], CellState::Bomb { .. })
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

    board.iter().for_each(|row| {
        row.iter().for_each(|cell| match cell {
            CellState::Empty { count, .. } => {
                print!("{}", count);
            }
            CellState::Bomb { .. } => {
                print!("X");
            }
        });
        println!("");
    });
}
