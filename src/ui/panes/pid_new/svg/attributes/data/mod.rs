pub mod close_path;
pub mod ellicptical_arc;
pub mod horizonta_line_to;
pub mod line_to;
pub mod move_to;
pub mod vertical_line_to;

use std::fmt::Display;

use self::{
    close_path::ClosePath, ellicptical_arc::EllipticalArc, horizonta_line_to::HorizontalLineTo,
    line_to::LineTo, move_to::MoveTo, vertical_line_to::VerticalLineTo,
};
use nom::{branch::alt, character::complete::char, multi::separated_list0, IResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data {
    pub segments: Vec<DToken>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DToken {
    MoveTo(MoveTo),
    LineTo(LineTo),
    HorizontalLineTo(HorizontalLineTo),
    VerticalLineTo(VerticalLineTo),
    EllipticalArc(EllipticalArc),
    ClosePath(ClosePath),
}

impl Display for DToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MoveTo(t) => t.fmt(f),
            Self::LineTo(t) => t.fmt(f),
            Self::HorizontalLineTo(t) => t.fmt(f),
            Self::VerticalLineTo(t) => t.fmt(f),
            Self::EllipticalArc(t) => t.fmt(f),
            Self::ClosePath(t) => t.fmt(f),
        }
    }
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ")
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;
        let d = separated_list0(char(' '), DToken::parse)(input.as_str())
            .map(|(_, segments)| Self { segments })
            .unwrap_or_default();
        Ok(d)
    }
}

impl DToken {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            MoveTo::parse,
            LineTo::parse,
            HorizontalLineTo::parse,
            VerticalLineTo::parse,
            ClosePath::parse,
            EllipticalArc::parse,
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        #[serde(rename = "@d")]
        d: Data,
    }

    #[test]
    fn it_serialize() {
        let test = Test {
            d: Data {
                segments: vec![
                    MoveTo::abs(-3.12, 1.0),
                    LineTo::rel(4.0, -4.0),
                    MoveTo::rel(-1.0, 1.0),
                    LineTo::abs(5.0, 0.0),
                ],
            },
        };
        let expected = "<test d=\"M -3.12 1 l 4 -4 m -1 1 L 5 0\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("test")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_serialize_default() {
        let test = Test { d: Data::default() };
        let expected = "<test d=\"\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("test")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_deserialize() {
        let test =
            "<test d=\"M 0.5 0 V 6 M 1.5 0 v 1 a 2 2 0 1 1 0 4 v 1 M 0 3 h 0.5 M 3.5 3 h 0.5\"/>";
        let expected = Test {
            d: Data {
                segments: vec![
                    MoveTo::abs(0.5, 0.0),
                    VerticalLineTo::abs(6.0),
                    MoveTo::abs(1.5, 0.0),
                    VerticalLineTo::rel(1.0),
                    EllipticalArc::rel(2.0, 2.0, 0.0, true, true, 0.0, 4.0),
                    VerticalLineTo::rel(1.0),
                    MoveTo::abs(0.0, 3.0),
                    HorizontalLineTo::rel(0.5),
                    MoveTo::abs(3.5, 3.0),
                    HorizontalLineTo::rel(0.5),
                ],
            },
        };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Test::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn it_deserialize_default() {
        let test = "<test d=\"\"/>";
        let expected = Test { d: Data::default() };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Test::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }
}
