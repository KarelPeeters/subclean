use std::fmt::{Display, Formatter};
use itertools::Itertools;

#[derive(Debug)]
pub struct Subtitle {
    pub blocks: Vec<SubBlock>,
}

impl Subtitle {
    pub fn parse(str: &str) -> Result<Subtitle, nom::Err<(String, nom::error::ErrorKind)>> {
        let str = str.trim_start().trim_start_matches('\u{feff}');
        parse::subtitle(str).map_err(|e| e.map(|(s, k)| (s.chars().take(20).join(""), k)))
    }
}

impl Display for Subtitle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, block) in self.blocks.iter().enumerate() {
            write!(f, "{}\n{}\n\n", i + 1, block)?
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SubBlock {
    pub start: TimePoint,
    pub end: TimePoint,
    pub text: String,
}

impl Display for SubBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} --> {}\n{}", self.start, self.end, self.text)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TimePoint { ms: u64 }

impl TimePoint {
    fn from_components(h: u64, m: u64, s: u64, ms: u64) -> TimePoint {
        TimePoint { ms: ms + 1000 * s + 60 * 1000 * m + 60 * 60 * 1000 * h }
    }
}

impl Display for TimePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ms = self.ms % 1000;
        let s = (self.ms / 1000) % 60;
        let m = (self.ms / (60 * 1000)) % 60;
        let h = (self.ms / (60 * 60 * 1000)) % 60;
        write!(f, "{:02}:{:02}:{:02},{:03}", h, m, s, ms)
    }
}

mod parse {
    use nom::{Err, InputLength, IResult};
    use nom::bytes::complete::{tag, take_until};
    use nom::character::complete::{digit1, newline};
    use nom::combinator::{map, map_res};
    use nom::error::{ErrorKind, ParseError};
    use nom::multi::many_till;
    use nom::sequence::{pair, tuple};

    use crate::srt::{SubBlock, Subtitle, TimePoint};

    pub fn subtitle(r: &str) -> Result<Subtitle, Err<(&str, ErrorKind)>> {
        many_till(sub_block, eof)(r).map(|(_, (blocks, _))| Subtitle { blocks })
    }

    fn sub_block(r: &str) -> IResult<&str, SubBlock> {
        map(tuple((number, newline, time_point, tag(" --> "), time_point, newline, text)),
            |(_number, _, start, _, end, _, text)| SubBlock { start, end, text })(r)
    }

    fn text(r: &str) -> IResult<&str, String> {
        map(pair(take_until("\n\n"), tag("\n\n")), |(s, _): (&str, _)| s.to_owned())(r)
    }

    fn time_point(r: &str) -> IResult<&str, TimePoint> {
        map(tuple((number, tag(":"), number, tag(":"), number, tag(","), number)),
            |(h, _, m, _, s, _, ms)| TimePoint::from_components(h, m, s, ms))(r)
    }

    fn number(r: &str) -> IResult<&str, u64> {
        map_res(digit1, |s: &str| s.parse())(r)
    }

    //missing in nom
    fn eof<I: InputLength + Copy, E: ParseError<I>>(input: I) -> IResult<I, I, E> {
        if input.input_len() == 0 {
            Ok((input, input))
        } else {
            Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
        }
    }
}