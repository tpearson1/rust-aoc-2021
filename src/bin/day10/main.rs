use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day10/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let illegal_score = score_illegal_lines(&input);
    println!("Syntax error score: {}", illegal_score);

    let incomplete_score = incomplete_lines_middle_score(&input).ok_or("No incomplete lines")?;
    println!("Middle incomplete score: {}", incomplete_score);

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckLineError {
    Incomplete(Vec<Symbol>),
    IllegalChar(Symbol),
    UnknownChar(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Bracket,
    Paren,
    Brace,
    Angle,
}

pub struct LineChecker {
    stack: Vec<Symbol>,
}

impl LineChecker {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(128),
        }
    }

    pub fn check_line(&mut self, line: &str) -> Result<(), CheckLineError> {
        use CheckLineError::*;
        use Symbol::*;

        // Prep for upcoming use
        self.stack.clear();

        for char in line.chars() {
            match char {
                '[' => self.stack.push(Bracket),
                '(' => self.stack.push(Paren),
                '{' => self.stack.push(Brace),
                '<' => self.stack.push(Angle),
                ']' => parse_expect(&mut self.stack, Bracket)?,
                ')' => parse_expect(&mut self.stack, Paren)?,
                '}' => parse_expect(&mut self.stack, Brace)?,
                '>' => parse_expect(&mut self.stack, Angle)?,
                c => return Err(UnknownChar(c)),
            }
        }

        fn parse_expect(stack: &mut Vec<Symbol>, cur: Symbol) -> Result<(), CheckLineError> {
            if let Some(actual) = stack.pop() {
                if actual != cur {
                    Err(IllegalChar(cur))
                } else {
                    Ok(())
                }
            } else {
                Err(IllegalChar(cur))
            }
        }

        if self.stack.len() > 0 {
            let remaining: Vec<_> = self.stack.iter().rev().copied().collect();
            Err(Incomplete(remaining))
        } else {
            Ok(())
        }
    }
}

pub fn score_error(err: CheckLineError) -> u64 {
    use CheckLineError::*;
    use Symbol::*;
    match err {
        Incomplete(_) => 0,
        UnknownChar(_) => 0,
        IllegalChar(c) => match c {
            Bracket => 57,
            Paren => 3,
            Brace => 1197,
            Angle => 25137,
        },
    }
}

pub fn score_completion(completion: &[Symbol]) -> u64 {
    use Symbol::*;
    completion
        .iter()
        .map(|&c| match c {
            Paren => 1,
            Bracket => 2,
            Brace => 3,
            Angle => 4,
        })
        .fold(0, |acc, x| acc * 5 + x)
}

pub fn score_illegal_lines(input: &str) -> u64 {
    let mut checker = LineChecker::new();
    input
        .lines()
        .filter_map(|line| checker.check_line(line).err().map(score_error))
        .sum()
}

pub fn incomplete_lines_middle_score(input: &str) -> Option<u64> {
    let mut checker = LineChecker::new();
    let mut incomplete_scores: Vec<_> = input
        .lines()
        .filter_map(|line| checker.check_line(line).err())
        .filter_map(|err| match err {
            CheckLineError::Incomplete(completion) => Some(score_completion(&completion)),
            _ => None,
        })
        .collect();

    // Middle score, assuming (as instructed) an odd number of incomplete lines
    incomplete_scores.sort_unstable();
    incomplete_scores.get(incomplete_scores.len() / 2).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_line() {
        use CheckLineError::*;
        use Symbol::*;

        let mut checker = LineChecker::new();

        // Wrong bracket pairing
        let result = checker.check_line("{([(<{}[<>[]}>{[]{[(<()>");
        assert_eq!(result, Err(IllegalChar(Brace)));
        let result = checker.check_line("[[<[([]))<([[{}[[()]]]");
        assert_eq!(result, Err(IllegalChar(Paren)));
        let result = checker.check_line("[{[{({}]{}}([{[{{{}}([]");
        assert_eq!(result, Err(IllegalChar(Bracket)));
        let result = checker.check_line("[<(<(<(<{}))><([]([]()");
        assert_eq!(result, Err(IllegalChar(Paren)));
        let result = checker.check_line("<{([([[(<>()){}]>(<<{{");
        assert_eq!(result, Err(IllegalChar(Angle)));

        // Unexpected extra closing bracket(s)
        let result = checker.check_line("[[]]]");
        assert_eq!(result, Err(IllegalChar(Bracket)));
        let result = checker.check_line("<{[]}>)");
        assert_eq!(result, Err(IllegalChar(Paren)));

        // Characters not in expected alphabet
        let result = checker.check_line("[[a");
        assert_eq!(result, Err(UnknownChar('a')));
        let result = checker.check_line("ab");
        assert_eq!(result, Err(UnknownChar('a')));

        // Incomplete (not enough closing)
        let result = checker.check_line("[[[");
        assert_eq!(result, Err(Incomplete(vec![Bracket; 3])));
        let result = checker.check_line("(<>");
        assert_eq!(result, Err(Incomplete(vec![Paren])));

        // Good
        let result = checker.check_line("{()}");
        assert_eq!(result, Ok(()));
        let result = checker.check_line("{([[[()]]{}]{{}})}");
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_score_error() {
        use CheckLineError::*;
        use Symbol::*;

        assert_eq!(score_error(IllegalChar(Bracket)), 57);
        assert_eq!(score_error(IllegalChar(Paren)), 3);
        assert_eq!(score_error(IllegalChar(Brace)), 1197);
        assert_eq!(score_error(IllegalChar(Angle)), 25137);

        assert_eq!(score_error(UnknownChar('a')), 0);
        assert_eq!(score_error(Incomplete(vec![])), 0);
    }

    const TEST_INPUT: &str = "\
[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]
";

    #[test]
    fn test_score_illegal_lines() {
        assert_eq!(score_illegal_lines(TEST_INPUT), 26397);
    }

    #[test]
    fn test_score_completion() {
        use Symbol::*;
        // ])}>
        let score = score_completion(&[Bracket, Paren, Brace, Angle]);
        assert_eq!(score, 294);
    }

    #[test]
    fn test_incomplete_lines_middle_score() {
        assert_eq!(incomplete_lines_middle_score(TEST_INPUT), Some(288957));
    }
}
