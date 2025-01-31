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
pub struct VerticalLineTo {
    abs: bool,
    y: f32,
}

impl VerticalLineTo {
    pub fn abs(y: f32) -> DToken {
        DToken::VerticalLineTo(Self { abs: true, y })
    }

    pub fn rel(y: f32) -> DToken {
        DToken::VerticalLineTo(Self { abs: false, y })
    }

    pub(super) fn parse(input: &str) -> IResult<&str, DToken> {
        map(
            tuple((
                alt((map(char('V'), |_| true), map(char('v'), |_| false))),
                preceded(char(' '), float),
            )),
            |(abs, y)| DToken::VerticalLineTo(Self { abs, y }),
        )(input)
    }
}

impl Display for VerticalLineTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.abs {
            write!(f, "V {}", self.y)
        } else {
            write!(f, "v {}", self.y)
        }
    }
}
