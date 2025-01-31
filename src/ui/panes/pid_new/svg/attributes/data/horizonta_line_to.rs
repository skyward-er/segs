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
pub struct HorizontalLineTo {
    abs: bool,
    x: f32,
}

impl HorizontalLineTo {
    pub fn abs(x: f32) -> DToken {
        DToken::HorizontalLineTo(Self { abs: true, x })
    }

    pub fn rel(x: f32) -> DToken {
        DToken::HorizontalLineTo(Self { abs: false, x })
    }

    pub(super) fn parse(input: &str) -> IResult<&str, DToken> {
        map(
            tuple((
                alt((map(char('H'), |_| true), map(char('h'), |_| false))),
                preceded(char(' '), float),
            )),
            |(abs, x)| DToken::HorizontalLineTo(Self { abs, x }),
        )(input)
    }
}

impl Display for HorizontalLineTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.abs {
            write!(f, "H {}", self.x)
        } else {
            write!(f, "h {}", self.x)
        }
    }
}
