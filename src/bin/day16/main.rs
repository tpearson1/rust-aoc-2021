use std::{
    fmt::{self, Debug},
    fs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day16/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let packet = Packet::parse(&input).map_err(|e| format!("Failed to parse packet: {}", e))?;
    let version_sum = packet.version_sum();
    println!("Sum of packet versions: {}", version_sum);
    let result = packet.eval();
    println!("Packet evaluates to: {}", result);

    Ok(())
}

// pub type Input<'a> = &'a str;
// pub type Result<'a, O> = nom::IResult<Input<'a>, O, nom::error::VerboseError<Input<'a>>>;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bit {
    L = 0,
    H,
}

impl Debug for Bit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::L => write!(f, "0"),
            Self::H => write!(f, "1"),
        }
    }
}

use Bit::{H, L};

impl Bit {
    #[inline]
    fn from(num: u8, bit: u8) -> Self {
        if (num & (1 << bit)) != 0 {
            H
        } else {
            L
        }
    }

    #[inline]
    fn as_num(bits: impl IntoIterator<Item = Bit>) -> u64 {
        let mut num: u64 = 0;
        for bit in bits.into_iter() {
            num *= 2;
            num += bit as u64
        }
        num
    }
}

fn to_bits(input: &str) -> Option<Vec<Bit>> {
    let result = input
        .trim()
        .chars()
        .map(|c| Some(c.to_digit(16)? as u8))
        .collect::<Option<Vec<_>>>()?
        .iter()
        .flat_map(|&nibble| {
            [
                Bit::from(nibble, 3),
                Bit::from(nibble, 2),
                Bit::from(nibble, 1),
                Bit::from(nibble, 0),
            ]
        })
        .collect();
    Some(result)
}

#[derive(Debug)]
pub struct Packet {
    pub version: u8,
    pub contents: PacketContents,
}

#[derive(Debug)]
pub enum PacketContents {
    Literal(u64),
    Operator {
        ty: OperatorType,
        subpackets: Vec<Packet>,
    },
}

#[derive(Debug)]
pub enum OperatorType {
    Sum,
    Product,
    Min,
    Max,
    GreaterThan,
    LessThan,
    Equal,
}

impl OperatorType {
    // Internal - panics on unexepcted input
    fn from_type_id(id: u8) -> OperatorType {
        use OperatorType::*;
        match id {
            0 => Sum,
            1 => Product,
            2 => Min,
            3 => Max,
            5 => GreaterThan,
            6 => LessThan,
            7 => Equal,
            _ => panic!("Unexpected type id: {}", id),
        }
    }

    pub fn binary_op(&self) -> bool {
        use OperatorType::*;
        matches!(self, GreaterThan | LessThan | Equal)
    }
}

impl Packet {
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        let bits = to_bits(input).ok_or("invalid hex input")?;
        Self::parse_bits(bits)
    }

    fn parse_bits(bits: impl IntoIterator<Item = Bit>) -> Result<Self, &'static str> {
        let (_, packet) = Self::parse_helper(&mut bits.into_iter().enumerate().into())?;
        Ok(packet)
    }

    fn parse_helper(
        bits: &mut CountingIter<impl Iterator<Item = (usize, Bit)>>,
    ) -> Result<(usize, Packet), &'static str> {
        let start_idx = bits.processed;

        let version = Bit::as_num(bits.by_ref().map(|x| x.1).take(3)) as u8;
        let type_id = Bit::as_num(bits.by_ref().map(|x| x.1).take(3)) as u8;

        let contents = match type_id {
            4 => {
                // Literal value (single binary number)

                let mut num = 0;
                loop {
                    let last_group = bits.next().ok_or("end of input reading literal")?.1 == L;

                    let nibble = Bit::as_num(bits.by_ref().map(|x| x.1).take(4));
                    num <<= 4;
                    num |= nibble;

                    if last_group {
                        break;
                    }
                }

                PacketContents::Literal(num)
            }
            _ => {
                // Operator
                let (_, length_type_id) = bits.next().ok_or("end of input reading length type")?;
                let subpackets = match length_type_id {
                    L => {
                        // Next 15 bits are number representing total length in
                        // bits of the subpackets contained by this packet
                        let total_length = Bit::as_num(bits.by_ref().map(|x| x.1).take(15));

                        let mut parsed_bits = 0;
                        let mut packets = Vec::new();
                        while parsed_bits < total_length as usize {
                            let (parsed, packet) = Self::parse_helper(bits.by_ref())?;
                            parsed_bits += parsed;
                            packets.push(packet);
                        }
                        packets
                    }
                    H => {
                        // Next 11 bits are number representing number of
                        // sub-packets contained by this packet
                        let num_subpackets = Bit::as_num(bits.by_ref().map(|x| x.1).take(11));
                        let mut packets = Vec::with_capacity(num_subpackets as usize);
                        for _ in 0..num_subpackets {
                            packets.push(Self::parse_helper(bits.by_ref())?.1);
                        }
                        packets
                    }
                };

                // `type_id` is a 3 bit value, and the below function accepts
                // 0..=7 without panicking except for 4, which is handled in
                // another match branch
                let ty = OperatorType::from_type_id(type_id);

                if ty.binary_op() && subpackets.len() != 2 {
                    return Err("binary operator does not contain exactly two supackets");
                }

                if subpackets.is_empty() {
                    // To provide some guarantees in `eval`, we require all
                    // operators to have at least one subpacket
                    return Err("operator has no supbackets");
                }

                PacketContents::Operator { ty, subpackets }
            }
        };

        let packet = Packet { version, contents };

        let end_idx = bits.processed;
        Ok((end_idx - start_idx, packet))
    }

    pub fn version_sum(&self) -> u64 {
        let child_sum = match &self.contents {
            PacketContents::Literal(_) => 0,
            PacketContents::Operator { subpackets, .. } => {
                subpackets.iter().map(|x| x.version_sum()).sum()
            }
        };
        self.version as u64 + child_sum
    }

    pub fn eval(&self) -> u64 {
        match &self.contents {
            PacketContents::Literal(value) => *value,
            PacketContents::Operator { ty, subpackets } => {
                use OperatorType::*;
                let ops = subpackets.iter().map(|x| x.eval());
                match ty {
                    Sum => ops.sum(),
                    Product => ops.product(),
                    Min => ops.min().unwrap(), // we guarantee at least one subpacket
                    Max => ops.max().unwrap(),
                    GreaterThan => (subpackets[0].eval() > subpackets[1].eval()) as u64,
                    LessThan => (subpackets[0].eval() < subpackets[1].eval()) as u64,
                    Equal => (subpackets[0].eval() == subpackets[1].eval()) as u64,
                }
            }
        }
    }
}

pub struct CountingIter<I> {
    iter: I,
    processed: usize,
}

impl<I: Iterator> Iterator for CountingIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.processed += 1;
        self.iter.next()
    }
}

impl<I: Iterator> From<I> for CountingIter<I> {
    fn from(iter: I) -> Self {
        Self { iter, processed: 0 }
    }
}

impl<I> CountingIter<I> {
    pub fn processed(&self) -> usize {
        self.processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_bits() {
        let result = to_bits("1F3\n").unwrap();
        assert_eq!(result, [L, L, L, H, H, H, H, H, L, L, H, H]);
    }

    const EXAMPLE_BITS: &str = "11101110000000001101010000001100100000100011000001100000";

    #[test]
    fn test_parse_bits() {
        let bits = EXAMPLE_BITS.chars().map(|c| if c == '1' { H } else { L });
        let packet = Packet::parse_bits(bits).unwrap();
        assert_eq!(packet.version_sum(), 7 + 2 + 4 + 1);
    }

    #[test]
    fn test_eval() {
        let result = Packet::parse("C200B40A82\n").unwrap().eval();
        assert_eq!(result, 3);
        let result = Packet::parse("04005AC33890").unwrap().eval();
        assert_eq!(result, 54);
        let result = Packet::parse("880086C3E88112").unwrap().eval();
        assert_eq!(result, 7);
        let result = Packet::parse("CE00C43D881120").unwrap().eval();
        assert_eq!(result, 9);
        let result = Packet::parse("D8005AC2A8F0").unwrap().eval();
        assert_eq!(result, 1);
        let result = Packet::parse("F600BC2D8F").unwrap().eval();
        assert_eq!(result, 0);
        let result = Packet::parse("9C005AC2F8F0").unwrap().eval();
        assert_eq!(result, 0);
        let result = Packet::parse("9C0141080250320F1802104A08").unwrap().eval();
        assert_eq!(result, 1);
    }
}
