use std::{collections::HashMap, fs};

use itertools::Itertools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day14/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let mut grower = PolymerGrower::parse_from_str(&input).ok_or("Failed to parse input")?;
    grower.grow(10);
    let polymer_score = grower.polymer_score().ok_or("Couldn't get polymer score")?;
    println!("Polymer Score (10 steps total): {}", polymer_score);

    Ok(())
}

pub struct PolymerGrower {
    polymer: Vec<char>,
    rules: HashMap<(char, char), char>,
}

impl PolymerGrower {
    pub fn parse_from_str(input: &str) -> Option<Self> {
        let mut lines = input.lines();

        let polymer = lines.next()?.chars().collect();
        if !lines.next()?.is_empty() {
            return None;
        }

        let rules = lines
            .map(|line| {
                let (from, to) = line.split_once(" -> ")?;

                let mut from_chars = from.chars();
                let from = (from_chars.next()?, from_chars.next()?);
                if from_chars.next().is_some() {
                    return None;
                }

                let mut to_chars = to.chars();
                let to = to_chars.next()?;
                if to_chars.next().is_some() {
                    return None;
                }

                Some((from, to))
            })
            .collect::<Option<HashMap<_, _>>>()?;

        Some(Self { polymer, rules })
    }

    pub fn grow(&mut self, steps: usize) {
        let mut scratch = Vec::new();
        for _ in 0..steps {
            for pair in self.polymer.windows(2) {
                let from = (pair[0], pair[1]);
                let to = self.rules.get(&from);
                if let Some(add) = to {
                    scratch.extend([pair[0], *add]);
                } else {
                    scratch.push(pair[0]);
                }
            }

            if let Some(last) = self.polymer.last() {
                scratch.push(*last);
            }

            std::mem::swap(&mut self.polymer, &mut scratch);
            scratch.clear();
        }
    }

    pub fn polymer(&self) -> String {
        self.polymer.iter().collect()
    }

    pub fn polymer_score(&self) -> Option<u32> {
        let mut counts: HashMap<char, u32> = HashMap::new();
        for &c in self.polymer.iter() {
            *counts.entry(c).or_insert(0) += 1;
        }

        counts
            .iter()
            .map(|(_, count)| *count)
            .minmax()
            .into_option()
            .map(|(min, max)| max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C
";

    #[test]
    fn test_parse_input() {
        let grower = PolymerGrower::parse_from_str(EXAMPLE_INPUT).unwrap();
        assert_eq!(grower.polymer, ['N', 'N', 'C', 'B']);
        assert_eq!(grower.polymer(), "NNCB");
        assert_eq!(grower.rules.len(), 16);
        assert_eq!(grower.rules.get(&('C', 'B')), Some(&'H'));
        assert_eq!(grower.rules.get(&('B', 'C')), Some(&'B'));
        assert_eq!(grower.rules.get(&('A', 'S')), None);
    }

    #[test]
    fn test_grow() {
        let mut grower = PolymerGrower::parse_from_str(EXAMPLE_INPUT).unwrap();

        grower.grow(1);
        assert_eq!(grower.polymer(), "NCNBCHB");
        grower.grow(1);
        assert_eq!(grower.polymer(), "NBCCNBBBCBHCB");
        grower.grow(1);
        assert_eq!(grower.polymer(), "NBBBCNCCNBBNBNBBCHBHHBCHB");
        grower.grow(1);
        assert_eq!(
            grower.polymer(),
            "NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB"
        );

        grower.grow(6);
        assert_eq!(grower.polymer.len(), 3073);
        assert_eq!(grower.polymer_score(), Some(1588));
    }
}
