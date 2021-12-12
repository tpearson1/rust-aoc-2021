use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day9/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let map = Map::from_str(&input).ok_or("Could not parse input")?;
    let risk_level = Map::total_risk_level(map.low_points());
    println!("Sum of risk levels: {}", risk_level);

    let (_, result) = Basins::new(map).compute_basins();
    let size_product = largest_basins_product(result.basin_sizes().collect());
    println!("Product of three largest basin sizes: {}", size_product);

    Ok(())
}

#[derive(Debug)]
struct Map {
    width: usize,
    height: usize,
    map: Vec<u8>,
}

impl Map {
    pub const MAX_HEIGHT: u8 = 9;

    pub fn from_str(input: &str) -> Option<Self> {
        let width = input.find('\n')?;
        if width == 0 {
            return None;
        }

        let map = input
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| Some(c.to_digit(10)? as u8))
            .collect::<Option<Vec<u8>>>()?;
        let height = map.len() / width;
        Some(Self { width, height, map })
    }

    pub fn points(&self) -> impl Iterator<Item = ((i32, i32), u8)> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| {
                (
                    (x as i32, y as i32),
                    self.height_at(x as i32, y as i32).unwrap(),
                )
            })
        })
    }

    pub fn low_points(&self) -> impl Iterator<Item = ((i32, i32), u8)> + '_ {
        self.points().filter(move |&((x, y), height)| {
            let up = self.height_at(x, y - 1).unwrap_or(u8::MAX);
            let down = self.height_at(x, y + 1).unwrap_or(u8::MAX);
            let left = self.height_at(x - 1, y).unwrap_or(u8::MAX);
            let right = self.height_at(x + 1, y).unwrap_or(u8::MAX);
            height < up && height < down && height < left && height < right
        })
    }

    #[inline]
    pub fn height_at(&self, x: i32, y: i32) -> Option<u8> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        self.map.get(y as usize * self.width + x as usize).copied()
    }

    pub fn total_risk_level(low_points: impl Iterator<Item = ((i32, i32), u8)>) -> u64 {
        low_points
            .map(|(_, height)| Self::risk_level(height) as u64)
            .sum()
    }

    pub fn risk_level(height: u8) -> u8 {
        height + 1
    }
}

type BasinId = usize;

#[derive(Debug)]
struct Basins {
    map: Map,
    unvisited: HashSet<(i32, i32)>,
    basin_sizes: HashMap<BasinId, usize>,
    basin_points: HashSet<(BasinId, i32, i32)>,
}

impl Basins {
    pub fn new(map: Map) -> Self {
        let unvisited = map
            .points()
            .filter(|&(_, height)| height != Map::MAX_HEIGHT)
            .map(|(point, _)| point)
            .collect();
        Self {
            map,
            unvisited,
            basin_sizes: HashMap::new(),
            basin_points: HashSet::new(),
        }
    }

    pub fn compute_basins(mut self) -> (Map, BasinsResult) {
        let mut basin_id = 0;
        while let Some(&entry) = self.unvisited.iter().next() {
            self.compute_basin(basin_id, entry);
            basin_id += 1;
        }

        debug_assert_eq!(self.unvisited.len(), 0);

        let result = BasinsResult {
            basin_sizes: self.basin_sizes,
        };
        (self.map, result)
    }

    fn compute_basin(&mut self, basin_id: BasinId, start: (i32, i32)) {
        let mut queue = VecDeque::new();
        queue.push_back(start);
        self.unvisited.remove(&start);

        let mut size = 0;
        while let Some((x, y)) = queue.pop_front() {
            for &(x, y) in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if let Some(height) = self.map.height_at(x, y) {
                    if height != Map::MAX_HEIGHT && self.unvisited.contains(&(x, y)) {
                        queue.push_back((x, y));
                        self.unvisited.remove(&(x, y));
                    }
                }
            }

            size += 1;
            self.basin_points.insert((basin_id, x, y));
        }
        self.basin_sizes.insert(basin_id, size);
    }
}

#[derive(Debug)]
struct BasinsResult {
    basin_sizes: HashMap<BasinId, usize>,
}

impl BasinsResult {
    pub fn basin_sizes(&self) -> impl Iterator<Item = usize> + '_ {
        self.basin_sizes.values().copied()
    }
}

pub fn largest_basins_product(mut basins: Vec<usize>) -> usize {
    basins.sort_unstable();
    basins.iter().rev().take(3).product()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "\
2199943210
3987894921
9856789892
8767896789
9899965678
";

    mod map {
        use super::*;

        #[test]
        fn from_str() {
            let map = Map::from_str(TEST_INPUT).unwrap();
            assert_eq!(map.width, 10);
            assert_eq!(&map.map[0..5], &[2, 1, 9, 9, 9]);
            assert_eq!(&map.map[10..15], &[3, 9, 8, 7, 8]);
        }

        #[test]
        fn low_points() {
            let map = Map::from_str(TEST_INPUT).unwrap();
            let low: Vec<_> = map.low_points().collect();
            assert_eq!(&low, &[((1, 0), 1), ((9, 0), 0), ((2, 2), 5), ((6, 4), 5)]);
            assert_eq!(Map::total_risk_level(low.iter().copied()), 15);
        }

        #[test]
        fn basins() {
            let map = Map::from_str(TEST_INPUT).unwrap();
            let (_, result) = Basins::new(map).compute_basins();
            let sizes: Vec<_> = result.basin_sizes().collect();
            assert_eq!(sizes.len(), 4);
            assert_eq!(largest_basins_product(sizes), 1134)
        }
    }
}
