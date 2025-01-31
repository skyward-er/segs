use egui::emath::Rot2;
use egui::{Pos2, Vec2};
use nom::IResult;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::map,
    number::complete::float,
    sequence::{delimited, preceded, tuple},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::string::ToString;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transform {
    pub rotate: Rotate,
    pub translate: Translate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rotate {
    pub angle: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Translate {
    pub x: f32,
    pub y: f32,
}

impl Transform {
    pub fn to_local_frame(&self, pos: Pos2) -> Pos2 {
        let pos = pos.to_vec2();

        // Apply the rotation
        let rot = Rot2::from_angle(-self.rotate.angle);
        let pos = self.rotate.to_vec2() + rot * (pos - self.rotate.to_vec2());

        // Apply the translation
        let pos = pos - self.translate.to_vec2();

        pos.to_pos2()
    }
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut transform = String::new();
        if self.rotate.angle != 0.0 {
            transform.push_str(&self.rotate.to_string());
        }
        if self.translate.x != 0.0 && self.translate.y != 0.0 {
            transform.push_str(&self.translate.to_string());
        }
        transform.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;

        let mut transform = Transform::default();
        let input = if let Ok((input, rotate)) = Rotate::parse(&input) {
            transform.rotate = rotate;
            input
        } else {
            &input
        };
        if let Ok((_, translate)) = Translate::parse(input) {
            transform.translate = translate;
        }

        Ok(transform)
    }
}

impl Rotate {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(
                delimited(tag("rotate("), preceded(char(' '), float), char(')')),
                |angle| Self {
                    angle,
                    x: 0.0,
                    y: 0.0,
                },
            ),
            map(
                delimited(
                    tag("rotate("),
                    tuple((
                        float,
                        preceded(char(' '), float),
                        preceded(char(' '), float),
                    )),
                    char(')'),
                ),
                |(angle, x, y)| Self { angle, x, y },
            ),
        ))(input)
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Display for Rotate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.x == 0.0 && self.y == 0.0 {
            write!(f, "rotate({})", self.angle)
        } else {
            write!(f, "rotate({} {} {})", self.angle, self.x, self.y)
        }
    }
}

impl Translate {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            delimited(
                tag("translate("),
                tuple((float, preceded(char(' '), float))),
                char(')'),
            ),
            |(x, y)| Self { x, y },
        )(input)
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Display for Translate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "translate({} {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        #[serde(rename = "@transform")]
        transform: Transform,
    }

    #[test]
    fn it_serialize() {
        let test = Test {
            transform: Transform {
                rotate: Rotate {
                    angle: 3.15,
                    x: 7.0,
                    y: -1.23,
                },
                translate: Translate { x: 4.0, y: -1.4 },
            },
        };
        let expected = "<test transform=\"rotate(3.15 7 -1.23)translate(4 -1.4)\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("test")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_serialize_default() {
        let test = Test {
            transform: Transform::default(),
        };
        let expected = "<test transform=\"\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("test")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_deserialize() {
        let test = "<test transform=\"rotate(45.7 -43.21 89)translate(4 -1.4)\"/>";
        let expected = Test {
            transform: Transform {
                rotate: Rotate {
                    angle: 45.7,
                    x: -43.21,
                    y: 89.0,
                },
                translate: Translate { x: 4.0, y: -1.4 },
            },
        };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Test::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn it_deserialize_default() {
        let test = "<test transform=\"\"/>";
        let expected = Test {
            transform: Transform::default(),
        };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Test::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }
}
