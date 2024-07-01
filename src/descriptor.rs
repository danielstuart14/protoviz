use serde::{Deserialize, Serialize};

/// Enum to hold the length of a field
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum FieldLength {
    Fixed(usize),
    Variable(String),
}

/// Struct to hold the options for a field
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldDescriptor {
    pub name: String,
    pub length: FieldLength,
}

/// Struct to hold the options for the image elements
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ElementsDescriptor {
    //TODO: pub is_network: bool, // Whether it is a network protocol (big endian)
    pub show_position: bool, // Whether to show the position of the fields
    pub show_length: bool, // Whether to show the length of the fields
}

/// Struct to hold the options for a protocol
#[derive(Debug, Deserialize, Default)]
pub struct ProtoDescriptor {
    #[serde(default)]
    pub elements: ElementsDescriptor, // Options for the image elements
    pub fields: Vec<FieldDescriptor>, // List of fields the protocol contains
}