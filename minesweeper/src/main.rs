use std::io::{stdin, stdout, Write};

use clap::{crate_authors, crate_description, crate_name, crate_version, Arg};
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use termion::{
    self,
    cursor::{Goto, SteadyBlock},
    event::{Event, Key, MouseButton, MouseEvent},
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::AlternateScreen,
};

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Empty { count: u8, hidden: bool },
    Bomb { hidden: bool },
}

impl CellState {
    pub fn open(&mut self) -> bool {
        match self {
            CellState::Empty { hidden, .. } => {
                *hidden = false;
                false
            }
            CellState::Bomb { hidden } => {
                *hidden = false;
                true
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, CellState::Empty { .. })
    }
    pub fn is_bomb(&self) -> bool {
        matches!(self, CellState::Bomb { .. })
    }
}

fn generate_board(width: usize, height: usize, max_bombs: usize) -> Vec<Vec<CellState>> {
    let rng = rand::thread_rng();
    let mut y_samples = Uniform::new(0, height).sample_iter(rng);
    let mut x_samples = Uniform::new(0, width).sample_iter(rng);
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
        let y = y_samples.next().unwrap();
        let x = x_samples.next().unwrap();
        if board[y][x].is_empty() {
            board[y][x] = CellState::Bomb { hidden: true };
            bombs += 1;
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

fn draw<W: Write, R: AsRef<[CellState]>>(out: &mut W, board: &[R], (x0, y0): (u16, u16)) {
    board.iter().enumerate().for_each(|(y, row)| {
        let line: String = row
            .as_ref()
            .iter()
            .map(|cell| match cell {
                CellState::Empty { count, hidden } => {
                    if *hidden {
                        '*'
                    } else {
                        if *count == 0 {
                            '.'
                        } else {
                            (count + '0' as u8) as char
                        }
                    }
                }
                CellState::Bomb { hidden } => {
                    if *hidden {
                        '*'
                    } else {
                        'X'
                    }
                }
            })
            .collect();
        write!(out, "{}{}", Goto(x0, y0 + y as u16), line).unwrap();
    });
    out.flush().unwrap();
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

    let (width, height) = {
        let width = arguments
            .value_of("width")
            .and_then(|width| width.parse::<usize>().ok());
        let height = arguments
            .value_of("height")
            .and_then(|height| height.parse::<usize>().ok());
        let (tw, th) = termion::terminal_size().unwrap();
        let (tw, th) = (tw as usize, th as usize);
        if tw < 3 && th < 3 {
            panic!("The terminal is too small!");
        }
        (
            width.unwrap_or(tw - 2).max(1).min(tw - 2),
            height.unwrap_or(th - 2).max(1).min(th - 2),
        )
    };
    let max_bombs = arguments
        .value_of("bombs")
        .and_then(|bombs| bombs.parse::<usize>().ok())
        .unwrap_or(width * height / 10);
    let max_bombs = max_bombs.min(width * height - 1);
    let mut board = generate_board(width, height, max_bombs);

    let mut screen = {
        let screen = AlternateScreen::from(stdout());
        let screen = MouseTerminal::from(screen);
        screen.into_raw_mode().unwrap()
    };
    writeln!(&mut screen, "{}", SteadyBlock).unwrap();

    let offset = {
        let (tw, th) = termion::terminal_size().unwrap();
        ((tw - width as u16) / 2 + 1, (th - height as u16) / 2 + 1)
    };
    let mut cursor_position = offset;
    let mut remaining = width * height - max_bombs;

    let mut events = stdin()
        .events()
        .take_while(|event| event.is_ok())
        .filter_map(|event| event.ok());
    'game_loop: loop {
        draw(&mut screen, &board, offset);
        write!(
            &mut screen,
            "{}",
            Goto(cursor_position.0, cursor_position.1)
        )
        .unwrap();
        screen.flush().unwrap();

        let mut bomb = false;
        let event = match events.next() {
            Some(event) => event,
            None => break 'game_loop,
        };
        match event {
            Event::Key(Key::Esc) => break 'game_loop,
            Event::Key(key) => match key {
                Key::Char('h') => {
                    if cursor_position.0 >= offset.0 + 1 {
                        cursor_position.0 -= 1;
                    }
                }
                Key::Char('j') => {
                    if cursor_position.1 + 1 < offset.1 + height as u16 {
                        cursor_position.1 += 1;
                    }
                }
                Key::Char('k') => {
                    if cursor_position.1 >= offset.1 + 1 {
                        cursor_position.1 -= 1;
                    }
                }
                Key::Char('l') => {
                    if cursor_position.0 + 1 < offset.0 + width as u16 {
                        cursor_position.0 += 1;
                    }
                }
                Key::Char('y') => {
                    if cursor_position.0 >= offset.0 + 1 && cursor_position.1 >= offset.1 + 1 {
                        cursor_position.0 -= 1;
                        cursor_position.1 -= 1;
                    }
                }
                Key::Char('u') => {
                    if cursor_position.0 + 1 < offset.0 + width as u16
                        && cursor_position.1 >= offset.1 + 1
                    {
                        cursor_position.0 += 1;
                        cursor_position.1 -= 1;
                    }
                }
                Key::Char('b') => {
                    if cursor_position.0 >= offset.0 + 1
                        && cursor_position.1 + 1 < offset.1 + height as u16
                    {
                        cursor_position.0 -= 1;
                        cursor_position.1 += 1;
                    }
                }
                Key::Char('n') => {
                    if cursor_position.0 + 1 < offset.0 + width as u16
                        && cursor_position.1 + 1 < offset.1 + height as u16
                    {
                        cursor_position.0 += 1;
                        cursor_position.1 += 1;
                    }
                }
                Key::Char('\n') | Key::Char(' ') => {
                    bomb = board[(cursor_position.1 - offset.1) as usize]
                        [(cursor_position.0 - offset.0) as usize]
                        .open();
                    if !bomb {
                        remaining -= 1;
                    }
                }
                _ => {}
            },
            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                if x >= offset.0
                    && x < offset.0 + width as u16
                    && y >= offset.1
                    && y < offset.1 + height as u16
                {
                    cursor_position = (x, y);
                    bomb = board[(y - offset.1) as usize][(x - offset.0) as usize].open();
                    if !bomb {
                        remaining -= 1;
                    }
                }
            }
            _ => {}
        }
        if bomb {
            board.iter_mut().for_each(|row| {
                row.iter_mut().for_each(|cell| {
                    cell.open();
                });
            });
            draw(&mut screen, &board, offset);

            let messages = [
                "                 ",
                "      Bomb!      ",
                " [Press any key] ",
                "                 ",
            ];
            let (tw, th) = termion::terminal_size().unwrap();
            let x0 = (tw - messages[0].len() as u16) / 2 + 1;
            let y0 = (th - messages.len() as u16) / 2 + 1;
            write!(&mut screen, "{}", Goto(x0, y0)).unwrap();
            messages.iter().enumerate().for_each(|(y, message)| {
                writeln!(&mut screen, "{}{}", Goto(x0, y0 + y as u16), message).unwrap();
            });
            write!(
                &mut screen,
                "{}",
                Goto(cursor_position.0, cursor_position.1)
            )
            .unwrap();
            screen.flush().unwrap();
            stdin().keys().next();
            break 'game_loop;
        }
        if remaining == 0 {
            board.iter_mut().for_each(|row| {
                row.iter_mut().for_each(|cell| {
                    cell.open();
                });
            });
            draw(&mut screen, &board, offset);

            let messages = [
                "                 ",
                "     Clear!      ",
                " [Press any key] ",
                "                 ",
            ];
            let (tw, th) = termion::terminal_size().unwrap();
            let x0 = (tw - messages[0].len() as u16) / 2 + 1;
            let y0 = (th - messages.len() as u16) / 2 + 1;
            write!(&mut screen, "{}", Goto(x0, y0)).unwrap();
            messages.iter().enumerate().for_each(|(y, message)| {
                writeln!(&mut screen, "{}{}", Goto(x0, y0 + y as u16), message).unwrap();
            });
            write!(
                &mut screen,
                "{}",
                Goto(cursor_position.0, cursor_position.1)
            )
            .unwrap();
            screen.flush().unwrap();
            stdin().keys().next();
            break 'game_loop;
        }
    }
}
