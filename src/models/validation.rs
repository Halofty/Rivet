use serde::{Deserialize, Serialize};

pub const PROJECT_NAME_MAX_CHARS: usize = 100;
pub const ISSUE_TITLE_MAX_CHARS: usize = 200;
pub const DESCRIPTION_MAX_CHARS: usize = 10_000;
pub const EMAIL_MAX_CHARS: usize = 254;
pub const PASSWORD_MIN_CHARS: usize = 12;
/// Bounds the work a single sign-in request can ask the password hasher to do.
pub const PASSWORD_MAX_CHARS: usize = 128;

/// The input a validation failure refers to, so forms can place the message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Name,
    Title,
    Description,
    Email,
    Password,
    /// The request as a whole, such as an update without any field.
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{message}")]
pub struct ValidationError {
    pub field: Field,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: Field, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

/// Trims surrounding whitespace and requires 1..=`max` Unicode scalar values.
pub(crate) fn required_text(
    field: Field,
    label: &str,
    value: &str,
    max: usize,
) -> Result<String, ValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > max {
        return Err(ValidationError::new(
            field,
            format!("Enter a {label} of 1–{max} characters."),
        ));
    }
    Ok(trimmed.to_owned())
}

/// Descriptions are plain text; whitespace is preserved and may be empty.
pub(crate) fn description(value: &str) -> Result<String, ValidationError> {
    if value.chars().count() > DESCRIPTION_MAX_CHARS {
        return Err(ValidationError::new(
            Field::Description,
            format!("Keep the description to at most {DESCRIPTION_MAX_CHARS} characters."),
        ));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_text_trims_and_counts_unicode_scalars() {
        assert_eq!(
            required_text(Field::Name, "name", "  안녕  ", 2).unwrap(),
            "안녕"
        );
        assert!(required_text(Field::Name, "name", "안녕하", 2).is_err());
        assert!(required_text(Field::Name, "name", " \t\n\u{3000}", 2).is_err());
    }

    #[test]
    fn description_preserves_whitespace_up_to_the_limit() {
        assert_eq!(description("  a\n").unwrap(), "  a\n");
        assert_eq!(description("").unwrap(), "");
        assert!(description(&"가".repeat(DESCRIPTION_MAX_CHARS)).is_ok());
        let error = description(&"가".repeat(DESCRIPTION_MAX_CHARS + 1)).unwrap_err();
        assert_eq!(error.field, Field::Description);
    }
}
