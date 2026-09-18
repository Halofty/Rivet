//! Fixed milestone 1 UI fixtures, not domain models or persistent state.
//! Replace with real loaders in milestones 3 and 4.
use crate::models::IssueStatus;

#[derive(Clone, Copy, PartialEq)]
pub struct PreviewProject {
    pub id: i64,
    pub name: &'static str,
    pub description: &'static str,
    pub initials: &'static str,
    pub color: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub struct PreviewIssue {
    pub id: i64,
    pub project_id: i64,
    pub title: &'static str,
    pub description: &'static str,
    pub status: IssueStatus,
}

pub const PROJECTS: [PreviewProject; 3] = [
    PreviewProject {
        id: 1,
        name: "Rivet workspace",
        description: "A focused home for projects, ideas, and the work that moves them forward.",
        initials: "RW",
        color: "violet",
    },
    PreviewProject {
        id: 2,
        name: "Desktop experience",
        description: "Make the everyday workflow feel at home on the desktop.",
        initials: "DE",
        color: "blue",
    },
    PreviewProject {
        id: 3,
        name: "Next chapter",
        description: "An open space for the next small thing worth building.",
        initials: "NC",
        color: "green",
    },
];

pub const ISSUES: [PreviewIssue; 6] = [
    PreviewIssue {
        id: 1,
        project_id: 1,
        title: "Give every project a clear starting point",
        description: "Keep the project overview simple: a name, a short description, and a place to find the next piece of work.",
        status: IssueStatus::Todo,
    },
    PreviewIssue {
        id: 2,
        project_id: 1,
        title: "Make empty states useful",
        description: "Explain what belongs here and show one clear next action.",
        status: IssueStatus::Todo,
    },
    PreviewIssue {
        id: 3,
        project_id: 1,
        title: "Build the shared workspace shell",
        description: "Bring the sidebar, navigation, and main content together with consistent spacing and keyboard focus.",
        status: IssueStatus::InProgress,
    },
    PreviewIssue {
        id: 4,
        project_id: 1,
        title: "Define the first three issue statuses",
        description: "Start with Todo, In Progress, and Done. Keep workflow rules small and explicit.",
        status: IssueStatus::Done,
    },
    PreviewIssue {
        id: 5,
        project_id: 2,
        title: "Check keyboard navigation",
        description: "Move between navigation, cards, and forms without using a mouse.",
        status: IssueStatus::InProgress,
    },
    PreviewIssue {
        id: 6,
        project_id: 2,
        title: "Review compact window layouts",
        description: "Keep navigation and issue columns usable when the window becomes narrow.",
        status: IssueStatus::Todo,
    },
];

pub fn project(id: i64) -> Option<PreviewProject> {
    PROJECTS.into_iter().find(|project| project.id == id)
}
pub fn issue(id: i64) -> Option<PreviewIssue> {
    ISSUES.into_iter().find(|issue| issue.id == id)
}
pub fn project_issues(id: i64) -> Vec<PreviewIssue> {
    ISSUES
        .into_iter()
        .filter(|issue| issue.project_id == id)
        .collect()
}
