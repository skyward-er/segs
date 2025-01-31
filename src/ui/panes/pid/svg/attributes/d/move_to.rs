use super::DToken;
use nom::{
    branch::alt,
    character::complete::char,
    combinator::map,
    number::complete::float,
    sequence::{preceded, tuple},
    IResult,
};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct MoveTo {
    abs: bool,
    x: f32,
    y: f32,
}

impl MoveTo {
    pub fn abs(x: f32, y: f32) -> DToken {
        DToken::MoveTo(Self { abs: true, x, y })
    }

    pub fn rel(x: f32, y: f32) -> DToken {
        DToken::MoveTo(Self { abs: false, x, y })
    }

    pub(super) fn parse(input: &str) -> IResult<&str, DToken> {
        map(
            tuple((
                alt((map(char('M'), |_| true), map(char('m'), |_| false))),
                preceded(char(' '), float),
                preceded(char(' '), float),
            )),
            |(abs, x, y)| DToken::MoveTo(Self { abs, x, y }),
        )(input)
    }
}

impl Display for MoveTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.abs {
            write!(f, "M {} {}", self.x, self.y)
        } else {
            write!(f, "m {} {}", self.x, self.y)
        }
    }
}
