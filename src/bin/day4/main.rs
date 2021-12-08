use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day4/input.txt";
    let file = File::open(filename).expect("File not found");
    let reader = BufReader::new(file);

    let game = Game::parse(reader)?;
    if let Some(winning_score) = game.clone().play() {
        println!("Winning score: {}", winning_score);
    } else {
        println!("No board won");
    }

    if let Some(losing_score) = game.play_to_lose() {
        println!("Losing score: {}", losing_score);
    } else {
        println!("No board won (so none lost)");
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Game {
    numbers_drawn: Vec<u32>,
    boards: Vec<Board>,
}

const BOARD_WIDTH: usize = 5;

#[derive(Debug, Clone)]
struct Board {
    drawn: [bool; BOARD_WIDTH * BOARD_WIDTH],
    nums: Vec<u32>,
}

impl Game {
    fn parse(input: impl BufRead) -> Result<Game, &'static str> {
        let mut lines = input.lines();

        let numbers_drawn = parse_integer_line(&mut lines, ',')?;

        let mut boards = vec![];
        loop {
            if lines.next().is_none() {
                break;
            }

            boards.push(Board::parse(&mut lines)?);
        }

        Ok(Game {
            numbers_drawn,
            boards,
        })
    }

    fn play(mut self) -> Option<u64> {
        for drawn in self.numbers_drawn {
            for board in &mut self.boards {
                if board.draw(drawn) {
                    return Some(board.score(drawn));
                }
            }
        }

        None
    }

    fn play_to_lose(mut self) -> Option<u64> {
        let drawn = self.numbers_drawn.clone();

        let mut counted: Vec<_> = self
            .boards
            .iter_mut()
            .map(|board| {
                let win_index = drawn.iter().take_while(|&&num| !board.draw(num)).count();
                (board, win_index)
            })
            .collect();

        counted.sort_by_key(|&(_, win_index)| win_index);
        if let Some((board, win_index)) = counted.last() {
            Some(board.score(drawn[*win_index]))
        } else {
            None
        }
    }
}

impl Board {
    fn parse(lines: &mut Lines<impl BufRead>) -> Result<Board, &'static str> {
        let mut board = Vec::with_capacity(BOARD_WIDTH * BOARD_WIDTH);
        for _ in 0..BOARD_WIDTH {
            let nums = parse_integer_line_ws(lines)?;
            if nums.len() != BOARD_WIDTH {
                return Err("Each row should contain 5 integers");
            }
            board.extend_from_slice(&nums);
        }

        Ok(Board {
            nums: board,
            drawn: [false; BOARD_WIDTH * BOARD_WIDTH],
        })
    }

    /// Updates the board with the given number. Returns true if the board has won.
    fn draw(&mut self, num: u32) -> bool {
        for (i, entry) in self.nums.iter().enumerate() {
            if *entry == num {
                self.drawn[i] = true;
                // We assume no duplicates in each grid
                break;
            }
        }

        let winning_row = self
            .drawn
            .chunks_exact(BOARD_WIDTH)
            .any(|row| row.iter().all(|&x| x));
        if winning_row {
            return true;
        }

        let winning_col = (0..BOARD_WIDTH).any(|col| {
            (col..)
                .step_by(BOARD_WIDTH)
                .take(5)
                .map(|idx| self.drawn[idx])
                .all(|x| x)
        });

        winning_col
    }

    fn score(&self, last_drawn: u32) -> u64 {
        let unmarked_sum: u64 = self
            .nums
            .iter()
            .zip(self.drawn.iter())
            .filter(|(_, &x)| !x)
            .map(|(num, _)| *num as u64)
            .sum();
        unmarked_sum * last_drawn as u64
    }
}

fn parse_integer_line(
    lines: &mut Lines<impl BufRead>,
    sep: char,
) -> Result<Vec<u32>, &'static str> {
    let integers = lines
        .next()
        .ok_or("Expected list of numbers")?
        .map_err(|_| "I/O Error reading line")?
        .split(sep)
        .map(|s| s.parse().map_err(|_| "Expected integer"))
        .collect::<Result<Vec<u32>, _>>()?;
    Ok(integers)
}

fn parse_integer_line_ws(lines: &mut Lines<impl BufRead>) -> Result<Vec<u32>, &'static str> {
    let integers = lines
        .next()
        .ok_or("Expected list of numbers")?
        .map_err(|_| "I/O Error reading line")?
        .split_whitespace()
        .map(|s| s.parse().map_err(|_| "Expected integer"))
        .collect::<Result<Vec<u32>, _>>()?;
    Ok(integers)
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    const TEST_INPUT: &str = "\
7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7
";

    #[test]
    fn test_parse_integer_line() {
        let input = parse_integer_line(&mut io::Cursor::new("3,4,5").lines(), ',');
        assert_eq!(input.unwrap(), vec![3, 4, 5]);
    }

    #[test]
    fn test_parse_integer_line_ws() {
        let input = parse_integer_line_ws(&mut io::Cursor::new("3 4 5").lines());
        assert_eq!(input.unwrap(), vec![3, 4, 5]);

        let input = parse_integer_line_ws(&mut io::Cursor::new(" 3 4  5").lines());
        assert_eq!(input.unwrap(), vec![3, 4, 5]);
    }

    #[test]
    fn test_game() {
        let game = Game::parse(&mut io::Cursor::new(TEST_INPUT)).unwrap();
        assert_eq!(game.play(), Some(4512));
    }

    #[test]
    fn test_game_lose() {
        let game = Game::parse(&mut io::Cursor::new(TEST_INPUT)).unwrap();
        assert_eq!(game.play_to_lose(), Some(1924));
    }
}
