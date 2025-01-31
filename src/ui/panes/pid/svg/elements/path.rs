use crate::ui::panes::pid::svg::{
    attributes::{
        d::{DToken, D},
        style::Style,
        transform::Transform,
    },
    utils::{is_default, is_zero},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct Path {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@width")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub width: f32,

    #[serde(rename = "@height")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub height: f32,

    #[serde(rename = "@d")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub d: D,

    #[serde(rename = "@style")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub style: Style,

    #[serde(rename = "@transfrom")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub transform: Transform,
}

impl Path {
    pub fn push_segment(&mut self, segment: DToken) {
        self.d.segments.push(segment);
        // self.update_size();
    }

    // fn update_size(&mut self) {
    //     let size = self
    //         .d
    //         .segments
    //         .iter()
    //         .map(|s| s.to_vec2())
    //         .fold(Vec2::new(self.width, self.height), Vec2::max);
    //     self.width = size.x + self.style.stroke_width / 2.0;
    //     self.height = size.y + self.style.stroke_width / 2.0;
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialize() {
        let test = Path {
            id: "paolino".to_string(),
            width: 23.4,
            height: 5.0,
            d: D::default(),
            style: Style::default(),
            transform: Transform::default(),
        };
        let expected = "<path id=\"paolino\" width=\"23.4\" height=\"5\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("path")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_serialize_default() {
        let test = Path::default();
        let expected = "<path id=\"\"/>";

        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("path")).unwrap();
        test.serialize(ser).unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn it_deserialize() {
        let test = "<test id=\"pepperoncino\" width=\"11\" height=\"24.5\"/>";
        let expected = Path {
            id: "pepperoncino".to_string(),
            width: 11.0,
            height: 24.5,
            d: D::default(),
            style: Style::default(),
            transform: Transform::default(),
        };

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Path::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn it_deserialize_default() {
        let test = "<path id=\"\"/>";
        let expected = Path::default();

        let mut des = quick_xml::de::Deserializer::from_str(test);
        let deserialized = Path::deserialize(&mut des).unwrap();
        assert_eq!(deserialized, expected);
    }
}
