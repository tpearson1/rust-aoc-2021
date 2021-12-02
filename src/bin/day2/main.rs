use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(PartialEq, Eq, Debug, Clone)]
enum Action {
    Forward(i64),
    Down(i64),
    Up(i64),
}

#[derive(Debug)]
struct Position {
    horizontal: i64,
    depth: i64,
    aim: i64,
}

impl Position {
    fn new() -> Self {
        Self {
            horizontal: 0,
            depth: 0,
            aim: 0,
        }
    }

    fn apply_action_naive(&mut self, action: &Action) {
        match action {
            Action::Forward(distance) => self.horizontal += distance,
            Action::Down(distance) => self.depth += distance,
            Action::Up(distance) => self.depth -= distance,
        }
    }

    fn apply_action(&mut self, action: &Action) {
        match action {
            Action::Forward(distance) => {
                self.horizontal += distance;
                self.depth += self.aim * distance;
            }
            Action::Down(amount) => self.aim += amount,
            Action::Up(amount) => self.aim -= amount,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Position:")?;
        writeln!(f, "  Horizontal Position: {}", self.horizontal)?;
        writeln!(f, "  Depth: {}", self.depth)?;
        write!(f, "  Product of above: {}", self.horizontal * self.depth)?;
        Ok(())
    }
}

fn parse_line(line: &str) -> Result<Action, &'static str> {
    use Action::*;
    let (action, count) = line
        .split_once(' ')
        .ok_or("Expected space delimiter on line")?;
    let count = count.parse().map_err(|_| "Invalid count")?;
    Ok(match action {
        "forward" => Forward(count),
        "down" => Down(count),
        "up" => Up(count),
        _ => return Err("Invalid action"),
    })
}

fn parse_input(input: impl BufRead) -> Result<Vec<Action>, Box<dyn Error>> {
    let actions = input
        .lines()
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|l| parse_line(l))
        .collect::<Result<Vec<_>, _>>();
    Ok(actions?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "src/bin/day2/input.txt";
    let file = File::open(filename).expect("File not found");
    let reader = BufReader::new(file);

    let actions = parse_input(reader)?;

    let mut position = Position::new();
    for action in actions.iter() {
        position.apply_action_naive(action);
    }

    println!("{}", position);

    let mut position = Position::new();
    for action in actions.iter() {
        position.apply_action(action);
    }

    println!("\n{}", position);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        assert_eq!(parse_line("forward 5"), Ok(Action::Forward(5)));
        assert_eq!(parse_line("down 7"), Ok(Action::Down(7)));
        assert_eq!(parse_line("up 0"), Ok(Action::Up(0)));
        assert_eq!(parse_line("up hi"), Err("Invalid count"));
        assert_eq!(parse_line("yes 3"), Err("Invalid action"));
        assert_eq!(
            parse_line("something"),
            Err("Expected space delimiter on line")
        );
    }

    mod test_parse_input {
        use super::*;

        use std::io;

        #[test]
        fn happy_path() {
            let input = b"forward 7\ndown 5\nup 3\n";
            let cursor = io::Cursor::new(input);
            let expected = [Action::Forward(7), Action::Down(5), Action::Up(3)];
            let result = parse_input(cursor).expect("Should succeed");
            assert_eq!(result, expected);
        }

        #[test]
        fn invalid_syntax() {
            let input = b"forward 7\nthisisnotright\nup 3";
            let cursor = io::Cursor::new(input);
            let result = parse_input(cursor);
            assert_eq!(
                result.expect_err("Should fail").to_string(),
                "Expected space delimiter on line"
            );
        }
    }

    #[test]
    fn test_apply_action_naive() {
        let mut position = Position::new();
        position.apply_action_naive(&Action::Down(7)); // depth -> 7
        position.apply_action_naive(&Action::Up(3)); // depth -> 4
        position.apply_action_naive(&Action::Forward(8)); // horizontal -> 8
        assert_eq!(position.depth, 4);
        assert_eq!(position.horizontal, 8);
        assert_eq!(position.aim, 0);
    }

    #[test]
    fn test_apply_action() {
        let mut position = Position::new();
        position.apply_action(&Action::Down(7)); // aim -> 7
        position.apply_action(&Action::Up(3)); // aim -> 4
        position.apply_action(&Action::Forward(8)); // horizontal -> 8, depth -> 4 * 8 = 32
        assert_eq!(position.depth, 32);
        assert_eq!(position.horizontal, 8);
        assert_eq!(position.aim, 4);
    }
}
