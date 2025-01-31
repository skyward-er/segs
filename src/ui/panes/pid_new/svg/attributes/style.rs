use std::fmt::Display;

use egui::Color32;
use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take,
    character::complete::char,
    combinator::{map, opt},
    number::complete::float,
    sequence::{preceded, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub fill: Color32,
    pub stroke: Color32,
    pub stroke_opacity: f32,
    pub stroke_width: f32,
    pub stroke_linejoin: LineJoin,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineJoin {
    Bevel,
    Miter,
    Round,
}

impl Style {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                opt(map(preceded(tag("fill:"), take(9usize)), Color32::from_hex)),
                opt(char(';')),
                opt(map(
                    preceded(tag("stroke:"), take(9usize)),
                    Color32::from_hex,
                )),
                opt(char(';')),
                opt(preceded(tag("stroke-opacity:"), float)),
                opt(char(';')),
                opt(preceded(tag("stroke-width:"), float)),
                opt(char(';')),
                opt(preceded(tag("stroke-linejoin:"), LineJoin::parse)),
            )),
            |(fill, _, stroke, _, stroke_opacity, _, stroke_width, _, stroke_linejoin)| {
                let fill = fill.and_then(|c| c.ok()).unwrap_or(Color32::BLACK);
                let stroke = stroke.and_then(|c| c.ok()).unwrap_or(Color32::TRANSPARENT);
                Self {
                    fill,
                    stroke,
                    stroke_opacity: stroke_opacity.unwrap_or(1.0),
                    stroke_width: stroke_width.unwrap_or(1.0),
                    stroke_linejoin: stroke_linejoin.unwrap_or_default(),
                }
            },
        )(input)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Color32::BLACK,
            stroke: Color32::TRANSPARENT,
            stroke_opacity: 1.0,
            stroke_width: 1.0,
            stroke_linejoin: LineJoin::Miter,
        }
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut style = Vec::new();

        if self.fill != Color32::BLACK {
            style.push(format!("fill:{}", self.fill.to_hex()));
        }
        if self.stroke != Color32::TRANSPARENT {
            style.push(format!("stroke:{}", self.stroke.to_hex()));
        }
        if self.stroke_opacity != 1.0 {
            style.push(format!("stroke-opacity:{}", self.stroke_opacity));
        }
        if self.stroke_width != 1.0 {
            style.push(format!("stroke-width:{}", self.stroke_width));
        }
        if self.stroke_linejoin != LineJoin::Miter {
            style.push(format!("stroke-linejoin:{}", self.stroke_linejoin));
        }

        style.join(";").serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;
        Ok(Style::parse(input.as_str())
            .map(|(_, style)| style)
            .unwrap_or_default())
    }
}

impl LineJoin {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(tag("bevel"), |_| Self::Bevel),
            map(tag("miter"), |_| Self::Miter),
            map(tag("round"), |_| Self::Round),
        ))(input)
    }
}

impl Default for LineJoin {
    fn default() -> Self {
        Self::Miter
    }
}

impl Display for LineJoin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bevel => write!(f, "bevel"),
            Self::Miter => write!(f, "miter"),
            Self::Round => write!(f, "round"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        #[serde(rename = "@style")]
        style: Style,
    }

    #[test]
    fn it_serialize() {
        let test = Test {
            style: Style {
                fill: Color32::from_rgb(23, 45, 76),
                stroke: Color32::from_rgb(123, 42, 29),
                stroke_opacity: 0.5,
                stroke_width: 3.21,
                stroke_linejoin: LineJoin::Bevel,
            },
        };
        let expected = "<test style=\"fill:#172d4cff;stroke:#7b2a1dff;stroke-opacity:0.5;stroke-width:3.21;stroke-linejoin:bevel\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("test")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_serialize_default() {
        let test = Test {
            style: Style::default(),
        };
        let expected = "<test style=\"\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("test")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_deserialize() {
        let test  = "<test style=\"fill:#43516fff;stroke:#001831ff;stroke-opacity:0.741;stroke-width:11.5;stroke-linejoin:round\"/>";
        let expected = Test {
            style: Style {
                fill: Color32::from_rgb(67, 81, 111),
                stroke: Color32::from_rgb(0, 24, 49),
                stroke_opacity: 0.741,
                stroke_width: 11.5,
                stroke_linejoin: LineJoin::Round,
            },
        };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Test::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn it_deserialize_default() {
        let test = "<test style=\"\"/>";
        let expected = Test {
            style: Style::default(),
        };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Test::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }
}
