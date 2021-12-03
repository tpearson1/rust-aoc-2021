use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const BIT_COUNT: u8 = 12;

fn most_common_bit(bit: u8, nums: &[u16]) -> u16 {
    let zero_count = nums.iter().filter(|num| (*num >> bit) & 1 == 0).count();
    if zero_count > nums.len() / 2 {
        0
    } else {
        1
    }
}

fn least_common_bit(bit: u8, nums: &[u16]) -> u16 {
    1 - most_common_bit(bit, nums)
}

fn calculate_gamma(bit_count: u8, nums: &[u16]) -> u16 {
    (0..bit_count)
        .map(|bit| most_common_bit(bit, nums) << bit)
        .sum()
}

fn calculate_epsilon(bit_count: u8, gamma: u16) -> u16 {
    !gamma & ((1 << bit_count) - 1)
}

fn calculate_reduced_rating(
    bit_count: u8,
    nums: &[u16],
    bit_selector: impl for<'a> Fn(u8, &'a [u16]) -> u16,
) -> u16 {
    if nums.len() == 1 {
        return nums[0];
    }

    let mut nums = nums.to_vec();
    for bit in (0..bit_count).rev() {
        let selector = bit_selector(bit, &nums);
        nums = nums
            .iter()
            .filter(|num| (*num >> bit) & 1 == selector)
            .copied()
            .collect();
        if nums.len() == 1 {
            return nums[0];
        }
    }

    panic!("Unexpected edge case");
}

fn calculate_oxygen_rating(bit_count: u8, nums: &[u16]) -> u16 {
    calculate_reduced_rating(bit_count, nums, most_common_bit)
}

fn calculate_co2_rating(bit_count: u8, nums: &[u16]) -> u16 {
    calculate_reduced_rating(bit_count, nums, least_common_bit)
}

fn main() {
    let filename = "src/bin/day3/input.txt";
    let file = File::open(filename).expect("File not found");
    let reader = BufReader::new(file);

    let nums: Vec<_> = reader
        .lines()
        .map(|l| u16::from_str_radix(&l.unwrap(), 2).unwrap())
        .collect();

    let gamma = calculate_gamma(BIT_COUNT, &nums);
    let epsilon = calculate_epsilon(BIT_COUNT, gamma);
    println!("Gamma: {}", gamma);
    println!("Epsilon: {}", epsilon);
    println!(
        "Power consumption (product of above): {}",
        gamma as u32 * epsilon as u32
    );

    println!();
    let oxygen = calculate_oxygen_rating(BIT_COUNT, &nums);
    let co2 = calculate_co2_rating(BIT_COUNT, &nums);
    println!("Oxygen generator rating: {}", oxygen);
    println!("CO2 scrubber rating: {}", co2);
    println!(
        "Life support rating (product of above): {}",
        oxygen as u32 * co2 as u32
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &[u16] = &[
        0b00100, 0b11110, 0b10110, 0b10111, 0b10101, 0b01111, 0b00111, 0b11100, 0b10000, 0b11001,
        0b00010, 0b01010,
    ];

    #[test]
    fn test_gamma_epsilon() {
        let gamma = calculate_gamma(5, EXAMPLE);
        assert_eq!(gamma, 0b10110);
        let epsilon = calculate_epsilon(5, gamma);
        assert_eq!(epsilon, 0b01001);
    }

    #[test]
    fn test_reduced_ratings() {
        let oxygen = calculate_oxygen_rating(5, EXAMPLE);
        assert_eq!(oxygen, 23);
        let co2 = calculate_co2_rating(5, EXAMPLE);
        assert_eq!(co2, 10);
    }
}
