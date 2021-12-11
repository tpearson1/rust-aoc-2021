use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day8/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let input = Input::from(&input).ok_or("Failed to read input")?;
    println!(
        "Simple digit count in output: {}",
        count_simple_digits(&input)
    );

    let outputs = OutputDecoder::new()
        .decode_all(&input)
        .ok_or("Failed to decode outputs")?;
    println!("Sum of output values: {}", outputs.iter().sum::<u64>());

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Segment {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Segment {
    fn try_many_from_str(s: &str) -> Option<Vec<Segment>> {
        s.chars()
            .map(|c| c.try_into())
            .collect::<Result<Vec<_>, _>>()
            .ok()
    }
}

impl TryFrom<char> for Segment {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Segment::*;
        Ok(match value {
            'a' => A,
            'b' => B,
            'c' => C,
            'd' => D,
            'e' => E,
            'f' => F,
            'g' => G,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
impl From<Segment> for char {
    fn from(segment: Segment) -> Self {
        use Segment::*;
        match segment {
            A => 'a',
            B => 'b',
            C => 'c',
            D => 'd',
            E => 'e',
            F => 'f',
            G => 'g',
        }
    }
}

struct Input {
    digits: Vec<Vec<Segment>>,
    outputs: Vec<Vec<Segment>>,
    entry_count: usize,
}

const DIGITS_PER_ENTRY: usize = 10;
const OUTPUTS_PER_ENTRY: usize = 4;

impl Input {
    fn from(input: &str) -> Option<Self> {
        let mut digits = Vec::new();
        let mut outputs = Vec::new();
        let mut entry_count = 0;
        for line in input.lines() {
            let (d, o) = line.split_once(" | ")?;
            digits.extend_from_slice(
                &d.split(' ')
                    .map(Segment::try_many_from_str)
                    .collect::<Option<Vec<_>>>()?,
            );
            outputs.extend_from_slice(
                &o.split(' ')
                    .map(Segment::try_many_from_str)
                    .collect::<Option<Vec<_>>>()?,
            );
            entry_count += 1;
        }
        Some(Self {
            digits,
            outputs,
            entry_count,
        })
    }

    fn entry_count(&self) -> usize {
        self.entry_count
    }

    fn digits(&self, index: usize) -> &[Vec<Segment>] {
        let offset = index * DIGITS_PER_ENTRY;
        &self.digits[offset..(offset + DIGITS_PER_ENTRY)]
    }

    fn outputs(&self, index: usize) -> &[Vec<Segment>] {
        let offset = index * OUTPUTS_PER_ENTRY;
        &self.outputs[offset..(offset + OUTPUTS_PER_ENTRY)]
    }

    fn all_outputs(&self) -> &[Vec<Segment>] {
        &self.outputs
    }
}

fn count_simple_digits(input: &Input) -> usize {
    input
        .all_outputs()
        .iter()
        .filter(|x| {
            x.len() == 2 // 1
            || x.len() == 4 // 4
            || x.len() == 3 // 7
            || x.len() == 7 // 8
        })
        .count()
}

fn decode_segments(digits: &[Vec<Segment>]) -> Option<[Segment; 7]> {
    let one = digits.iter().find(|x| x.len() == 2)?;
    let seven = digits.iter().find(|x| x.len() == 3)?;

    let top_segment = *seven.iter().find(|x| !one.contains(*x))?;

    use Segment::*;
    let segment_counts = [A, B, C, D, E, F, G].map(|segment| {
        (
            segment,
            digits
                .iter()
                .filter(|digit| digit.contains(&segment))
                .count(),
        )
    });

    let top_left_segment = segment_counts.iter().find(|x| x.1 == 6)?.0;
    let bottom_left_segment = segment_counts.iter().find(|x| x.1 == 4)?.0;
    let bottom_right_segment = segment_counts.iter().find(|x| x.1 == 9)?.0;
    let top_right_segment = segment_counts
        .iter()
        .find(|x| x.1 == 8 && x.0 != top_segment)?
        .0;

    // Bottom or middle
    let four = digits.iter().find(|x| x.len() == 4)?;
    // Four does not use the bottom segment
    let bottom_segment = segment_counts
        .iter()
        .filter(|x| x.1 == 7)
        .map(|x| x.0)
        .find(|x| !four.contains(x))?;
    let middle_segment = segment_counts
        .iter()
        .filter(|x| x.1 == 7)
        .map(|x| x.0)
        .find(|x| four.contains(x))?;

    Some([
        top_segment,
        top_right_segment,
        bottom_right_segment,
        bottom_segment,
        bottom_left_segment,
        top_left_segment,
        middle_segment,
    ])
}

struct OutputDecoder {
    /// Maps patterns to digits
    lookup: HashMap<u8, u8>,
}

impl OutputDecoder {
    fn new() -> Self {
        let lookup = HashMap::from([
            (0b00111111, 0),
            (0b00000110, 1),
            (0b01011011, 2),
            (0b01001111, 3),
            (0b01100110, 4),
            (0b01101101, 5),
            (0b01111101, 6),
            (0b00000111, 7),
            (0b01111111, 8),
            (0b01101111, 9),
        ]);
        Self { lookup }
    }

    fn decode_all(&self, input: &Input) -> Option<Vec<u64>> {
        (0..input.entry_count())
            .map(|idx| self.decode_outputs(input.digits(idx), input.outputs(idx)))
            .collect::<Option<Vec<_>>>()
    }

    fn decode_outputs(&self, digits: &[Vec<Segment>], outputs: &[Vec<Segment>]) -> Option<u64> {
        let segments = decode_segments(digits)?;

        let result_digits = outputs
            .iter()
            .map(|output| {
                let pattern = segments
                    .iter()
                    .enumerate()
                    .map(|(idx, segment)| {
                        if output.contains(segment) {
                            1u8 << idx
                        } else {
                            0
                        }
                    })
                    .sum::<u8>();
                Some(*self.lookup.get(&pattern)? as u64)
            })
            .collect::<Option<Vec<_>>>()?;

        Some(
            result_digits[0] * 1000
                + result_digits[1] * 100
                + result_digits[2] * 10
                + result_digits[3],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "\
be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce
";

    #[test]
    fn test_input() {
        assert!(Input::from("not valid").is_none());

        let input = Input::from(TEST_INPUT).expect("it's valid input");

        #[rustfmt::skip]
        let expected = ["fbegcd", "cbd", "adcefb", "dageb", "afcb", "bc", "aefdc", "ecdab", "fgdeca", "fcdbega"];
        let expected = expected.map(|s| Segment::try_many_from_str(s).unwrap());
        assert_eq!(input.digits(3), expected);

        let expected = ["efabcd", "cedba", "gadfec", "cb"];
        let expected = expected.map(|s| Segment::try_many_from_str(s).unwrap());
        assert_eq!(input.outputs(3), expected);
    }

    #[test]
    fn test_count_simple_digits() {
        let input = Input::from(TEST_INPUT).unwrap();
        assert_eq!(count_simple_digits(&input), 26);
    }

    #[test]
    fn test_decode() {
        let input =
            "acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf";
        let input = Input::from(input).unwrap();

        let decoder = OutputDecoder::new();
        let result = decoder.decode_outputs(input.digits(0), input.outputs(0));
        assert_eq!(result, Some(5353));

        let input = Input::from(TEST_INPUT).unwrap();
        let expected = [8394, 9781, 1197, 9361, 4873, 8418, 4548, 1625, 8717, 4315];
        let actual = decoder.decode_all(&input).unwrap();
        assert_eq!(&actual, &expected);
    }
}
