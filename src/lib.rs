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
    context.insert("show_length", &descriptor.elements.show_length);
    context.insert("show_position", &descriptor.elements.show_position);

    Tera::one_off(include_str!("../template.svg"), &context, false).map_err(|e| Error::TeraError(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let descriptor = descriptor::ProtoDescriptor {
            elements: descriptor::ElementsDescriptor {
                show_position: true,
                show_length: true,
            },
            fields: vec![
                descriptor::FieldDescriptor {
                    name: "field0".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                },
                descriptor::FieldDescriptor {
                    name: "field1".to_string(),
                    length: descriptor::FieldLength::Fixed(2),
                },
                descriptor::FieldDescriptor {
                    name: "field2".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                },
                descriptor::FieldDescriptor {
                    name: "field3".to_string(),
                    length: descriptor::FieldLength::Variable("N".to_string()),
                },
                descriptor::FieldDescriptor {
                    name: "field4".to_string(),
                    length: descriptor::FieldLength::Fixed(1),
                },
            ],
        };

        let result = render(&descriptor).unwrap();
        assert!(result.contains("field1"));
        assert!(result.contains("field3"));
    }
}