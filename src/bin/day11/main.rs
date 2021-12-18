use std::{collections::VecDeque, fmt::Display, fs};

use itertools::Itertools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day11/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let mut grid = OctopusGrid::from_str(&input).ok_or("Failed to parse input grid")?;
    let flash_total = grid.clone().simulate(100);
    println!("Flash total after 100 steps: {}", flash_total);

    let steps_required = grid.simulate_until_all_flash();
    println!(
        "First step where all are flashing at once: {}",
        steps_required
    );

    Ok(())
}

#[derive(Clone, Copy)]
pub struct Octopus(u8);

impl Octopus {
    /// If the octopus has any value greater than this then it is flashing
    const MAX_INACTIVE_VALUE: u8 = 9;

    pub fn is_flashing(&self) -> bool {
        self.0 > Self::MAX_INACTIVE_VALUE
    }

    /// Returns `true` if the octopus started flashing as a result of the
    /// increase
    pub fn increase(&mut self) -> bool {
        self.0 += 1;
        self.0 == Self::MAX_INACTIVE_VALUE + 1
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

impl Display for Octopus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_flashing() {
            write!(f, "X")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[derive(Clone)]
struct OctopusGrid {
    width: i32,
    height: i32,
    grid: Vec<Octopus>,
}

impl OctopusGrid {
    pub fn from_str(input: &str) -> Option<Self> {
        let width = input.find('\n')? as i32;
        if width == 0 {
            return None;
        }

        let grid = input
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| Some(Octopus(c.to_digit(10)? as u8)))
            .collect::<Option<Vec<Octopus>>>()?;
        let height = grid.len() as i32 / width;
        Some(Self {
            width,
            height,
            grid,
        })
    }

    pub fn simulate(&mut self, steps: usize) -> u64 {
        (0..steps).map(|_| self.step()).sum()
    }

    /// Returns the number of steps from the current grid before all octopi are
    /// flashing at once.
    ///
    /// NOTE: if all are flashing already, this will not return 0, but rather
    /// the number of steps until the next such occurrence
    pub fn simulate_until_all_flash(&mut self) -> u64 {
        let mut i = 0;
        loop {
            i += 1;
            let flash_total = self.step();
            if flash_total == self.grid.len() as u64 {
                return i;
            }
        }
    }

    pub fn step(&mut self) -> u64 {
        let mut unprocessed_flashing = VecDeque::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let oct = self.entry_mut(x, y).unwrap();
                if oct.increase() {
                    unprocessed_flashing.push_back((x, y));
                }
            }
        }

        while let Some((x, y)) = unprocessed_flashing.pop_front() {
            unprocessed_flashing.extend(self.process_neighbors(x, y));
        }

        self.grid
            .iter_mut()
            .filter(|oct| oct.is_flashing())
            .fold(0, |acc, oct| {
                oct.reset();
                acc + 1
            })
    }

    fn entry_mut(&mut self, x: i32, y: i32) -> Option<&mut Octopus> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        let index = (y * self.width as i32 + x) as usize;
        Some(&mut self.grid[index])
    }

    /// Increase neighbours (all 8) and return an iterator of the neighbors that
    /// increase began flashing
    fn process_neighbors(&mut self, x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        (x - 1..=x + 1)
            .cartesian_product(y - 1..=y + 1)
            .flat_map(move |(x, y)| {
                if self.entry_mut(x, y)?.increase() {
                    Some((x, y))
                } else {
                    None
                }
            })
    }
}

// #[cfg(test)]
impl Display for OctopusGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.grid[((y * self.width) + x) as usize])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "\
5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526
";

    #[test]
    fn test_parse_input() {
        let grid = OctopusGrid::from_str(TEST_INPUT).unwrap();
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.grid.len(), 100);
        assert_eq!(grid.to_string(), TEST_INPUT);
    }

    #[test]
    fn test_step() {
        const EXAMPLE: &str = "\
11111
19991
19191
19991
11111";

        let mut grid = OctopusGrid::from_str(EXAMPLE).unwrap();

        let flash_count = grid.step();
        assert_eq!(
            grid.to_string(),
            "\
34543
40004
50005
40004
34543
"
        );
        assert_eq!(flash_count, 9);

        let flash_count = grid.step();
        assert_eq!(
            grid.to_string(),
            "\
45654
51115
61116
51115
45654
"
        );
        assert_eq!(flash_count, 0);
    }

    #[test]
    fn test_simulate() {
        let mut grid = OctopusGrid::from_str(TEST_INPUT).unwrap();
        let flash_total = grid.simulate(100);
        assert_eq!(grid.to_string(), RESULT);
        assert_eq!(flash_total, 1656);

        const RESULT: &str = "\
0397666866
0749766918
0053976933
0004297822
0004229892
0053222877
0532222966
9322228966
7922286866
6789998766
";
    }

    #[test]
    fn test_simulate_sync() {
        let mut grid = OctopusGrid::from_str(TEST_INPUT).unwrap();
        let steps_required = grid.simulate_until_all_flash();
        assert_eq!(steps_required, 195);
    }
}
