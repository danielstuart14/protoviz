use std::{collections::HashMap, vec};

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
const DEFAULT_DYN_SPACING_VALUE: f64 = 10.0;

// PERCENTAGE FROM UNIT_WIDTH
const DEFAULT_DYN_LENGTH_1: f64 = 2.0 / 3.0;
const DEFAULT_DYN_LENGTH_2: f64 = 1.0 / 3.0;
const DEFAULT_DYN_SPACING_UPPER: f64 = 0.2;
const DEFAULT_DYN_SPACING_LOWER: f64 = 0.5;
const DEFAULT_DYN_DELTA: f64 = 0.5;

/// Generate the data consumed by the SVG template
pub fn generate_data(descriptor: &descriptor::ProtoDescriptor) -> TemplateData {
    let mut static_fields_rows = vec![Vec::new()];
    let mut dynamic_fields_rows = vec![Vec::new()];
    let mut field_texts_rows = vec![Vec::new()];
    let mut field_ticks_rows = vec![Vec::new()];
    let mut wrap_lines_rows = vec![Vec::new()];
    let mut start_symbol: Option<StartSymbol> = None;

    // Used to create the field position subtitles
    let mut positions_rows = vec![Vec::new()];

    // Used to create the field length subtitles
    let mut lengths_rows = vec![Vec::new()];

    // Used to offset the X coord if not in network order
    let mut row_sizes = Vec::new();

    let mut x = DEFAULT_PADDING;
    let mut y = DEFAULT_PADDING;

    let mut max_x = 0.0;
    let mut row_max_x = 0.0;
    let mut last_row_y = DEFAULT_PADDING;

    // Reverse the fields if the elements are not in network order (big-endian)
    let fields: Box<dyn Iterator<Item = _>> = if descriptor.elements.network_order {
        Box::new(descriptor.fields.iter())
    } else {
        Box::new(descriptor.fields.iter().rev())
    };

    // Default width of a field unit
    let unit_width = descriptor.style.unit_width as f64;

    // Default units of a dynamic field
    let dyn_units = descriptor.style.dyn_units as f64;

    for (i, field) in fields.into_iter().enumerate() {
        // Wrap line before field if not in network order and wrap is enabled
        if !descriptor.elements.network_order && field.wrap && i != 0 {
            if let Some(wrap_line) = wrap_line(descriptor, &mut x, &mut y) {
                wrap_lines_rows.last_mut().unwrap().push(wrap_line);
            }
        }

        // If the field is in a new row, save the previous row size
        if y != last_row_y {
            row_sizes.push(row_max_x);
            row_max_x = 0.0;
            last_row_y = y;

            static_fields_rows.push(Vec::new());
            dynamic_fields_rows.push(Vec::new());
            field_texts_rows.push(Vec::new());
            field_ticks_rows.push(Vec::new());
            wrap_lines_rows.push(Vec::new());
            positions_rows.push(Vec::new());
            lengths_rows.push(Vec::new());
        }

        let coordinates = Components { x, y };

        let length = match &field.length {
            descriptor::FieldLength::Fixed(length) => {
                // Add field ticks
                for i in 1..*length {
                    field_ticks_rows.last_mut().unwrap().extend(
                        [0.0, DEFAULT_SIZE_Y - DEFAULT_TICK_SIZE]
                            .into_iter()
                            .map(|y_delta| FieldTicks {
                                coordinates: Components {
                                    x: x + i as f64 * unit_width,
                                    y: y + y_delta,
                                },
                                size: Components {
                                    x: DEFAULT_STROKE_WIDTH,
                                    y: DEFAULT_TICK_SIZE,
                                },
                                color: descriptor.style.text_color,
                            }),
                    );
                }

                let size = Components {
                    x: *length as f64 * unit_width,
                    y: DEFAULT_SIZE_Y,
                };

                static_fields_rows.last_mut().unwrap().push(StaticFields {
                    background: field.color.unwrap_or(descriptor.style.field_color),
                    coordinates,
                    size: size,
                    stroke_color: descriptor.style.text_color,
                    stroke_width: DEFAULT_STROKE_WIDTH,
                });

                field_texts_rows.last_mut().unwrap().push(FieldText {
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
                let spacing = if DEFAULT_DYN_SPACING_UPPER * unit_width < DEFAULT_DYN_SPACING_VALUE
                {
                    if unit_width >= DEFAULT_DYN_SPACING_VALUE * 2.0 {
                        DEFAULT_DYN_SPACING_VALUE
                    } else {
                        DEFAULT_DYN_SPACING_LOWER * unit_width
                    }
                } else {
                    DEFAULT_DYN_SPACING_UPPER * unit_width
                };

                let size = ComponentsDynamic {
                    x1: (DEFAULT_DYN_LENGTH_1 * dyn_units).ceil() * unit_width - spacing / 2.0,
                    x2: (DEFAULT_DYN_LENGTH_2 * dyn_units).floor() * unit_width - spacing / 2.0,
                    spacing: spacing,
                    delta: DEFAULT_DYN_DELTA * unit_width,
                    y: DEFAULT_SIZE_Y,
                };

                dynamic_fields_rows.last_mut().unwrap().push(DynamicFields {
                    background: field.color.unwrap_or(descriptor.style.field_color),
                    coordinates: coordinates,
                    size,
                    stroke_color: descriptor.style.text_color,
                    stroke_width: DEFAULT_STROKE_WIDTH,
                });

                field_texts_rows.last_mut().unwrap().push(FieldText {
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

        // If position subtitles are enabled, add them to the positions vector
        if descriptor.elements.field_position {
            let pos_x = if descriptor.elements.network_order {
                x + unit_width / 2.0
            } else {
                x + length - unit_width / 2.0
            };

            let pos_y = if descriptor.elements.network_order {
                y - DEFAULT_SUB_PADDING
            } else {
                y + DEFAULT_SIZE_Y + DEFAULT_SUB_PADDING
            };

            positions_rows
                .last_mut()
                .unwrap()
                .push((field.length.clone(), Components { x: pos_x, y: pos_y }));
        }

        // If field length subtitles are enabled, add them
        if descriptor.elements.field_length {
            let pos_y = if descriptor.elements.network_order {
                y + DEFAULT_SIZE_Y + DEFAULT_SUB_PADDING + DEFAULT_LENGTH_SIZE / 2.0
            } else {
                y - DEFAULT_SUB_PADDING - DEFAULT_LENGTH_SIZE / 2.0
            };

            let length_sub = FieldLength {
                coordinates: Components { x: x, y: pos_y },
                size: Components {
                    x: length,
                    y: DEFAULT_LENGTH_SIZE,
                },
                stroke: DEFAULT_STROKE_WIDTH,
                color: descriptor.style.subtitle_color,
            };

            let (pos_y, baseline) = if descriptor.elements.network_order {
                (pos_y + DEFAULT_LENGTH_SIZE / 2.0, TextBaseline::Hanging)
            } else {
                (pos_y - DEFAULT_LENGTH_SIZE / 2.0, TextBaseline::Auto)
            };

            let length_text = FieldText {
                text: field.length.to_string(),
                coordinates: Components {
                    x: x + length / 2.0,
                    y: pos_y,
                },
                color: descriptor.style.subtitle_color,
                baseline: baseline,
                height: DEFAULT_TEXT_SIZE,
            };

            lengths_rows
                .last_mut()
                .unwrap()
                .push((length_sub, length_text));
        }

        x += length;

        if x > row_max_x {
            row_max_x = x;

            if x > max_x {
                max_x = x;
            }
        }

        // Wrap line after field if in network order and wrap is enabled
        if descriptor.elements.network_order && field.wrap && i != descriptor.fields.len() - 1 {
            if let Some(wrap_line) = wrap_line(descriptor, &mut x, &mut y) {
                wrap_lines_rows.last_mut().unwrap().push(wrap_line);
            }
        }
    }

    // Add the last row size
    row_sizes.push(row_max_x);

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

    // Flatten the rows and apply the offset if needed (align to the right if not in network order)
    let static_fields = static_fields_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter().map(move |mut field| {
                if !descriptor.elements.network_order {
                    field.coordinates.x += max_x - row_sizes[i];
                }
                field
            })
        })
        .collect::<Vec<_>>();

    let dynamic_fields = dynamic_fields_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter().map(move |mut field| {
                if !descriptor.elements.network_order {
                    field.coordinates.x += max_x - row_sizes[i];
                }
                field
            })
        })
        .collect::<Vec<_>>();

    let field_ticks = field_ticks_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter().map(move |mut field| {
                if !descriptor.elements.network_order {
                    field.coordinates.x += max_x - row_sizes[i];
                }
                field
            })
        })
        .collect::<Vec<_>>();

    let wrap_lines = wrap_lines_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter().map(move |mut field| {
                if !descriptor.elements.network_order {
                    field.start.x += max_x - row_sizes[i];
                    field.end.x += max_x - row_sizes[i + 1];
                }
                field
            })
        })
        .collect::<Vec<_>>();

    let mut lengths = lengths_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter()
                .map(move |(mut length_sub, mut length_text)| {
                    if !descriptor.elements.network_order {
                        length_sub.coordinates.x += max_x - row_sizes[i];
                        length_text.coordinates.x += max_x - row_sizes[i];
                    }
                    (length_sub, length_text)
                })
        })
        .collect::<Vec<_>>();

    let mut positions = positions_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter().map(move |(length, mut position)| {
                if !descriptor.elements.network_order {
                    position.x += max_x - row_sizes[i];
                }
                (length, position)
            })
        })
        .collect::<Vec<_>>();

    let mut field_texts = field_texts_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row_sizes = &row_sizes;
            row.into_iter().map(move |mut field| {
                if !descriptor.elements.network_order {
                    field.coordinates.x += max_x - row_sizes[i];
                }
                field
            })
        })
        .collect::<Vec<_>>();

    // Apply offset to the start symbol if needed
    if !descriptor.elements.network_order {
        if let Some(start_symbol) = start_symbol.as_mut() {
            start_symbol.coordinates.x += max_x - row_sizes.last().unwrap();
        }
    }

    // If field position subtitles are enabled, add them
    if descriptor.elements.field_position {
        if !descriptor.elements.network_order {
            positions.reverse();
        }

        let mut var_length = HashMap::new();
        let mut fixed_length = 0;
        let start_y = positions.first().map(|(_, pos)| pos.y).unwrap_or(0.0);
        for (length, position) in positions {
            // If only outer subtitles are enabled, break if the Y position changes
            if !descriptor.elements.inner_subtitles && start_y != position.y {
                break;
            }

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
                    let baseline = if descriptor.elements.network_order {
                        TextBaseline::Auto
                    } else {
                        TextBaseline::Hanging
                    };

                    field_texts.push(FieldText {
                        text: create_position_sub(&mut var_length, fixed_length),
                        coordinates: position,
                        color: descriptor.style.subtitle_color,
                        baseline: baseline,
                        height: DEFAULT_TEXT_SIZE,
                    });

                    fixed_length += length;
                }
            }
        }
    }

    let mut field_lengths = Vec::new();

    // If field length subtitles are enabled, add them
    if descriptor.elements.field_length {
        if descriptor.elements.network_order {
            lengths.reverse();
        }

        let start_y = lengths
            .first()
            .map(|(length_sub, _)| length_sub.coordinates.y)
            .unwrap_or(0.0);
        for (length_sub, length_text) in lengths {
            // If only outer subtitles are enabled, break if the Y position changes
            if !descriptor.elements.inner_subtitles && start_y != length_sub.coordinates.y {
                break;
            }

            field_lengths.push(length_sub);
            field_texts.push(length_text);
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
        || (descriptor.elements.inner_subtitles
            && (descriptor.elements.field_length || descriptor.elements.field_position))
    {
        *y += 2.0 * DEFAULT_SUB_PADDING
    }

    if descriptor.elements.network_order {
        if descriptor.elements.inner_subtitles && descriptor.elements.field_length {
            let len_y = DEFAULT_LENGTH_SIZE + DEFAULT_TEXT_SIZE + DEFAULT_SUB_PADDING / 2.0;
            *y += len_y;
            center_delta += len_y;
        }
        if descriptor.elements.inner_subtitles && descriptor.elements.field_position {
            *y += DEFAULT_TEXT_SIZE + DEFAULT_SUB_PADDING / 2.0;
        }
    } else {
        if descriptor.elements.inner_subtitles && descriptor.elements.field_position {
            let len_y = DEFAULT_TEXT_SIZE + DEFAULT_SUB_PADDING / 2.0;
            *y += len_y;
            center_delta += len_y;
        }
        if descriptor.elements.inner_subtitles && descriptor.elements.field_length {
            *y += DEFAULT_LENGTH_SIZE + DEFAULT_TEXT_SIZE + DEFAULT_SUB_PADDING / 2.0;
        }
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
