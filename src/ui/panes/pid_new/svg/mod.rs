pub mod attributes;
pub mod elements;
pub mod utils;

use attributes::data::{
    close_path::ClosePath, horizonta_line_to::HorizontalLineTo, line_to::LineTo, move_to::MoveTo,
    vertical_line_to::VerticalLineTo, Data,
};
use attributes::style::{LineJoin, Style};
use attributes::transform::Transform;
use egui::Color32;
use elements::path::Path;
use elements::{defs::Defs, use_node::Use};
use serde::{Deserialize, Serialize};
use utils::{is_default, is_zero};

use super::{Element, Pid3};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Svg {
    #[serde(rename = "@width")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub width: f32,

    #[serde(rename = "@height")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub height: f32,

    #[serde(rename = "@version")]
    pub version: f32, // 1.1

    #[serde(rename = "@xmlns")]
    pub xmlns: String, // http://www.w3.org/2000/svg

    #[serde(rename = "defs")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub defs: Defs,

    #[serde(rename = "use")]
    #[serde(default)]
    pub uses: Vec<Use>,
}

impl Default for Svg {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            version: 1.1,
            xmlns: "http://www.w3.org/2000/svg".to_string(),
            defs: Defs::default(),
            uses: Vec::new(),
        }
    }
}

impl From<&Pid3> for Svg {
    fn from(pid: &Pid3) -> Self {
        let mut svg = Self {
            // TODO: Compute size
            width: 10.0,
            height: 10.0,
            ..Default::default()
        };

        pid.elements.iter().for_each(|(id, element)| match element {
            Element::Icon => {
                svg.defs.paths.push(Path {
                    id: id.clone(),
                    width: 4.0,
                    height: 4.0,
                    data: Data {
                        segments: vec![
                            MoveTo::abs(0.7, 2.0),
                            LineTo::rel(2.6, -1.5),
                            VerticalLineTo::rel(3.0),
                            ClosePath::token(),
                            MoveTo::abs(0.0, 2.0),
                            HorizontalLineTo::rel(4.0),
                        ],
                    },
                    style: Style {
                        stroke: Color32::BLACK,
                        stroke_width: 0.2,
                        stroke_linejoin: LineJoin::Round,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            Element::Label => {}
        });

        pid.references
            .iter()
            .flat_map(|r| Some(r).zip(pid.elements.get(&r.id)))
            .for_each(|(r, _)| {
                svg.uses.push(Use {
                    href: r.id.clone(),
                    width: 4.0,
                    height: 4.0,
                    transform: Transform::default(),
                });
            });

        svg
    }
}

#[cfg(test)]
mod tests {

    use super::{
        attributes::{
            data::ellicptical_arc::EllipticalArc,
            transform::{Rotate, Transform, Translate},
        },
        elements::text::Text,
        *,
    };

    fn test_svg() -> Svg {
        Svg {
            width: 14.0,
            height: 17.196152,
            defs: Defs {
                paths: vec![
                    Path {
                        id: "arrow".to_string(),
                        width: 4.0,
                        height: 4.0,
                        data: Data {
                            segments: vec![
                                MoveTo::abs(0.7, 2.0),
                                LineTo::rel(2.6, -1.5),
                                VerticalLineTo::rel(3.0),
                                ClosePath::token(),
                                MoveTo::abs(0.0, 2.0),
                                HorizontalLineTo::rel(4.0),
                            ],
                        },
                        style: Style {
                            stroke: Color32::BLACK,
                            stroke_width: 0.2,
                            stroke_linejoin: LineJoin::Round,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Path {
                        id: "burst_disk".to_string(),
                        width: 4.0,
                        height: 6.0,
                        data: Data {
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
                        style: Style {
                            stroke: Color32::BLACK,
                            fill: Color32::TRANSPARENT,
                            stroke_width: 0.2,
                            stroke_linejoin: LineJoin::Round,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ],
                texts: vec![Text {
                    font_size: 2.0,
                    font_family: "monospace".to_string(),
                    transform: Transform {
                        translate: Translate { x: 1.0, y: 3.0 },
                        ..Default::default()
                    },
                    format: "{:.2f}".to_string(),
                    text: "Hi mom!".to_string(),
                }],
            },
            uses: vec![
                Use {
                    href: "arrow".to_string(),
                    width: 4.0,
                    height: 4.0,
                    transform: Transform {
                        rotate: Rotate {
                            angle: 180.0,
                            x: 7.0,
                            y: 8.0,
                        },
                        translate: Translate { x: 5.0, y: 6.0 },
                    },
                },
                Use {
                    href: "burst_disk".to_string(),
                    width: 4.0,
                    height: 6.0,
                    transform: Transform {
                        translate: Translate { x: 1.0, y: 5.0 },
                        ..Default::default()
                    },
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn it_serialize() {
        let test = test_svg();
        let expected =
            String::from_utf8(std::fs::read("test_assets/simple_pid.svg").unwrap()).unwrap();

        let mut serialized = String::new();
        let mut ser = quick_xml::se::Serializer::with_root(&mut serialized, Some("svg")).unwrap();
        ser.indent(' ', 4);
        test.serialize(ser).unwrap();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn it_serialize_default() {
        let test = Svg::default();
        let expected = "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\"/>";

        let mut serialized = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut serialized, Some("svg")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn it_deserializes() {
        let test = String::from_utf8(std::fs::read("test_assets/simple_pid.svg").unwrap()).unwrap();
        let expected = test_svg();

        let mut des = quick_xml::de::Deserializer::from_str(&test);
        let deserialized = Svg::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn it_deserialize_default() {
        let svg = "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\"/>";
        let expected = Svg::default();

        let mut des = quick_xml::de::Deserializer::from_str(svg);
        let deserialized = Svg::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }
}
