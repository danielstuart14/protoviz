use std::collections::HashMap;

use hex_color::HexColor;
use serde::Serialize;

use crate::descriptor;

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
enum TextBaseline {
    Auto,
    Middle,
    Hanging,
}

#[derive(Debug, Serialize, Clone, Copy)]
struct Components {
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize, Clone, Copy)]
struct ComponentsDynamic {
    x1: f64,
    x2: f64,
    spacing: f64,
    delta: f64,
    y: f64,
}

#[derive(Debug, Serialize)]
struct StaticFields {
    background: HexColor,
    coordinates: Components,
    size: Components,
    stroke_color: HexColor,
    stroke_width: f64,
}

#[derive(Debug, Serialize)]
struct DynamicFields {
    background: HexColor,
    coordinates: Components,
    size: ComponentsDynamic,
    stroke_color: HexColor,
    stroke_width: f64,
}

#[derive(Debug, Serialize)]
struct FieldText {
    text: String,
    coordinates: Components,
    color: HexColor,
    baseline: TextBaseline,
}

#[derive(Debug, Serialize)]
struct FieldTicks {
    coordinates: Components,
    size: Components,
    color: HexColor,
}

#[derive(Debug, Serialize)]
struct FieldLength {
    coordinates: Components,
    size: Components,
    stroke: f64,
    color: HexColor,
}

#[derive(Debug, Serialize)]
pub struct TemplateData {
    size: Components,
    background: HexColor,
    static_fields: Vec<StaticFields>,
    dynamic_fields: Vec<DynamicFields>,
    field_texts: Vec<FieldText>,
    field_ticks: Vec<FieldTicks>,
    field_lengths: Vec<FieldLength>,
}

const DEFAULT_PADDING: f64 = 50.0;
const DEFAULT_STROKE_WIDTH: f64 = 2.0;
const DEFAULT_SIZE_X: f64 = 50.0;
const DEFAULT_SIZE_Y: f64 = 100.0;
const DEFAULT_DYN_LENGTH_1: f64 = 95.0;
const DEFAULT_DYN_LENGTH_2: f64 = 45.0;
const DEFAULT_DYN_SPACING: f64 = 10.0;
const DEFAULT_DYN_DELTA: f64 = 20.0;
const DEFAULT_TICK_SIZE: f64 = 20.0;
const DEFAULT_SUB_PADDING: f64 = 10.0;
const DEFAULT_LENGTH_SIZE: f64 = 10.0;

pub fn generate_data(descriptor: &descriptor::ProtoDescriptor) -> TemplateData {
    let mut static_fields = Vec::new();
    let mut dynamic_fields = Vec::new();
    let mut field_texts = Vec::new();
    let mut field_ticks = Vec::new();
    let mut field_lengths = Vec::new();

    // Used to create the field position subtitles
    let mut positions = Vec::new();

    let mut x = DEFAULT_PADDING;
    let mut y = DEFAULT_PADDING;

    // Reverse the fields if the elements are not in network order (big-endian)
    let fields: Box<dyn Iterator<Item=_>> = if descriptor.elements.is_network {
        Box::new(descriptor.fields.iter())
    } else {
        Box::new(descriptor.fields.iter().rev())
    };

    for field in fields {
        let coordinates = Components { x, y };

        let length = match &field.length {
            descriptor::FieldLength::Fixed(length) => {
                // Add field ticks
                for i in 1..*length {
                    field_ticks.extend([DEFAULT_PADDING, DEFAULT_PADDING + DEFAULT_SIZE_Y - DEFAULT_TICK_SIZE].into_iter().map(|y| FieldTicks {
                        coordinates: Components { x: x + i as f64 * DEFAULT_SIZE_X, y },
                        size: Components { x: DEFAULT_STROKE_WIDTH, y: DEFAULT_TICK_SIZE },
                        color: descriptor.style.text_color,
                    }));
                }

                let size = Components { x: *length as f64 * DEFAULT_SIZE_X, y: DEFAULT_SIZE_Y };

                static_fields.push(StaticFields {
                    background: field.color.unwrap_or(descriptor.style.field_color),
                    coordinates,
                    size: size,
                    stroke_color: descriptor.style.text_color,
                    stroke_width: DEFAULT_STROKE_WIDTH,
                });

                field_texts.push(FieldText {
                    text: field.name.clone(),
                    coordinates: Components { x: x + size.x / 2.0, y: y + DEFAULT_SIZE_Y / 2.0 },
                    color: descriptor.style.text_color,
                    baseline: TextBaseline::Middle,
                });

                size.x
            },
            descriptor::FieldLength::Variable(_length) => {
                let size = ComponentsDynamic {
                    x1: DEFAULT_DYN_LENGTH_1,
                    x2: DEFAULT_DYN_LENGTH_2,
                    spacing: DEFAULT_DYN_SPACING,
                    delta: DEFAULT_DYN_DELTA,
                    y: DEFAULT_SIZE_Y,
                };

                dynamic_fields.push(DynamicFields {
                    background: field.color.unwrap_or(descriptor.style.field_color),
                    coordinates: coordinates,
                    size,
                    stroke_color: descriptor.style.text_color,
                    stroke_width: DEFAULT_STROKE_WIDTH,
                });

                field_texts.push(FieldText {
                    text: field.name.clone(),
                    coordinates: Components { x: x + size.x1 / 2.0, y: y + DEFAULT_SIZE_Y / 2.0 },
                    color: descriptor.style.text_color,
                    baseline: TextBaseline::Middle,
                });


                size.x1 + size.spacing + size.x2
            }
        };

        // Add length and position subtitle coordinates to the positions vector
        let pos_x = if descriptor.elements.is_network {
            x + DEFAULT_SIZE_X / 2.0
        } else {
            x + length - DEFAULT_SIZE_X / 2.0
        };
        positions.push((field.length.clone(), Components { x: pos_x, y: y - DEFAULT_SUB_PADDING }));

        // If field length subtitles are enabled, add them
        if descriptor.elements.length {
            field_lengths.push(FieldLength {
                coordinates: Components { x: x, y: y + DEFAULT_SIZE_Y + DEFAULT_SUB_PADDING + DEFAULT_LENGTH_SIZE / 2.0 },
                size: Components { x: length, y: DEFAULT_LENGTH_SIZE },
                stroke: DEFAULT_STROKE_WIDTH,
                color: descriptor.style.subtitle_color,
            });

            field_texts.push(FieldText {
                text: field.length.to_string(),
                coordinates: Components { x: x + length / 2.0, y: y + DEFAULT_SIZE_Y + DEFAULT_SUB_PADDING + DEFAULT_LENGTH_SIZE },
                color: descriptor.style.subtitle_color,
                baseline: TextBaseline::Hanging,
            });
        }

        x += length;
    }

    // If field position subtitles are enabled, add them
    if descriptor.elements.position {
        if !descriptor.elements.is_network {
            positions.reverse();
        }

        let mut var_length = HashMap::new();
        let mut fixed_length = 0;
        for (length, position) in positions {
            match length {
                descriptor::FieldLength::Variable(length) => {
                    let length = length.trim().to_owned();
                    if let Some(value) = var_length.get(&length) {
                        var_length.insert(length, *value + 1);
                    } else {
                        var_length.insert(length, 1 as usize);
                    }
                },
                descriptor::FieldLength::Fixed(length) => {
                    field_texts.push(FieldText {
                        text: create_position_sub(&mut var_length, fixed_length),
                        coordinates: position,
                        color: descriptor.style.subtitle_color,
                        baseline: TextBaseline::Auto,
                    });

                    fixed_length += length;
                }
            }
        }
    }

    x += DEFAULT_PADDING;
    y += DEFAULT_SIZE_Y + DEFAULT_PADDING;

    TemplateData {
        size: Components { x, y },
        background: descriptor.style.background_color,
        static_fields,
        dynamic_fields,
        field_texts,
        field_ticks,
        field_lengths,
    }
}

fn create_position_sub(var_length: &mut HashMap<String, usize>, fixed_length: usize) -> String {
    let mut result = String::new();

    if fixed_length > 0 || var_length.is_empty() {
        result.push_str(&fixed_length.to_string());
    }

    for (length, count) in var_length {
        if !result.is_empty() {
            result.push_str(" + ");
        }

        if *count == 1 {
            result.push_str(&length);
        } else {
            result.push_str(&format!("{}{}", count, length));
        }
    }

    result
}