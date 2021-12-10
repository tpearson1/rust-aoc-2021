use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day7/input.txt";
    let file = File::open(filename).map_err(|_| "Could not open file")?;
    let reader = BufReader::new(file);

    let positions = parse_input(reader).ok_or("Failed to read input")?;

    let (best, fuel) = best_destination::<false>(&positions).ok_or("No positions given")?;
    println!(
        "(Linear) Best position is {} with fuel usage {}",
        best, fuel
    );

    let (best, fuel) = best_destination::<true>(&positions).ok_or("No positions given")?;
    println!(
        "(Quadratic) Best position is {} with fuel usage {}",
        best, fuel
    );

    Ok(())
}

fn parse_input(reader: impl BufRead) -> Option<Vec<u32>> {
    let line = reader.lines().next()?.ok()?;
    let nums: Vec<_> = line
        .split(',')
        .map(|l| l.parse().ok())
        .collect::<Option<Vec<_>>>()?;
    Some(nums)
}

fn best_destination<const QUADRATIC: bool>(positions: &[u32]) -> Option<(u32, u64)> {
    let min = *positions.iter().min()?;
    let max = *positions.iter().max()?;
    let result = (min..=max)
        .map(|dst| (dst, cost_for_destination::<QUADRATIC>(positions, dst)))
        .min_by_key(|(_, cost)| *cost)?;
    Some(result)
}

fn cost_for_destination<const QUADRATIC: bool>(positions: &[u32], destination: u32) -> u64 {
    if QUADRATIC {
        positions
            .iter()
            .map(|&p| {
                let dist = (p as i64 - destination as i64).abs() as u64;
                dist * (dist + 1) / 2
            })
            .sum()
    } else {
        positions
            .iter()
            .map(|&p| (p as i64 - destination as i64).abs() as u64)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn test_parse_input() {
        let cursor = io::Cursor::new("1,2,3,4,5");
        let result = parse_input(cursor).expect("it's valid input");
        let expected = &[1, 2, 3, 4, 5];
        assert_eq!(result, expected);

        let cursor = io::Cursor::new("1,2 ,3,4,5");
        assert_eq!(parse_input(cursor), None);
    }

    const TEST_POSITIONS: &[u32] = &[16, 1, 2, 0, 4, 2, 7, 1, 2, 14];

    mod test_best_destination {
        use super::*;

        #[test]
        fn linear() {
            let result = best_destination::<false>(TEST_POSITIONS);
            assert_eq!(result, Some((2, 37)));
        }

        #[test]
        fn quadratic() {
            let result = best_destination::<true>(TEST_POSITIONS);
            assert_eq!(result, Some((5, 168)));
        }
    }

    mod test_cost_for_destination {
        use super::*;

        #[test]
        fn linear() {
            let result = cost_for_destination::<false>(TEST_POSITIONS, 2);
            assert_eq!(result, 37);
            let result = cost_for_destination::<false>(TEST_POSITIONS, 1);
            assert_eq!(result, 41);
            let result = cost_for_destination::<false>(TEST_POSITIONS, 3);
            assert_eq!(result, 39);
            let result = cost_for_destination::<false>(TEST_POSITIONS, 10);
            assert_eq!(result, 71);
        }

        #[test]
        fn quadratic() {
            let result = cost_for_destination::<true>(TEST_POSITIONS, 2);
            assert_eq!(result, 206);
            let result = cost_for_destination::<true>(TEST_POSITIONS, 5);
            assert_eq!(result, 168);
        }
    }
}
