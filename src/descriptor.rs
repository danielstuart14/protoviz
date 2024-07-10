use hex_color::HexColor;
use serde::{Deserialize, Serialize};

/// Enum to hold the length of a field
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum FieldLength {
    Fixed(usize),
    Variable(String),
}

impl ToString for FieldLength {
    fn to_string(&self) -> String {
        match self {
            FieldLength::Fixed(length) => length.to_string(),
            FieldLength::Variable(name) => name.clone(),
        }
    }
}

/// Struct to hold the options for a field
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldDescriptor {
    pub name: String,
    pub length: FieldLength,
    #[serde(default)]
    pub wrap: bool, // Whether to wrap at the end of the field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<HexColor>, // Color of the field
}

/// Struct to hold the options for the image elements
#[derive(Debug, Deserialize, Serialize)]
pub struct ElementsDescriptor {
    #[serde(default = "default_true", alias = "is_network")]
    pub network_order: bool, // Whether it is a network protocol (big endian)
    #[serde(default = "default_true", alias = "position")]
    pub field_position: bool, // Whether to show the position of the fields
    #[serde(default = "default_true", alias = "length")]
    pub field_length: bool, // Whether to show the length of the fields
    #[serde(default = "default_true")]
    pub wrap_line: bool, // Whether to show the wrap line
    #[serde(default = "default_true")]
    pub start_symbol: bool, // Whether to show the wrap line
}

impl Default for ElementsDescriptor {
    fn default() -> Self {
        Self {
            network_order: true,
            field_position: true,
            field_length: true,
            wrap_line: true,
            start_symbol: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Struct to hold the options for the image style
#[derive(Debug, Deserialize, Serialize)]
pub struct StyleDescriptor {
    #[serde(default = "default_white")]
    pub background_color: HexColor, // Background color of the image
    #[serde(default = "default_white")]
    pub field_color: HexColor, // Color of the field background
    #[serde(default = "default_black")]
    pub text_color: HexColor, // Text color of the image (field names + stroke)
    #[serde(default = "default_black")]
    pub subtitle_color: HexColor, // Color of the subtitle text (field length and position)
}

impl Default for StyleDescriptor {
    fn default() -> Self {
        Self {
            background_color: default_white(),
            field_color: default_white(),
            text_color: default_black(),
            subtitle_color: default_black(),
        }
    }
}

fn default_white() -> HexColor {
    HexColor::rgb(255, 255, 255)
}

fn default_black() -> HexColor {
    HexColor::rgb(0, 0, 0)
}

/// Struct to hold the options for a protocol
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ProtoDescriptor {
    #[serde(default)]
    pub elements: ElementsDescriptor, // Options for the image elements
    #[serde(default)]
    pub style: StyleDescriptor, // Options for the image style
    pub fields: Vec<FieldDescriptor>, // List of fields the protocol contains
}