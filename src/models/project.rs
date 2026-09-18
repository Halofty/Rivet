use super::validation::{self, Field, PROJECT_NAME_MAX_CHARS, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    /// UTC Unix milliseconds, set by the server.
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: String,
}

impl CreateProjectInput {
    pub fn validate(&self) -> Result<NewProject, ValidationError> {
        Ok(NewProject {
            name: validation::required_text(
                Field::Name,
                "project name",
                &self.name,
                PROJECT_NAME_MAX_CHARS,
            )?,
            description: validation::description(&self.description)?,
        })
    }
}

/// A normalized project input. Only [`CreateProjectInput::validate`] creates one,
/// so persistence code cannot receive unchecked values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewProject {
    name: String,
    description: String,
}

impl NewProject {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_name_is_trimmed_and_bounded() {
        let input = CreateProjectInput {
            name: "  Rivet  ".into(),
            description: " keep ".into(),
        };
        let valid = input.validate().unwrap();
        assert_eq!(valid.name(), "Rivet");
        assert_eq!(valid.description(), " keep ");

        for name in ["", "   ", &"a".repeat(PROJECT_NAME_MAX_CHARS + 1)] {
            let error = CreateProjectInput {
                name: name.into(),
                description: String::new(),
            }
            .validate()
            .unwrap_err();
            assert_eq!(error.field, Field::Name);
        }
        assert!(
            CreateProjectInput {
                name: "가".repeat(PROJECT_NAME_MAX_CHARS),
                description: String::new(),
            }
            .validate()
            .is_ok()
        );
    }
}
