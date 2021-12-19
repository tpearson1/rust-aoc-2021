use std::{collections::HashSet, fmt::Display, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day13/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let mut paper = Paper::parse_from_str(&input).ok_or("Failed to parse input")?;
    paper.apply_fold();
    println!("Points remaining after first fold: {}", paper.num_points());

    paper.apply_folds();
    println!("Points remaining after all folds: {}", paper.num_points());
    println!("After all folds:\n{}", paper);

    Ok(())
}

pub struct Paper {
    points: Vec<(i32, i32)>,
    folds: Vec<Fold>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Fold {
    Left(i32),
    Up(i32),
}

impl Paper {
    pub fn parse_from_str(input: &str) -> Option<Paper> {
        let mut lines = input.lines();

        let points = lines
            .by_ref()
            .take_while(|line| !line.is_empty())
            .map(|line| {
                let (x, y) = line.split_once(',')?;
                Some((x.parse().ok()?, y.parse().ok()?))
            })
            .collect::<Option<Vec<(i32, i32)>>>()?;

        let folds = lines
            .rev()
            .map(|line| {
                let (instruction, coord) = line.split_once('=')?;
                match instruction {
                    "fold along x" => Some(Fold::Left(coord.parse().ok()?)),
                    "fold along y" => Some(Fold::Up(coord.parse().ok()?)),
                    _ => None,
                }
            })
            .collect::<Option<Vec<Fold>>>()?;

        Some(Self { points, folds })
    }

    pub fn apply_folds(&mut self) {
        loop {
            if self.apply_fold().is_none() {
                break;
            }
        }
    }

    pub fn apply_fold(&mut self) -> Option<Fold> {
        let fold = self.folds.pop()?;

        self.points = match &fold {
            Fold::Left(foldx) => self
                .points
                .iter()
                .map(|&(x, y)| {
                    if x > *foldx {
                        (foldx - (x - foldx), y)
                    } else {
                        (x, y)
                    }
                })
                .collect(),
            Fold::Up(foldy) => self
                .points
                .iter()
                .map(|&(x, y)| {
                    if y > *foldy {
                        (x, foldy - (y - foldy))
                    } else {
                        (x, y)
                    }
                })
                .collect(),
        };

        // Could call this after multiple folds if performance is a concern.
        // Could even keep a tally of number of points that actually get moved
        // in folds and when it exceeds a certain value, perform this. For the
        // day's input it runs very fast regardless
        self.points.sort_unstable();
        self.points.dedup();

        Some(fold)
    }

    pub fn num_points(&self) -> usize {
        self.points.len()
    }
}

impl Display for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let maxx = self.points.iter().map(|&(x, _)| x).max().unwrap_or(0);
        let maxy = self.points.iter().map(|&(_, y)| y).max().unwrap_or(0);

        let points: HashSet<_> = self.points.iter().copied().collect();
        for y in 0..=maxy {
            for x in 0..=maxx {
                if points.contains(&(x, y)) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5
";

    #[test]
    fn test_parse_input() {
        let paper = Paper::parse_from_str(EXAMPLE_INPUT).unwrap();

        assert_eq!(paper.points.len(), 18);
        assert_eq!(paper.points[0], (6, 10));
        assert_eq!(paper.points[1], (0, 14));
        assert_eq!(paper.points[17], (9, 0));

        // Reversed (as a stack)
        assert_eq!(paper.folds, vec![Fold::Left(5), Fold::Up(7)]);
    }

    #[test]
    fn test_apply_fold() {
        let mut paper = Paper::parse_from_str(EXAMPLE_INPUT).unwrap();

        let fold = paper.apply_fold().unwrap();
        assert_eq!(fold, Fold::Up(7));
        assert_eq!(paper.num_points(), 17);

        let fold = paper.apply_fold().unwrap();
        assert_eq!(fold, Fold::Left(5));
        assert_eq!(paper.num_points(), 16);

        assert_eq!(paper.apply_fold(), None);
        assert_eq!(paper.num_points(), 16);
    }

    #[test]
    fn test_apply_folds() {
        let mut paper = Paper::parse_from_str(EXAMPLE_INPUT).unwrap();
        paper.apply_folds();
        assert_eq!(paper.num_points(), 16);
    }

    #[test]
    fn test_display() {
        let mut paper = Paper::parse_from_str(EXAMPLE_INPUT).unwrap();
        paper.apply_folds();

        assert_eq!(
            paper.to_string(),
            "\
#####
#...#
#...#
#...#
#####
"
        );
    }
}
