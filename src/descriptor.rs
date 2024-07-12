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
    /// Whether it is a network protocol (big endian)
    pub network_order: bool,
    #[serde(default = "default_true", alias = "position")]
    /// Whether to show the position of the fields
    pub field_position: bool,
    #[serde(default = "default_true", alias = "length")]
    /// Whether to show the length of the fields
    pub field_length: bool,
    #[serde(default = "default_true")]
    /// Whether to show the wrap line
    pub wrap_line: bool,
    #[serde(default = "default_true")]
    /// Whether to show the wrap line
    pub start_symbol: bool,
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
    /// Background color of the image
    pub background_color: HexColor,
    #[serde(default = "default_white")]
    /// Color of the field background
    pub field_color: HexColor,
    #[serde(default = "default_black")]
    /// Text color of the image (field names + stroke)
    pub text_color: HexColor,
    #[serde(default = "default_black")]
    /// Color of the subtitle text (field length and position)
    pub subtitle_color: HexColor,
    #[serde(default = "default_50")]
    /// Width of a field unit in the image
    pub unit_width: usize,
}

impl Default for StyleDescriptor {
    fn default() -> Self {
        Self {
            background_color: default_white(),
            field_color: default_white(),
            text_color: default_black(),
            subtitle_color: default_black(),
            unit_width: default_50(),
        }
    }
}

fn default_white() -> HexColor {
    HexColor::rgb(255, 255, 255)
}

fn default_black() -> HexColor {
    HexColor::rgb(0, 0, 0)
}

fn default_50() -> usize {
    50
}

/// Struct to hold the options for a protocol
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ProtoDescriptor {
    #[serde(default)]
    /// Options for the image elements
    pub elements: ElementsDescriptor,
    #[serde(default)]
    /// Options for the image style
    pub style: StyleDescriptor,
    /// List of fields the protocol contains
    pub fields: Vec<FieldDescriptor>,
}
