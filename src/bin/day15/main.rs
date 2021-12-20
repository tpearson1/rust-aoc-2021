use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    fs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day15/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let grid = Grid::parse_from_str(&input).ok_or("Failed to parse input grid")?;

    let risk = grid.lowest_total_risk(false).ok_or("Failed to find path")?;
    println!("Lowest risk: {}", risk);
    let risk = grid
        .lowest_total_risk(true)
        .ok_or("Failed to find path (tiled)")?;
    println!("Lowest risk (tiled): {}", risk);

    Ok(())
}

#[derive(Clone)]
struct Grid {
    width: i32,
    height: i32,
    grid: Vec<u8>,
}

impl Grid {
    const TILE_COUNT: i32 = 5;

    pub fn parse_from_str(input: &str) -> Option<Self> {
        let width = input.find('\n')? as i32;
        if width == 0 {
            return None;
        }

        let grid = input
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| Some(c.to_digit(10)? as u8))
            .collect::<Option<Vec<u8>>>()?;
        let height = grid.len() as i32 / width;
        Some(Self {
            width,
            height,
            grid,
        })
    }

    fn get_at(&self, x: i32, y: i32, tiled: bool) -> Option<u8> {
        if tiled {
            return self.get_at_tiled(x, y);
        }

        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return None;
        }
        Some(self.grid[(y * self.width + x) as usize])
    }

    fn get_at_tiled(&self, x: i32, y: i32) -> Option<u8> {
        if x < 0
            || x >= Self::TILE_COUNT * self.width
            || y < 0
            || y >= Self::TILE_COUNT * self.height
        {
            return None;
        }

        let tile_x = x / self.width;
        let tile_y = y / self.height;

        let x = x - self.width * tile_x;
        let y = y - self.height * tile_y;

        let original_value = self.grid[(y * self.width + x) as usize];
        Some((original_value - 1 + tile_x as u8 + tile_y as u8) % 9 + 1)
    }

    // We use Uniform Cost Search
    pub fn lowest_total_risk(&self, tiled: bool) -> Option<u32> {
        let node = Node {
            grid: self,
            total_cost: 0,
            x: 0,
            y: 0,
        };

        let mut frontier: BinaryHeap<Node> = BinaryHeap::from([node]);
        let mut added = HashSet::new();
        let mut explored = HashSet::new();

        while let Some(node) = frontier.pop() {
            added.remove(&(node.x, node.y));

            if node.is_goal(tiled) {
                return Some(node.total_cost);
            }

            explored.insert((node.x, node.y));

            for neighbor in node.neighbors(tiled) {
                if !explored.contains(&(neighbor.x, neighbor.y))
                    && !added.contains(&(neighbor.x, neighbor.y))
                {
                    frontier.push(neighbor.clone());
                    added.insert((neighbor.x, neighbor.y));
                }
            }
        }

        None
    }
}

#[derive(Clone)]
struct Node<'grid> {
    grid: &'grid Grid,
    total_cost: u32,
    x: i32,
    y: i32,
}

impl<'grid> PartialEq for Node<'grid> {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost == other.total_cost && self.x == other.x && self.y == other.y
    }
}

impl<'grid> Eq for Node<'grid> {}

impl<'grid> Ord for Node<'grid> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invert so as to get a min heap
        other
            .total_cost
            .cmp(&self.total_cost)
            .then_with(|| self.x.cmp(&other.x))
            .then_with(|| self.y.cmp(&other.y))
    }
}

impl<'grid> PartialOrd for Node<'grid> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'grid> Node<'grid> {
    const OFFSETS: &'static [(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];

    fn is_goal(&self, tiled: bool) -> bool {
        let width = if tiled {
            Grid::TILE_COUNT * self.grid.width
        } else {
            self.grid.width
        };

        let len = if tiled {
            self.grid.grid.len() as i32 * Grid::TILE_COUNT * Grid::TILE_COUNT
        } else {
            self.grid.grid.len() as i32
        };

        self.y * width + self.x == len - 1
    }

    fn neighbors(&self, tiled: bool) -> impl Iterator<Item = Node<'grid>> + '_ {
        Self::OFFSETS
            .iter()
            .map(move |(dx, dy)| {
                let x = self.x + dx;
                let y = self.y + dy;
                self.grid.get_at(x, y, tiled).map(|cost| Node {
                    grid: self.grid,
                    total_cost: self.total_cost + cost as u32,
                    x,
                    y,
                })
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "\
1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581";

    #[test]
    fn test_parse_input() {
        let grid = Grid::parse_from_str(TEST_INPUT).unwrap();
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(&grid.grid[..13], [1, 1, 6, 3, 7, 5, 1, 7, 4, 2, 1, 3, 8]);
    }

    #[test]
    fn test_get_at() {
        let grid = Grid::parse_from_str(TEST_INPUT).unwrap();
        assert_eq!(grid.get_at(0, 0, false), Some(1));
        assert_eq!(grid.get_at(0, 0, true), Some(1));
        assert_eq!(grid.get_at(2, 1, false), Some(8));
        assert_eq!(grid.get_at(2, 1, true), Some(8));

        assert_eq!(grid.get_at(3, 6, false), Some(9));
        assert_eq!(grid.get_at(3, 6, true), Some(9));
        assert_eq!(grid.get_at(10 * 3 + 3, 10 * 2 + 6, false), None);
        // 9 + 5 = 14, 10 -> 1 so 14 -> 5
        assert_eq!(grid.get_at(10 * 3 + 3, 10 * 2 + 6, true), Some(5));
    }

    #[test]
    fn test_lowest_cost_path() {
        let grid = Grid::parse_from_str(TEST_INPUT).unwrap();

        let risk = grid.lowest_total_risk(false);
        assert_eq!(risk, Some(40));

        let tiled_risk = grid.lowest_total_risk(true);
        assert_eq!(tiled_risk, Some(315));
    }
}
