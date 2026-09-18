use super::validation::{
    EMAIL_MAX_CHARS, Field, PASSWORD_MAX_CHARS, PASSWORD_MIN_CHARS, ValidationError,
};
use serde::{Deserialize, Serialize};

/// The account visible to its signed-in owner. The password hash never leaves the server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: i64,
}

/// Which kind of client a session belongs to. Web sessions travel in an HttpOnly cookie;
/// desktop sessions use a bearer token kept in the OS credential store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Web,
    Desktop,
}

impl ClientKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
        }
    }
}

/// Trims and lowercases an email address used as a sign-in name.
///
/// This checks shape only; Rivet does not send email or verify ownership.
pub fn normalize_email(value: &str) -> Result<String, ValidationError> {
    let email = value.trim().to_lowercase();
    let invalid = || ValidationError::new(Field::Email, "Enter a valid email address.");
    if email.chars().count() > EMAIL_MAX_CHARS
        || email.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(invalid());
    }
    match email.split_once('@') {
        Some((local, domain))
            if !local.is_empty() && !domain.is_empty() && !domain.contains('@') =>
        {
            Ok(email)
        }
        _ => Err(invalid()),
    }
}

/// Passwords are not trimmed; only their length is checked.
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let length = password.chars().count();
    if !(PASSWORD_MIN_CHARS..=PASSWORD_MAX_CHARS).contains(&length) {
        return Err(ValidationError::new(
            Field::Password,
            format!("Use a password of {PASSWORD_MIN_CHARS}–{PASSWORD_MAX_CHARS} characters."),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_is_trimmed_lowercased_and_shape_checked() {
        assert_eq!(
            normalize_email("  Me@Example.COM ").unwrap(),
            "me@example.com"
        );
        for invalid in ["", "me", "@example.com", "me@", "a@b@c", "me @example.com"] {
            assert_eq!(normalize_email(invalid).unwrap_err().field, Field::Email);
        }
        let long = format!("{}@example.com", "a".repeat(EMAIL_MAX_CHARS));
        assert!(normalize_email(&long).is_err());
    }

    #[test]
    fn password_length_is_bounded_without_trimming() {
        assert!(validate_password(&"a".repeat(PASSWORD_MIN_CHARS)).is_ok());
        assert!(validate_password(&"비".repeat(PASSWORD_MAX_CHARS)).is_ok());
        assert!(validate_password(&"a".repeat(PASSWORD_MIN_CHARS - 1)).is_err());
        assert!(validate_password(&"a".repeat(PASSWORD_MAX_CHARS + 1)).is_err());
        let padded = format!("  {}  ", "a".repeat(PASSWORD_MIN_CHARS - 4));
        assert!(validate_password(&padded).is_ok());
    }
}
