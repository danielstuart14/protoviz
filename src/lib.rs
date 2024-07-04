pub mod descriptor;
pub mod errors;

use errors::Error;
use tera::{Context, Tera};

// TODO: Add support for little endian
// TODO: Add support for larger variable length tags (subtitle text wrapping)
// TODO: Add support for larger field names (field name text wrapping)

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

    let mut context = Context::new();

    context.insert("fields", &descriptor.fields);
    context.insert("elements", &descriptor.elements);
    context.insert("style", &descriptor.style);

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
                is_network: true,
                position: true,
                length: true,
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
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field1".to_string(),
                    length: descriptor::FieldLength::Fixed(2),
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field2".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field3".to_string(),
                    length: descriptor::FieldLength::Variable("N".to_string()),
                    color: None,
                },
                descriptor::FieldDescriptor {
                    name: "field4".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                    color: None,
                },
            ],
        };

        let result = render(&descriptor).unwrap();
        assert!(result.contains("field1"));
        assert!(result.contains("field3"));
    }
}