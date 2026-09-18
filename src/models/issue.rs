use super::validation::{self, Field, ISSUE_TITLE_MAX_CHARS, ValidationError};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Todo,
    InProgress,
    Done,
}

impl IssueStatus {
    /// Board column order.
    pub const ALL: [Self; 3] = [Self::Todo, Self::InProgress, Self::Done];

    pub fn label(self) -> &'static str {
        match self {
            Self::Todo => "Todo",
            Self::InProgress => "In Progress",
            Self::Done => "Done",
        }
    }

    /// Stable value used in storage, transport, and form controls.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown issue status `{0}`")]
pub struct UnknownStatus(pub String);

impl FromStr for IssueStatus {
    type Err = UnknownStatus;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == value)
            .ok_or_else(|| UnknownStatus(value.to_owned()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: String,
    pub status: IssueStatus,
    /// UTC Unix milliseconds, set by the server.
    pub created_at: i64,
    pub updated_at: i64,
}

/// New issues always start as Todo; the parent project is a separate argument.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateIssueInput {
    pub title: String,
    pub description: String,
}

impl CreateIssueInput {
    pub fn validate(&self) -> Result<NewIssue, ValidationError> {
        Ok(NewIssue {
            title: issue_title(&self.title)?,
            description: validation::description(&self.description)?,
        })
    }
}

/// A normalized issue input, created only through [`CreateIssueInput::validate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewIssue {
    title: String,
    description: String,
}

impl NewIssue {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// A patch: `None` leaves a field unchanged and `Some("")` clears a description.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<IssueStatus>,
}

impl UpdateIssueInput {
    pub fn validate(&self) -> Result<IssuePatch, ValidationError> {
        if self.title.is_none() && self.description.is_none() && self.status.is_none() {
            return Err(ValidationError::new(
                Field::Input,
                "Change at least one field before saving.",
            ));
        }
        Ok(IssuePatch {
            title: self.title.as_deref().map(issue_title).transpose()?,
            description: self
                .description
                .as_deref()
                .map(validation::description)
                .transpose()?,
            status: self.status,
        })
    }
}

/// A validated, non-empty patch, created only through [`UpdateIssueInput::validate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuePatch {
    title: Option<String>,
    description: Option<String>,
    status: Option<IssueStatus>,
}

impl IssuePatch {
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> Option<IssueStatus> {
        self.status
    }
}

fn issue_title(value: &str) -> Result<String, ValidationError> {
    validation::required_text(Field::Title, "title", value, ISSUE_TITLE_MAX_CHARS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trips_through_storage_and_serde_values() {
        for status in IssueStatus::ALL {
            assert_eq!(status.as_str().parse::<IssueStatus>().unwrap(), status);
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", status.as_str()));
        }
        assert!("blocked".parse::<IssueStatus>().is_err());
        assert!("Todo".parse::<IssueStatus>().is_err());
    }

    #[test]
    fn create_issue_normalizes_title_and_keeps_description() {
        let valid = CreateIssueInput {
            title: "  Fix the board\n".into(),
            description: "\n  details".into(),
        }
        .validate()
        .unwrap();
        assert_eq!(valid.title(), "Fix the board");
        assert_eq!(valid.description(), "\n  details");

        let error = CreateIssueInput {
            title: "a".repeat(ISSUE_TITLE_MAX_CHARS + 1),
            description: String::new(),
        }
        .validate()
        .unwrap_err();
        assert_eq!(error.field, Field::Title);
    }

    #[test]
    fn update_requires_a_field_and_distinguishes_clear_from_unchanged() {
        let error = UpdateIssueInput::default().validate().unwrap_err();
        assert_eq!(error.field, Field::Input);

        let clear = UpdateIssueInput {
            description: Some(String::new()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(clear.description(), Some(""));
        assert_eq!(clear.title(), None);
        assert_eq!(clear.status(), None);

        let status_only = UpdateIssueInput {
            status: Some(IssueStatus::Done),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(status_only.status(), Some(IssueStatus::Done));

        let blank_title = UpdateIssueInput {
            title: Some("   ".into()),
            ..Default::default()
        }
        .validate()
        .unwrap_err();
        assert_eq!(blank_title.field, Field::Title);
    }
}
