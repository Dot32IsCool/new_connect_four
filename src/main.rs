mod board;
use board::*;
use colored::*;
use unicode_segmentation::UnicodeSegmentation;

use std::io::{stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::terminal_size;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Colour {
    None,
    Red,
    Blue,
}

fn main() {
    // For handling Termion and it's input
    let stdout = stdout();
    let mut stdout = stdout
        .lock()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    let stdin = stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();
    let size = terminal_size().unwrap();

    // Initialise the game
    let mut board = Board::new();
    let mut turn = Colour::Red;
    let mut input = 8;
    let mut turns = 0;
    // Game loop
    while turns < 42 { // 42 is the maximum number of turns, as the board is 7x6
        // Refreshes the board, current turn, highlighted column, etc
        redraw_game(&mut stdout, &board, turn);

        // This checks for input and updates the board
        loop {
            let b = bytes.next().unwrap().unwrap();
            match b {
                // Quit on Ctrl+C
                3 => return,
                // Enter or space key
                13 | 32 => if input != 8 {break;}
                // 1-7
                49..=55 => {
                    input = (b - 49) as usize;
                    board.highlighted_column = Some(input);
                    redraw_game(&mut stdout, &board, turn);
                }
                // Left arrow
                68 => {
                    match board.highlighted_column {
                        Some(column) => {
                            if column > 0 {
                                input = column - 1;
                                board.highlighted_column = Some(column - 1);
                            }
                        }
                        None => (),
                    }
                    redraw_game(&mut stdout, &board, turn);
                }
                // Right arrow
                67 => {
                    match board.highlighted_column {
                        Some(column) => {
                            if column < 6 {
                                input = column + 1;
                                board.highlighted_column = Some(column + 1);
                            }
                        }
                        None => (),
                    }
                    redraw_game(&mut stdout, &board, turn);
                }
                // Deselect board if random key pressed
                _ => {
                    board.highlighted_column = None;
                    redraw_game(&mut stdout, &board, turn);
                }
            }

            stdout.flush().unwrap();
        }
        match board.drop_piece(input, turn) {
            Ok(y) => {
                // Animate the piece falling, iterating from 0 to the Y value returned from drop_piece
                for frame in 0..y {
                    redraw_game(&mut stdout, &board.animation_frame(frame), turn);
                    std::thread::sleep(std::time::Duration::from_millis(15));
                }
            }
            // If the column is full, just continue the game loop, and don't change turns
            Err(_) => continue,
        }
        match board.check_win_at(input) {
            Some(colour) => {
                // Clears the screen and hides the cursor
                centred_print(
                    &mut stdout,
                    &format!("{}{}", termion::clear::All, termion::cursor::Hide),
                    None,
                    1,
                );

                centred_print(
                    &mut stdout,
                    &format!(
                        "{} wins!",
                        if colour == Colour::Red {
                            "Red".red()
                        } else {
                            "Blue".blue()
                        }
                    ),
                    Some(4),
                    size.1 / 2 - 6,
                );
                centred_print(
                    &mut stdout,
                    &format!("{}", board.to_string()),
                    Some(11),
                    size.1 / 2 - 4,
                );
                centred_print(
                    &mut stdout,
                    &format!(
                        "{}{}",
                        "Press any key to quit.".dimmed(),
                        termion::cursor::Goto(0, size.1)
                    ),
                    Some(10),
                    size.1 / 2 + 5,
                );
                let _ = bytes.next();
                return;
            }
            _ => (),
        }
        turn = if turn == Colour::Red {
            Colour::Blue
        } else {
            Colour::Red
        }; // Switch turns
        turns += 1;
    }
    // Draw condition
    // Only runs if the game loop ends without a winner
    centred_print(&mut stdout, &format!("{}{}", termion::clear::All, termion::cursor::Hide), None, 1);
    
    centred_print(
        &mut stdout,
        &format!(
            "{}{}",
            "It's a draw!".dimmed(),
            termion::cursor::Goto(0, size.1)
        ),
        Some(6),
        size.1 / 2 - 6,
    );
    centred_print(
        &mut stdout,
        &format!("{}", board.to_string()),
        Some(11),
        size.1 / 2 - 4,
    );
    centred_print(
        &mut stdout,
        &format!(
            "{}{}",
            "Press any key to quit.".dimmed(),
            termion::cursor::Goto(0, size.1)
        ),
        Some(10),
        size.1 / 2 + 5,
    );
    let _ = bytes.next();
}

fn centred_print(
    stdout: &mut termion::raw::RawTerminal<std::io::StdoutLock>,
    strings: &str,
    x: Option<u16>,
    y: u16,
) {
    let size = terminal_size().unwrap();

    // Splits the string by newlines and prints each line centred.
    for (i, string) in strings.lines().enumerate() {
        write!(
            stdout,
            "{}{}",
            // If an x value is provided, use that, otherwise, centre the text
            // This is because using colored's methods add a number of invisable characters, making centering invalid.
            // This is a workaround for that.
            match x {
                Some(x) => termion::cursor::Goto(size.0 / 2 - x, y + i as u16),
                None => termion::cursor::Goto(
                    size.0 / 2 - (string.graphemes(true).count() / 2) as u16,
                    y + i as u16
                ),
            },
            string,
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}

fn redraw_game(
    stdout: &mut termion::raw::RawTerminal<std::io::StdoutLock>,
    board: &Board,
    turn: Colour,
) {
    let size = terminal_size().unwrap();

    // Clears the screen and hides the cursor
    centred_print(stdout, &format!("{}{}", termion::clear::All, termion::cursor::Hide), None, 1);

    centred_print(
        stdout,
        &format!(
            "It's {}'s turn!",
            if turn == Colour::Red {
                "Red".red()
            } else {
                "Blue".blue()
            }
        ),
        Some(8),
        size.1 / 2 - 6,
    );
    centred_print(
        stdout,
        &format!("{}", board.to_string()),
        Some(11),
        size.1 / 2 - 4,
    );

    // If a column is selected, print the number of the column
    centred_print(
        stdout,
        &format!(
            "Select column: {}",
            match board.highlighted_column {
                Some(column) => format!("{}", (column + 1).to_string().yellow()),
                None => "".to_string(),
            }
        ),
        Some(8),
        size.1 / 2 + 5,
    );
    // If a column is selected, print the "Press enter to confirm" message
    match board.highlighted_column {
        Some(_) => centred_print(
            stdout,
            &format!("{}", "Press enter to confirm."),
            None,
            size.1 / 2 + 6,
        ),
        None => (),
    }
    centred_print(
        stdout,
        &format!(
            "{}{}",
            "Press Ctrl+C to quit at any time.".dimmed(),
            termion::cursor::Goto(0, size.1)
        ),
        Some(16),
        size.1 / 2 + 8,
    );
}
