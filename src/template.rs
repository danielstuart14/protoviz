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
    height: f64,
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
struct WrapLine {
    start: Components,
    end: Components,
    center_delta: f64,
    padding: f64,
    stroke: f64,
    color: HexColor,
}

#[derive(Debug, Serialize)]
struct StartSymbol {
    coordinates: Components,
    size: Components,
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
    wrap_lines: Vec<WrapLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_symbol: Option<StartSymbol>,
}

const DEFAULT_PADDING: f64 = 50.0;
const DEFAULT_STROKE_WIDTH: f64 = 2.0;
const DEFAULT_SIZE_Y: f64 = 80.0;
const DEFAULT_TICK_SIZE: f64 = 20.0;
const DEFAULT_SUB_PADDING: f64 = 10.0;
const DEFAULT_LENGTH_SIZE: f64 = 10.0;
const DEFAULT_TEXT_SIZE: f64 = 16.0;
const DEFAULT_START_SYMBOL_X: f64 = 10.0;
const DEFAULT_START_SYMBOL_Y: f64 = 20.0;

// PERCENTAGE FROM UNIT_WIDTH
const DEFAULT_DYN_LENGTH_1: f64 = 1.9;
const DEFAULT_DYN_LENGTH_2: f64 = 0.9;
const DEFAULT_DYN_SPACING: f64 = 0.2;
const DEFAULT_DYN_DELTA: f64 = 0.5;

/// Generate the data consumed by the SVG template
pub fn generate_data(descriptor: &descriptor::ProtoDescriptor) -> TemplateData {
    let mut static_fields = Vec::new();
    let mut dynamic_fields = Vec::new();
    let mut field_texts = Vec::new();
    let mut field_ticks = Vec::new();
    let mut field_lengths = Vec::new();
    let mut wrap_lines = Vec::new();
    let mut start_symbol: Option<StartSymbol> = None;

    // Used to create the field position subtitles
    let mut positions = Vec::new();

    let mut x = DEFAULT_PADDING;
    let mut y = DEFAULT_PADDING;

    let mut max_x = 0.0;

    // Reverse the fields if the elements are not in network order (big-endian)
    let fields: Box<dyn Iterator<Item = _>> = if descriptor.elements.network_order {
        Box::new(descriptor.fields.iter())
    } else {
        Box::new(descriptor.fields.iter().rev())
    };

    // Default width of a field unit
    let default_width = descriptor.style.unit_width as f64;

    for (i, field) in fields.into_iter().enumerate() {
        // Wrap line before field if not in network order and wrap is enabled
        if !descriptor.elements.network_order && field.wrap && i != 0 {
            if let Some(wrap_line) = wrap_line(descriptor, &mut x, &mut y) {
                wrap_lines.push(wrap_line);
            }
        }

        let coordinates = Components { x, y };

        let length = match &field.length {
            descriptor::FieldLength::Fixed(length) => {
                // Add field ticks
                for i in 1..*length {
                    field_ticks.extend([0.0, DEFAULT_SIZE_Y - DEFAULT_TICK_SIZE].into_iter().map(
                        |y_delta| FieldTicks {
                            coordinates: Components {
                                x: x + i as f64 * default_width,
                                y: y + y_delta,
                            },
                            size: Components {
                                x: DEFAULT_STROKE_WIDTH,
                                y: DEFAULT_TICK_SIZE,
                            },
                            color: descriptor.style.text_color,
                        },
                    ));
                }

                let size = Components {
                    x: *length as f64 * default_width,
                    y: DEFAULT_SIZE_Y,
                };

                static_fields.push(StaticFields {
                    background: field.color.unwrap_or(descriptor.style.field_color),
                    coordinates,
                    size: size,
                    stroke_color: descriptor.style.text_color,
                    stroke_width: DEFAULT_STROKE_WIDTH,
                });

                field_texts.push(FieldText {
                    text: field.name.clone(),
                    coordinates: Components {
                        x: x + size.x / 2.0,
                        y: y + DEFAULT_SIZE_Y / 2.0,
                    },
                    color: descriptor.style.text_color,
                    baseline: TextBaseline::Middle,
                    height: DEFAULT_TEXT_SIZE,
                });

                size.x
            }
            descriptor::FieldLength::Variable(_length) => {
                let size = ComponentsDynamic {
                    x1: DEFAULT_DYN_LENGTH_1 * default_width,
                    x2: DEFAULT_DYN_LENGTH_2 * default_width,
                    spacing: DEFAULT_DYN_SPACING * default_width,
                    delta: DEFAULT_DYN_DELTA * default_width,
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
                    coordinates: Components {
                        x: x + size.x1 / 2.0,
                        y: y + DEFAULT_SIZE_Y / 2.0,
                    },
                    color: descriptor.style.text_color,
                    baseline: TextBaseline::Middle,
                    height: DEFAULT_TEXT_SIZE,
                });

                size.x1 + size.spacing + size.x2
            }
        };

        // Add length and position subtitle coordinates to the positions vector
        let pos_x = if descriptor.elements.network_order {
            x + default_width / 2.0
        } else {
            x + length - default_width / 2.0
        };
        positions.push((
            field.length.clone(),
            Components {
                x: pos_x,
                y: y - DEFAULT_SUB_PADDING,
            },
        ));

        // If field length subtitles are enabled, add them
        if descriptor.elements.field_length {
            field_lengths.push(FieldLength {
                coordinates: Components {
                    x: x,
                    y: y + DEFAULT_SIZE_Y + DEFAULT_SUB_PADDING + DEFAULT_LENGTH_SIZE / 2.0,
                },
                size: Components {
                    x: length,
                    y: DEFAULT_LENGTH_SIZE,
                },
                stroke: DEFAULT_STROKE_WIDTH,
                color: descriptor.style.subtitle_color,
            });

            field_texts.push(FieldText {
                text: field.length.to_string(),
                coordinates: Components {
                    x: x + length / 2.0,
                    y: y + DEFAULT_SIZE_Y + DEFAULT_SUB_PADDING + DEFAULT_LENGTH_SIZE,
                },
                color: descriptor.style.subtitle_color,
                baseline: TextBaseline::Hanging,
                height: DEFAULT_TEXT_SIZE,
            });
        }

        x += length;

        if x > max_x {
            max_x = x;
        }

        // Wrap line after field if in network order and wrap is enabled
        if descriptor.elements.network_order && field.wrap && i != descriptor.fields.len() - 1 {
            if let Some(wrap_line) = wrap_line(descriptor, &mut x, &mut y) {
                wrap_lines.push(wrap_line);
            }
        }
    }

    // Add start symbol if enabled
    if descriptor.elements.start_symbol {
        if descriptor.elements.network_order {
            start_symbol = Some(StartSymbol {
                coordinates: Components {
                    x: DEFAULT_PADDING - DEFAULT_SUB_PADDING,
                    y: DEFAULT_PADDING + DEFAULT_SIZE_Y / 2.0,
                },
                size: Components {
                    x: -DEFAULT_START_SYMBOL_X,
                    y: DEFAULT_START_SYMBOL_Y,
                },
                color: descriptor.style.subtitle_color,
            });
        } else {
            start_symbol = Some(StartSymbol {
                coordinates: Components {
                    x: x + DEFAULT_SUB_PADDING,
                    y: y + DEFAULT_SIZE_Y / 2.0,
                },
                size: Components {
                    x: DEFAULT_START_SYMBOL_X,
                    y: DEFAULT_START_SYMBOL_Y,
                },
                color: descriptor.style.subtitle_color,
            });
        }
    }

    // If field position subtitles are enabled, add them
    if descriptor.elements.field_position {
        if !descriptor.elements.network_order {
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
                }
                descriptor::FieldLength::Fixed(length) => {
                    field_texts.push(FieldText {
                        text: create_position_sub(&mut var_length, fixed_length),
                        coordinates: position,
                        color: descriptor.style.subtitle_color,
                        baseline: TextBaseline::Auto,
                        height: DEFAULT_TEXT_SIZE,
                    });

                    fixed_length += length;
                }
            }
        }
    }

    max_x += DEFAULT_PADDING;
    y += DEFAULT_SIZE_Y + DEFAULT_PADDING;

    TemplateData {
        size: Components { x: max_x, y },
        background: descriptor.style.background_color,
        static_fields,
        dynamic_fields,
        field_texts,
        field_ticks,
        field_lengths,
        wrap_lines,
        start_symbol,
    }
}

/// Create the position subtitle string
fn create_position_sub(var_length: &mut HashMap<String, usize>, fixed_length: usize) -> String {
    let mut result = String::new();

    if fixed_length > 0 || var_length.is_empty() {
        result.push_str(&fixed_length.to_string());
    }

    for (length, count) in var_length {
        if !result.is_empty() {
            result.push('+');
        }

        if *count == 1 {
            result.push_str(&length);
        } else {
            result.push_str(&format!("{}{}", count, length));
        }
    }

    result
}

/// Create a wrap line if needed
fn wrap_line(
    descriptor: &descriptor::ProtoDescriptor,
    x: &mut f64,
    y: &mut f64,
) -> Option<WrapLine> {
    let start = Components {
        x: *x,
        y: *y + DEFAULT_SIZE_Y / 2.0,
    };
    // Delta from start Y to center of the line
    let mut center_delta = DEFAULT_SUB_PADDING + DEFAULT_SIZE_Y / 2.0;

    // Add new line with subtitle spacing if needed
    *y += DEFAULT_SIZE_Y;
    if descriptor.elements.wrap_line
        || descriptor.elements.field_length
        || descriptor.elements.field_position
    {
        *y += 2.0 * DEFAULT_SUB_PADDING
    }
    if descriptor.elements.field_length {
        let len_y = DEFAULT_LENGTH_SIZE + DEFAULT_TEXT_SIZE + DEFAULT_SUB_PADDING / 2.0;
        *y += len_y;
        center_delta += len_y;
    }
    if descriptor.elements.field_position {
        *y += DEFAULT_TEXT_SIZE + DEFAULT_SUB_PADDING / 2.0;
    }

    *x = DEFAULT_PADDING;

    let end = Components {
        x: *x,
        y: *y + DEFAULT_SIZE_Y / 2.0,
    };

    if descriptor.elements.wrap_line {
        Some(WrapLine {
            start,
            end,
            center_delta,
            padding: DEFAULT_PADDING / 2.0,
            stroke: DEFAULT_STROKE_WIDTH,
            color: descriptor.style.subtitle_color,
        })
    } else {
        None
    }
}
