//! Portable records, inputs, and validation shared by the web, desktop, and server builds.
//! Nothing here depends on SQLx, routes, components, or native APIs.

mod issue;
mod project;
mod user;
pub mod validation;

pub use issue::{CreateIssueInput, Issue, IssuePatch, IssueStatus, NewIssue, UpdateIssueInput};
pub use project::{CreateProjectInput, NewProject, Project};
pub use user::{ClientKind, User, normalize_email, validate_password};
pub use validation::{Field, ValidationError};
