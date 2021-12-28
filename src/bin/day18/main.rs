use std::{fmt::Display, fs, mem, ops::AddAssign};

mod parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day18/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let nums = parse::parse(&input)?;
    let expr = sum_many(&nums);
    let mag = expr.magnitude();
    println!("Magnitude of sum of all numbers in input: {}", mag);

    let max = max_magnitude_pair(&nums);
    println!("Maximum magnitude of pair of numbers in input: {}", max);

    Ok(())
}

pub fn sum_many(nums: &[parse::SnailfishNum]) -> Expression {
    let mut simplifier = ExpressionSimplifier::default();
    let mut expr = Expression::default();
    for num in nums {
        expr += num;
        expr = simplifier.simplify(expr);
    }
    expr
}

pub fn max_magnitude_pair(nums: &[parse::SnailfishNum]) -> u64 {
    let mut simplifier = ExpressionSimplifier::default();
    let mut expr = Expression::default();

    let mut max_magnitude = 0;

    for i in 0..nums.len() {
        for j in 0..nums.len() {
            if i == j {
                continue;
            }

            expr.join(&nums[i]);
            expr.join(&nums[j]);

            expr = simplifier.simplify(expr);
            max_magnitude = max_magnitude.max(expr.magnitude());

            expr.clear();
        }
    }

    max_magnitude
}

#[derive(Default)]
pub struct Expression {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    Begin,
    End,
    Num(u32),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut needs_sep = false;
        for entry in &self.entries {
            if needs_sep {
                needs_sep = false;
                if *entry != Entry::End {
                    write!(f, ",")?;
                }
            }

            match entry {
                Entry::Begin => write!(f, "[")?,
                Entry::End => {
                    write!(f, "]")?;
                    needs_sep = true;
                }
                Entry::Num(n) => {
                    write!(f, "{}", n)?;
                    needs_sep = true;
                }
            }
        }

        Ok(())
    }
}

impl Expression {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn join(&mut self, num: &parse::SnailfishNum) {
        fn inner(num: &parse::SnailfishNum, buf: &mut Vec<Entry>) {
            match num {
                parse::SnailfishNum::Num(n) => buf.push(Entry::Num(*n)),
                parse::SnailfishNum::Pair(children) => {
                    buf.push(Entry::Begin);
                    inner(&children[0], buf);
                    inner(&children[1], buf);
                    buf.push(Entry::End);
                }
            }
        }

        let first = self.entries.is_empty();
        if !first {
            self.entries.insert(0, Entry::Begin);
        }

        inner(num, &mut self.entries);

        if !first {
            self.entries.push(Entry::End);
        }
    }

    pub fn magnitude(&self) -> u64 {
        fn inner(entries: &[Entry]) -> (usize, u64) {
            match &entries[0] {
                Entry::Begin => {
                    let (next, mag) = inner(&entries[1..]);
                    let (next2, mag2) = inner(&entries[(1 + next)..]);
                    (1 + next + next2, 3 * mag + 2 * mag2)
                }
                Entry::End => {
                    let (next, mag) = inner(&entries[1..]);
                    (1 + next, mag)
                }
                Entry::Num(n) => (1, *n as u64),
            }
        }

        inner(&self.entries).1
    }
}

impl<'a> AddAssign<&'a parse::SnailfishNum> for Expression {
    fn add_assign(&mut self, rhs: &'a parse::SnailfishNum) {
        self.join(rhs);
    }
}

#[derive(Default)]
struct ExpressionSimplifier {
    expr: Vec<Entry>,
    buffer: Vec<Entry>,
}

impl ExpressionSimplifier {
    pub fn simplify(&mut self, expr: Expression) -> Expression {
        self.expr = expr.entries;

        loop {
            while self.iter_explode() {}
            if !self.iter_split() {
                break;
            }
        }

        self.buffer.clear();
        Expression {
            entries: mem::take(&mut self.expr),
        }
    }

    fn iter_explode(&mut self) -> bool {
        self.buffer.clear();

        let mut has_changed = false;

        let mut nesting = 0;
        let mut prev_num: Option<usize> = None;
        let mut next_inc: Option<u32> = None;
        let mut iter = self.expr.iter().enumerate();

        while let Some((idx, entry)) = iter.next() {
            match entry {
                Entry::Begin => {
                    nesting += 1;
                    self.buffer.push(Entry::Begin);
                }
                Entry::End => {
                    nesting -= 1;
                    self.buffer.push(Entry::End);
                }
                Entry::Num(n) => {
                    let n = if let Some(next_inc) = next_inc {
                        n + next_inc
                    } else {
                        *n
                    };

                    next_inc = None;

                    if nesting != 5 {
                        prev_num = Some(self.buffer.len());
                        self.buffer.push(Entry::Num(n));
                        continue;
                    }

                    if let Some(Entry::Num(next)) = self.expr.get(idx + 1) {
                        // Perform explode
                        has_changed = true;

                        if let Some(prev) = prev_num {
                            if let Entry::Num(prev) = &mut self.buffer[prev] {
                                *prev += n;
                            }
                        }

                        next_inc = Some(*next);

                        self.buffer.pop(); // remove Entry::Begin
                        nesting -= 1; // Alter nesting as appropriate
                        prev_num = Some(self.buffer.len());
                        self.buffer.push(Entry::Num(0));

                        iter.next(); // Skip next entry
                        iter.next(); // Skip Entry::End
                    }
                }
            }
        }

        debug_assert_eq!(nesting, 0);

        mem::swap(&mut self.expr, &mut self.buffer);
        has_changed
    }

    fn iter_split(&mut self) -> bool {
        self.buffer.clear();

        let mut has_changed = false;

        for entry in &self.expr {
            match entry {
                Entry::Begin => {
                    self.buffer.push(Entry::Begin);
                }
                Entry::End => {
                    self.buffer.push(Entry::End);
                }
                Entry::Num(n) => {
                    if *n >= 10 && !has_changed {
                        has_changed = true;
                        self.buffer.extend([
                            Entry::Begin,
                            Entry::Num(n / 2),
                            Entry::Num(n / 2 + (if n % 2 == 0 { 0 } else { 1 })),
                            Entry::End,
                        ])
                    } else {
                        self.buffer.push(Entry::Num(*n));
                    }
                }
            }
        }

        mem::swap(&mut self.expr, &mut self.buffer);
        has_changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod expression {
        use super::*;

        #[test]
        fn test_add_expr() {
            let parsed = parse::parse("[[1,2],3]\n[4,5]\n[[6,7],8]").unwrap();

            let mut expr = Expression::default();
            expr.join(&parsed[0]);

            let expected = vec![
                Entry::Begin,
                Entry::Begin,
                Entry::Num(1),
                Entry::Num(2),
                Entry::End,
                Entry::Num(3),
                Entry::End,
            ];

            assert_eq!(expr.entries, expected);
            assert_eq!(expr.to_string(), "[[1,2],3]");

            expr.join(&parsed[1]);
            assert_eq!(expr.to_string(), "[[[1,2],3],[4,5]]");

            // Also works with += operator
            expr += &parsed[2];
            assert_eq!(expr.to_string(), "[[[[1,2],3],[4,5]],[[6,7],8]]");
        }

        #[test]
        fn test_display() {
            check("[[1,2],3]");
            check("[[[[[9,8],1],2],3],4]");
            check("[[[[0,9],2],3],4]");
            check("[[3,[2,[8,0]]],[9,[5,[7,0]]]]");
            check("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");

            fn check(input: &str) {
                let parsed = parse::parse(input).unwrap();
                let mut expr = Expression::default();
                expr.join(&parsed[0]);
                assert_eq!(expr.to_string(), input);
            }
        }
    }

    #[test]
    fn test_simplify() {
        let mut simplifier = ExpressionSimplifier::default();

        // Explosion
        let result = simple(&mut simplifier, "[[[[[9,8],1],2],3],4]");
        assert_eq!(result, "[[[[0,9],2],3],4]");
        let result = simple(&mut simplifier, "[7,[6,[5,[4,[3,2]]]]]");
        assert_eq!(result, "[7,[6,[5,[7,0]]]]");
        let result = simple(&mut simplifier, "[[6,[5,[4,[3,2]]]],1]");
        assert_eq!(result, "[[6,[5,[7,0]]],3]");

        // 2x Explosion
        let result = simple(&mut simplifier, "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]");
        assert_eq!(result, "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");

        // 2x Explosion, 2x Split
        let result = simple(&mut simplifier, "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]");
        assert_eq!(result, "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");

        fn simple(simplifier: &mut ExpressionSimplifier, input: &str) -> String {
            let tree = parse::parse(input).unwrap();

            let mut expr = Expression::default();
            expr.join(&tree[0]);

            let expr = simplifier.simplify(expr);
            expr.to_string()
        }
    }

    #[test]
    fn test_sum_all() {
        let sum = sum_from_str("[1,1]\n[2,2]\n[3,3]\n[4,4]");
        assert_eq!(sum, "[[[[1,1],[2,2]],[3,3]],[4,4]]");

        let sum = sum_from_str("[1,1]\n[2,2]\n[3,3]\n[4,4]\n[5,5]");
        assert_eq!(sum, "[[[[3,0],[5,3]],[4,4]],[5,5]]");

        let sum = sum_from_str("[1,1]\n[2,2]\n[3,3]\n[4,4]\n[5,5]\n[6,6]");
        assert_eq!(sum, "[[[[5,0],[7,4]],[5,5]],[6,6]]");

        let example = "\
[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
[7,[5,[[3,8],[1,4]]]]
[[2,[2,2]],[8,[8,1]]]
[2,9]
[1,[[[9,3],9],[[9,0],[0,7]]]]
[[[5,[7,4]],7],1]
[[[[4,2],2],6],[8,7]]";
        let expected = "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]";
        assert_eq!(sum_from_str(example), expected);

        let example = "\
[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";
        let expected = "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]";
        assert_eq!(sum_from_str(example), expected);

        fn sum_from_str(input: &str) -> String {
            sum_many(&parse::parse(input).unwrap()).to_string()
        }
    }

    #[test]
    fn test_magnitude() {
        check("[9,1]", 29);
        check("[1,9]", 21);
        check("[[9,1],[1,9]]", 3 * 29 + 2 * 21);

        check("[[1,2],[[3,4],5]]", 143);
        check("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384);
        check("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445);
        check("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791);
        check("[[[[5,0],[7,4]],[5,5]],[6,6]]", 1137);
        check(
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]",
            3488,
        );

        check(
            "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]",
            4140,
        );

        fn check(input: &str, magnitude: u64) {
            let mut expr = Expression::default();
            expr.join(&parse::parse(input).unwrap()[0]);
            let expected = expr.magnitude();
            assert_eq!(magnitude, expected);
        }
    }

    #[test]
    fn test_max_magnitude_pair() {
        const EXAMPLE: &str = "\
[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";

        let nums = parse::parse(EXAMPLE).unwrap();
        let max = max_magnitude_pair(&nums);
        assert_eq!(max, 3993);
    }
}
