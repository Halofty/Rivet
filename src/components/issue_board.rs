use crate::{demo::PreviewIssue, models::IssueStatus, routes::Route};
use dioxus::prelude::*;

#[component]
pub fn StatusBadge(status: IssueStatus) -> Element {
    rsx! { span { class: "status-badge {status.as_str()}", span { class: "status-dot", aria_hidden: "true" } "{status.label()}" } }
}

#[component]
pub fn IssueBoard(issues: Vec<PreviewIssue>) -> Element {
    rsx! {
        div { class: "issue-board",
            for status in IssueStatus::ALL {
                section { key: "{status.as_str()}", class: "board-column", aria_label: status.label(),
                    div { class: "column-header", h2 { StatusBadge { status } } span { class: "count", "{issues.iter().filter(|issue| issue.status == status).count()}" } }
                    div { class: "column-cards",
                        for issue in issues.iter().filter(|issue| issue.status == status) { IssueCard { key: "{issue.id}", issue: *issue } }
                        if !issues.iter().any(|issue| issue.status == status) { p { class: "column-empty", "No issues here yet" } }
                    }
                }
            }
        }
    }
}

#[component]
fn IssueCard(issue: PreviewIssue) -> Element {
    rsx! {
        Link { class: "issue-card", to: Route::IssueDetail { issue_id: issue.id },
            span { class: "issue-id", "Issue #{issue.id}" }
            h3 { "{issue.title}" }
            p { "{issue.description}" }
            div { class: "issue-card-footer", StatusBadge { status: issue.status } span { aria_hidden: "true", "↗" } }
        }
    }
}
