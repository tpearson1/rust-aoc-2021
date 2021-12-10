use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day5/input.txt";
    let file = File::open(filename).map_err(|_| "File not found")?;
    let reader = BufReader::new(file);

    let lines = parse_lines(reader).ok_or("Failed to read input")?;

    let nondiagonals: Vec<_> = lines
        .iter()
        .filter(|l| l.kind() != LineKind::Diagonal)
        .cloned()
        .collect();
    let intersections = Grid::from(&nondiagonals)
        .ok_or("No non-diagonal lines")?
        .count_intersections();
    println!(
        "Number of intersections (nondiagonals only): {}",
        intersections,
    );

    let intersections = Grid::from(&lines).ok_or("No lines")?.count_intersections();
    println!("Number of intersections (all lines): {}", intersections);

    Ok(())
}

struct Grid {
    #[cfg(test)]
    left: u32,
    #[cfg(test)]
    width: u32,
    #[cfg(test)]
    top: u32,
    counts: Vec<usize>,
}

impl Grid {
    fn from(lines: &Vec<Line>) -> Option<Self> {
        let left = lines.iter().flat_map(|x| [x.x1, x.x2]).min()?;
        let right = lines.iter().flat_map(|x| [x.x1, x.x2]).max()?;
        let top = lines.iter().flat_map(|x| [x.y1, x.y2]).min()?;
        let bottom = lines.iter().flat_map(|x| [x.y1, x.y2]).max()?;

        let width = right - left + 1;
        let height = bottom - top + 1;

        let size = width as usize * height as usize;
        let mut counts = vec![0; size];
        for line in lines {
            line.map_points(|(x, y)| {
                let idx = (y - top) * width + (x - left);
                counts[idx as usize] += 1;
            });
        }

        Some(Self {
            #[cfg(test)]
            left,
            #[cfg(test)]
            width,
            #[cfg(test)]
            top,
            counts,
        })
    }

    #[cfg(test)]
    fn intersections(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        let width = self.width as usize;
        let left = self.left;
        let top = self.top;

        self.counts
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, count)| count > 1)
            .map(move |(idx, _)| ((idx % width) as u32 + left, (idx / width) as u32 + top))
    }

    fn count_intersections(&self) -> usize {
        self.counts.iter().filter(|&&c| c > 1).count()
    }
}

// NOTE: x1 <= x2 is guaranteed by construction
#[derive(Debug, PartialEq, Eq, Clone)]
struct Line {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum LineKind {
    Horizontal,
    Vertical,
    Diagonal,
}

impl Line {
    fn new(x1: u32, y1: u32, x2: u32, y2: u32) -> Self {
        if x1 > x2 {
            Self {
                x1: x2,
                y1: y2,
                x2: x1,
                y2: y1,
            }
        } else {
            Self { x1, y1, x2, y2 }
        }
    }

    fn parse(line: &str) -> Option<Self> {
        let mut parts = line.split(" -> ");
        let (x1, y1) = parse_point(parts.next()?)?;
        let (x2, y2) = parse_point(parts.next()?)?;
        Some(Self::new(x1, y1, x2, y2))
    }

    fn kind(&self) -> LineKind {
        if self.x1 == self.x2 {
            LineKind::Vertical
        } else if self.y1 == self.y2 {
            LineKind::Horizontal
        } else {
            LineKind::Diagonal
        }
    }

    fn map_points(&self, mut f: impl FnMut((u32, u32))) {
        match self.kind() {
            LineKind::Vertical => {
                let ymin = self.y1.min(self.y2);
                let ymax = self.y1.max(self.y2);
                for y in ymin..=ymax {
                    f((self.x1, y));
                }
            }
            LineKind::Horizontal => {
                // x1 <= x2 by construction
                for x in self.x1..=self.x2 {
                    f((x, self.y1));
                }
            }
            LineKind::Diagonal => {
                let mut x = self.x1;
                let mut y = self.y1 as i64;
                let dy = if self.y2 > self.y1 { 1i64 } else { -1 };

                while y != self.y2 as i64 {
                    f((x, y as u32));
                    y += dy;
                    x += 1;
                }

                f((x, y as u32));
            }
        }
    }
}

fn parse_point(point: &str) -> Option<(u32, u32)> {
    let (x, y) = point.split_once(',')?;
    Some((x.parse().ok()?, y.parse().ok()?))
}

fn parse_lines(lines: impl BufRead) -> Option<Vec<Line>> {
    lines
        .lines()
        .map(|l| Some(Line::parse(&l.ok()?)?))
        .collect::<Option<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, io};

    use super::*;

    #[test]
    fn test_parse_point() {
        assert_eq!(parse_point("1,2"), Some((1, 2)));
        assert_eq!(parse_point("30,200"), Some((30, 200)));
        assert_eq!(parse_point("30 200"), None);
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(Line::parse("1,2 -> 1,4"), Some(Line::new(1, 2, 1, 4)));
        assert_eq!(Line::parse("1, ->1,4"), None);
    }

    const SHORT_INPUT: &str = "\
0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
";

    #[test]
    fn test_parse_lines() {
        let lines = parse_lines(io::Cursor::new(SHORT_INPUT));
        assert_eq!(
            lines,
            Some(vec![
                Line::new(0, 9, 5, 9),
                Line::new(8, 0, 0, 8),
                Line::new(9, 4, 3, 4)
            ])
        );
    }

    mod map_points {
        use super::*;

        fn collect_points(line: &Line) -> Vec<(u32, u32)> {
            let mut result = Vec::new();
            line.map_points(|p| result.push(p));
            result
        }

        #[test]
        fn works_single_points() {
            let point = Line::new(5, 3, 5, 3);
            assert_eq!(collect_points(&point), vec![(5, 3)]);
        }

        #[test]
        fn works_horizontal() {
            let line = Line::new(5, 3, 7, 3);
            assert_eq!(collect_points(&line), vec![(5, 3), (6, 3), (7, 3)]);
            let line = Line::new(7, 3, 5, 3);
            assert_eq!(collect_points(&line), vec![(5, 3), (6, 3), (7, 3)]);
        }

        #[test]
        fn works_vertical() {
            let line = Line::new(8, 2, 8, 4);
            assert_eq!(collect_points(&line), vec![(8, 2), (8, 3), (8, 4)]);
            let line = Line::new(8, 4, 8, 2);
            assert_eq!(collect_points(&line), vec![(8, 2), (8, 3), (8, 4)]);
        }

        #[test]
        fn works_diagonal() {
            let line = Line::new(5, 2, 2, 5);
            assert_eq!(collect_points(&line), vec![(2, 5), (3, 4), (4, 3), (5, 2)]);
            let line = Line::new(0, 3, 3, 0);
            assert_eq!(collect_points(&line), vec![(0, 3), (1, 2), (2, 1), (3, 0)]);
        }
    }

    const TEST_INPUT: &str = "\
0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2
";

    #[test]
    fn test_intersection_points() {
        let lines = parse_lines(io::Cursor::new(TEST_INPUT)).unwrap();

        // Non-diagonals
        let nondiagonals: Vec<_> = lines
            .iter()
            .filter(|l| l.kind() != LineKind::Diagonal)
            .cloned()
            .collect();
        let grid = Grid::from(&nondiagonals).unwrap();
        let intersections: HashSet<_> = grid.intersections().collect();
        assert_eq!(
            intersections,
            HashSet::from([(3, 4), (7, 4), (0, 9), (1, 9), (2, 9)]),
        );
        assert_eq!(grid.count_intersections(), 5);

        // Diagonals
        let grid = Grid::from(&lines).unwrap();
        assert_eq!(grid.count_intersections(), 12);
    }
}
