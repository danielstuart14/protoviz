pub mod descriptor;
pub mod errors;
mod template;

use errors::Error;
use template::generate_data;
use tera::{Context, Tera};

/// Render the SVG image of the protocol
pub fn render(descriptor: &descriptor::ProtoDescriptor) -> Result<String, Error> {
    if descriptor.fields.is_empty() {
        return Err(Error::FormatError("No fields provided".to_string()));
    }

    for field in &descriptor.fields {
        if let descriptor::FieldLength::Fixed(0) = field.length {
            return Err(Error::FormatError("Field length cannot be zero".to_string()));
        }

        if let descriptor::FieldLength::Variable(name) = &field.length {
            if name.is_empty() {
                return Err(Error::FormatError("Field length cannot be empty".to_string()));
            }
        }
    }

    let data = generate_data(descriptor);

    let mut context = Context::new();

    context.insert("data", &data);

    Tera::one_off(include_str!("../template.svg"), &context, false).map_err(|e| Error::TeraError(e))
}

#[cfg(test)]
mod tests {
    use hex_color::HexColor;

    use super::*;

    #[test]
    fn test_render() {
        let descriptor = descriptor::ProtoDescriptor {
            elements: descriptor::ElementsDescriptor {
                network_order: true,
                field_position: true,
                field_length: true,
                wrap_line: true,
                start_symbol: true,
            },
            style: descriptor::StyleDescriptor {
                background_color: HexColor::rgb(255, 255, 255),
                field_color: HexColor::rgb(255, 255, 255),
                text_color: HexColor::rgb(0, 0, 0),
                subtitle_color: HexColor::rgb(0, 0, 0),
            },
            fields: vec![
                descriptor::FieldDescriptor {
                    name: "field0".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                    wrap: false,
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field1".to_string(),
                    length: descriptor::FieldLength::Fixed(2),
                    wrap: false,
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field2".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                    wrap: false,
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field3".to_string(),
                    length: descriptor::FieldLength::Variable("N".to_string()),
                    wrap: false,
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field4".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                    wrap: false,
                    color: None,
                },
            ],
        };

        let result = render(&descriptor).unwrap();
        assert!(result.contains("field1"));
        assert!(result.contains("field3"));
    }
}