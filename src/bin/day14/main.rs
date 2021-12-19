use std::{collections::HashMap, fs};

use itertools::Itertools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day14/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let mut grower: PolymerGrower = PolymerInput::parse_from_str(&input)
        .ok_or("Failed to parse input")?
        .into();
    grower.grow(10);
    let polymer_score = grower.polymer_score().ok_or("Couldn't get polymer score")?;
    println!("Polymer Score (10 steps total): {}", polymer_score);

    grower.grow(40 - 10);
    let polymer_score = grower.polymer_score().ok_or("Couldn't get polymer score")?;
    println!("Polymer Score (40 steps total): {}", polymer_score);

    Ok(())
}

pub struct PolymerInput {
    polymer: Vec<char>,
    rules: HashMap<(char, char), char>,
}

impl PolymerInput {
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
}

pub struct PolymerGrower {
    polymer_triple_counts: HashMap<(char, char, char), usize>,
    rules: HashMap<(char, char), char>,
}

impl From<PolymerInput> for PolymerGrower {
    fn from(input: PolymerInput) -> Self {
        let mut polymer_triple_counts = HashMap::new();
        let triples = input
            .polymer
            .windows(3)
            .map(|triple| (triple[0], triple[1], triple[2]));
        for triple in triples {
            *polymer_triple_counts.entry(triple).or_insert(0) += 1;
        }

        if let Some(pair) = input.polymer.windows(2).next() {
            // Special triples to make counting easier
            polymer_triple_counts.insert((Self::MARKER_CHAR, Self::MARKER_CHAR, pair[0]), 1);
            polymer_triple_counts.insert((Self::MARKER_CHAR, pair[0], pair[1]), 1);
        }

        if let Some(pair) = input.polymer.windows(2).last() {
            // Special triple so we always check the last two
            polymer_triple_counts.insert((pair[0], pair[1], Self::MARKER_CHAR), 1);
            // Make counting easier
            polymer_triple_counts.insert((pair[1], Self::MARKER_CHAR, Self::MARKER_CHAR), 1);
        }

        Self {
            polymer_triple_counts,
            rules: input.rules,
        }
    }
}

impl PolymerGrower {
    // This is fine to use since by using `input.lines()` when parsing, we know
    // for certain that '\n' cannot be used as part of the polymer or for any of
    // the substitutions
    const MARKER_CHAR: char = '\n';

    pub fn grow(&mut self, steps: usize) {
        let mut new_triples = HashMap::new();
        for _ in 0..steps {
            for (&(c1, c2, c3), &count) in self.polymer_triple_counts.iter() {
                let rule1 = self.rules.get(&(c1, c2));
                let rule2 = self.rules.get(&(c2, c3));

                match (rule1, rule2) {
                    (None, None) => {
                        // Keep as is
                        *new_triples.entry((c1, c2, c3)).or_insert(0) += count;
                    }
                    (None, Some(&to)) => {
                        *new_triples.entry((c1, c2, to)).or_insert(0) += count;
                        // (c2, to, c3) handled by next triple
                    }
                    (Some(&to), None) => {
                        *new_triples.entry((c1, to, c2)).or_insert(0) += count;
                        *new_triples.entry((to, c2, c3)).or_insert(0) += count;
                    }
                    (Some(&to1), Some(&to2)) => {
                        *new_triples.entry((c1, to1, c2)).or_insert(0) += count;
                        *new_triples.entry((to1, c2, to2)).or_insert(0) += count;
                        // (c2, to2, c3) handled by next triple
                    }
                }
            }

            std::mem::swap(&mut new_triples, &mut self.polymer_triple_counts);
            new_triples.clear();
        }
    }

    #[cfg(test)]
    fn polymer_len(&self) -> usize {
        // NOTE: no -2 instead of +2 because of the four "special" triples
        self.polymer_triple_counts.values().sum::<usize>() - 2
    }

    pub fn polymer_score(&self) -> Option<usize> {
        let mut counts: HashMap<char, usize> = HashMap::new();

        for (&(c1, c2, c3), &count) in self.polymer_triple_counts.iter() {
            *counts.entry(c1).or_insert(0) += count;
            *counts.entry(c2).or_insert(0) += count;
            *counts.entry(c3).or_insert(0) += count;
        }

        counts.remove(&Self::MARKER_CHAR);

        counts
            .iter()
            .map(|(_, count)| *count)
            .minmax()
            .into_option()
            // Divide by 3 because (due to the special triples we added) each
            // character in the polymer is counted three times (in three
            // different triples)
            .map(|(min, max)| (max - min) / 3)
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
        let input = PolymerInput::parse_from_str(EXAMPLE_INPUT).unwrap();
        assert_eq!(input.polymer, ['N', 'N', 'C', 'B']);
        assert_eq!(input.rules.len(), 16);
        assert_eq!(input.rules.get(&('C', 'B')), Some(&'H'));
        assert_eq!(input.rules.get(&('B', 'C')), Some(&'B'));
        assert_eq!(input.rules.get(&('A', 'S')), None);
    }

    #[test]
    fn test_grow() {
        let mut grower: PolymerGrower = PolymerInput::parse_from_str(EXAMPLE_INPUT).unwrap().into();

        grower.grow(1);
        assert_eq!(grower.polymer_len(), "NCNBCHB".len());
        grower.grow(1);
        assert_eq!(grower.polymer_len(), "NBCCNBBBCBHCB".len());
        grower.grow(1);
        assert_eq!(grower.polymer_len(), "NBBBCNCCNBBNBNBBCHBHHBCHB".len());
        grower.grow(1);
        assert_eq!(
            grower.polymer_len(),
            "NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB".len()
        );

        grower.grow(6);
        assert_eq!(grower.polymer_len(), 3073);
        assert_eq!(grower.polymer_score(), Some(1588));

        grower.grow(40 - 10);
        assert_eq!(grower.polymer_score(), Some(2188189693529));
    }
}
