use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    let filename = "src/bin/.../input.txt";
    let file = File::open(filename).expect("File not found");
    let reader = BufReader::new(file);

    let data: Vec<_> = reader
        .lines()
        .map(|l| l.unwrap().parse::<i64>().unwrap())
        .collect();

    println!("{:?}", data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xyz() {}
}
