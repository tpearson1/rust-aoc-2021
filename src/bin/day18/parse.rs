#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnailfishNum {
    Pair(Box<[SnailfishNum; 2]>),
    Num(u32),
}

pub fn parse(input: &str) -> Result<Vec<SnailfishNum>, &'static str> {
    let (_, nums) = snailfish_nums(input).map_err(|_| "Failed to parse")?;
    Ok(nums)
}

type ParseInput<'a> = &'a str;
type ParseResult<'a, O> = nom::IResult<ParseInput<'a>, O, nom::error::VerboseError<ParseInput<'a>>>;

fn snailfish_nums(i: ParseInput<'_>) -> ParseResult<'_, Vec<SnailfishNum>> {
    use nom::{character::complete::line_ending, multi::separated_list1};
    separated_list1(line_ending, snailfish_num)(i)
}

fn snailfish_num(i: ParseInput<'_>) -> ParseResult<'_, SnailfishNum> {
    use nom::{branch::alt, character::complete::char, combinator::map, sequence::tuple};
    alt((
        regular_num,
        map(
            tuple((
                char('['),
                snailfish_num,
                char(','),
                snailfish_num,
                char(']'),
            )),
            |(_, fst, _, snd, _)| SnailfishNum::Pair(Box::new([fst, snd])),
        ),
    ))(i)
}

fn regular_num(i: ParseInput<'_>) -> ParseResult<'_, SnailfishNum> {
    use nom::{character::complete::u32, combinator::map};
    map(u32, SnailfishNum::Num)(i)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        use super::{parse, SnailfishNum::*};

        let result = parse("[1,2]\n").unwrap();
        assert_eq!(result, vec![Pair(Box::new([Num(1), Num(2)]))]);

        let result = parse("[[1,2],3]").unwrap();
        let expected = vec![Pair(Box::new([Pair(Box::new([Num(1), Num(2)])), Num(3)]))];
        assert_eq!(result, expected);

        let result = parse("[9,[8,7]]").unwrap();
        let expected = vec![Pair(Box::new([Num(9), Pair(Box::new([Num(8), Num(7)]))]))];
        assert_eq!(result, expected);

        let result = parse("[[1,9],[8,5]]\n").unwrap();
        let expected = vec![Pair(Box::new([
            Pair(Box::new([Num(1), Num(9)])),
            Pair(Box::new([Num(8), Num(5)])),
        ]))];
        assert_eq!(result, expected);

        let result = parse("[1,2]\n[3,4]").unwrap();
        assert_eq!(
            result,
            vec![
                Pair(Box::new([Num(1), Num(2)])),
                Pair(Box::new([Num(3), Num(4)]))
            ]
        );
    }
}
