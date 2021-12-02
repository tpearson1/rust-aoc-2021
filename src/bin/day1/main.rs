use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn count_increasing(data: &[i64]) -> usize {
    data.windows(2).filter(|pair| pair[1] > pair[0]).count()
}

fn window_sums(data: &[i64], window_size: usize) -> Vec<i64> {
    assert_ne!(window_size, 0);
    data.windows(window_size)
        .map(|w| w.iter().sum())
        .collect::<Vec<_>>()
}

fn count_window_increasing(data: &[i64], window_size: usize) -> usize {
    count_increasing(&window_sums(data, window_size))
}

fn main() {
    let filename = "src/bin/day1/input.txt";
    let file = File::open(filename).expect("File not found");
    let reader = BufReader::new(file);

    let data: Vec<_> = reader
        .lines()
        .map(|l| l.unwrap().parse::<i64>().unwrap())
        .collect();

    let total_increasing = count_increasing(&data);
    println!("Total increasing: {}", total_increasing);

    const WINDOW_SIZE: usize = 3;
    let window_increasing = count_window_increasing(&data, WINDOW_SIZE);
    println!(
        "Total windows (of size {}) increasing: {}",
        WINDOW_SIZE, window_increasing
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_increasing() {
        assert_eq!(count_increasing(&[]), 0);
        assert_eq!(count_increasing(&[1]), 0);
        assert_eq!(count_increasing(&[1, 2, 3, 4]), 3);
        assert_eq!(count_increasing(&[1, 2, 2, 4]), 2);
        assert_eq!(count_increasing(&[1, 2, 1, 4, 3, 2, 7]), 3);

        // Example given
        let arr = [199, 200, 208, 210, 200, 207, 240, 269, 260, 263];
        assert_eq!(count_increasing(&arr), 7);
    }

    #[test]
    fn test_window_sums() {
        // Window size = 1 is idempotent
        let data = [199, 200, 208, 210, 200, 207, 240, 269, 260, 263];
        assert_eq!(window_sums(&data, 1), data);

        let data = [5, 7, 6, 7, 4, 10];
        assert_eq!(window_sums(&data, 2), &[12, 13, 13, 11, 14]);
    }

    #[test]
    fn test_count_window_increasing() {
        let data = [199, 200, 208, 210, 200, 207, 240, 269, 260, 263];
        assert_eq!(count_window_increasing(&data, 1), 7);

        let data = [5, 7, 6, 7, 4, 10];
        assert_eq!(count_window_increasing(&data, 2), 2);

        // Example given
        let arr = [199, 200, 208, 210, 200, 207, 240, 269, 260, 263];
        assert_eq!(count_window_increasing(&arr, 3), 5);
    }
}
