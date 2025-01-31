use super::super::{
    attributes::{data::Data, style::Style, transform::Transform},
    utils::{is_default, is_zero},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug, Default, Clone)]
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
    pub data: Data,

    #[serde(rename = "@style")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub style: Style,

    #[serde(rename = "@transfrom")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub transform: Transform,
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
            data: Data::default(),
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
            data: Data::default(),
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
