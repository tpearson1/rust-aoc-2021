use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

const INITIAL_TIMER: u8 = 8;
const REPEAT_TIMER: u8 = 6;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day6/input.txt";
    let file = File::open(filename).map_err(|_| "Could not open file")?;
    let reader = BufReader::new(file);

    let initial = parse_input(reader).ok_or("Failed to read input")?;
    println!("After 80 days: {}", simulate(&initial, 80));
    println!("After 256 days: {}", simulate(&initial, 256));

    Ok(())
}

fn simulate(fish: &[Lanternfish], cycles: usize) -> u64 {
    let mut map: HashMap<Lanternfish, u64> = HashMap::new();
    for fish in fish {
        *map.entry(fish.clone()).or_insert(0) += 1;
    }

    for _ in 0..cycles {
        map = step(map);
    }

    map.values().sum()
}

fn step(mut map: HashMap<Lanternfish, u64>) -> HashMap<Lanternfish, u64> {
    let mut current_fish: Vec<_> = map
        .iter()
        .map(|(fish, count)| (fish.clone(), *count))
        .collect();

    let children: Vec<_> = current_fish
        .iter_mut()
        .filter_map(|(fish, count)| Some((fish.age()?, *count)))
        .collect();

    map.clear();
    for (parent, count) in current_fish {
        *map.entry(parent).or_insert(0) += count;
    }

    for (child, count) in children {
        *map.entry(child).or_insert(0) += count;
    }

    map
}

fn parse_input(reader: impl BufRead) -> Option<Vec<Lanternfish>> {
    let line = reader.lines().next()?.ok()?;
    let nums: Vec<_> = line
        .split(',')
        .map(|l| Some(Lanternfish::from(l.parse().ok()?)))
        .collect::<Option<Vec<_>>>()?;
    Some(nums)
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Lanternfish(u8);

impl std::fmt::Debug for Lanternfish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Lanternfish {
    fn new() -> Self {
        Lanternfish(INITIAL_TIMER)
    }

    fn from(timer: u8) -> Self {
        Lanternfish(timer)
    }

    fn age(&mut self) -> Option<Lanternfish> {
        if self.0 == 0 {
            self.0 = REPEAT_TIMER;
            Some(Lanternfish::new())
        } else {
            self.0 -= 1;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    fn make_state(nums: &[u8]) -> Vec<Lanternfish> {
        nums.iter().copied().map(Lanternfish::from).collect()
    }

    #[test]
    fn test_age() {
        let mut fish = Lanternfish::from(0);
        assert_eq!(fish.age(), Some(Lanternfish::new()));
        assert_eq!(fish.0, REPEAT_TIMER);

        let mut fish = Lanternfish::from(5);
        assert_eq!(fish.age(), None);
        assert_eq!(fish.0, 4);
    }

    #[test]
    fn test_parse_input() {
        let cursor = io::Cursor::new("1,2,3,4,5");
        let result = parse_input(cursor).expect("it's valid input");
        let expected = make_state(&[1, 2, 3, 4, 5]);
        assert_eq!(result, expected);

        let cursor = io::Cursor::new("1,2 ,3,4,5");
        assert_eq!(parse_input(cursor), None);
    }

    #[test]
    fn test_simulate() {
        let initial = make_state(&[3, 4, 3, 1, 2]);
        assert_eq!(simulate(&initial, 18), 26);
        assert_eq!(simulate(&initial, 80), 5934);
    }
}
