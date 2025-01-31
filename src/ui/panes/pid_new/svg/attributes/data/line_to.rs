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
pub struct LineTo {
    abs: bool,
    x: f32,
    y: f32,
}

impl LineTo {
    pub fn abs(x: f32, y: f32) -> DToken {
        DToken::LineTo(Self { abs: true, x, y })
    }

    pub fn rel(x: f32, y: f32) -> DToken {
        DToken::LineTo(Self { abs: false, x, y })
    }

    pub(super) fn parse(input: &str) -> IResult<&str, DToken> {
        map(
            tuple((
                alt((map(char('L'), |_| true), map(char('l'), |_| false))),
                preceded(char(' '), float),
                preceded(char(' '), float),
            )),
            |(abs, x, y)| DToken::LineTo(Self { abs, x, y }),
        )(input)
    }
}

impl Display for LineTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.abs {
            write!(f, "L {} {}", self.x, self.y)
        } else {
            write!(f, "l {} {}", self.x, self.y)
        }
    }
}
