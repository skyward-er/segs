use super::DToken;
use nom::{branch::alt, character::complete::char, combinator::map, IResult};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct ClosePath {}

impl ClosePath {
    pub fn token() -> DToken {
        DToken::ClosePath(Self {})
    }

    pub(super) fn parse(input: &str) -> IResult<&str, DToken> {
        map(alt((char('Z'), char('z'))), |_| DToken::ClosePath(Self {}))(input)
    }
}

impl Display for ClosePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Z")
    }
}
