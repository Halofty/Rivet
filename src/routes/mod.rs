use crate::components::AppShell;
use dioxus::prelude::*;

mod connection;
mod home;
mod issue_detail;
mod not_found;
mod project_detail;
mod projects;

use connection::Connection;
use home::Home;
use issue_detail::IssueDetail;
use not_found::NotFound;
use project_detail::ProjectDetail;
use projects::Projects;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},
    #[route("/projects")]
    Projects {},
    #[route("/projects/:project_id")]
    ProjectDetail { project_id: i64 },
    #[route("/issues/:issue_id")]
    IssueDetail { issue_id: i64 },
    #[route("/dev/connection")]
    Connection {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
