use crate::{
    components::{IssueDraft, IssueForm, MissingRecord, StatusBadge},
    demo::{self, PreviewIssue},
    routes::Route,
};
use dioxus::prelude::*;

#[component]
pub fn IssueDetail(issue_id: i64) -> Element {
    match demo::issue(issue_id) {
        Some(issue) => rsx! { IssuePage { key: "{issue_id}", issue } },
        None => rsx! { MissingRecord { kind: "Issue" } },
    }
}

#[component]
fn IssuePage(issue: PreviewIssue) -> Element {
    let mut editing = use_signal(|| false);
    let mut previewed = use_signal(|| false);
    let mut draft = use_signal(|| IssueDraft {
        title: issue.title.to_owned(),
        description: issue.description.to_owned(),
        status: issue.status,
    });
    let project_name = demo::project(issue.project_id)
        .map(|p| p.name)
        .unwrap_or("Project");
    let current = draft();
    rsx! {
        document::Title { "{current.title} · Rivet" }
        Link { class: "back-link", to: Route::ProjectDetail { project_id: issue.project_id }, "← {project_name}" }
        div { class: "page-heading",
            div { p { class: "eyebrow", "ISSUE #{issue.id}" } h1 { "{current.title}" } }
            if !editing() { button { class: "secondary", onclick: move |_| editing.set(true), "Edit preview" } }
        }
        if previewed() { p { class: "notice", role: "status", "Local preview updated. The board is unchanged; leaving this page resets the preview." } }
        if editing() {
            IssueForm { initial: current,
                on_preview: move |next| { draft.set(next); previewed.set(true); editing.set(false); },
                on_cancel: move |_| editing.set(false),
            }
        } else {
            div { class: "issue-detail-grid",
                section { class: "description-panel", h2 { "Description" }
                    if current.description.is_empty() { p { class: "muted", "No description yet." } }
                    else { p { class: "issue-description", "{current.description}" } }
                }
                aside { class: "properties-panel", aria_label: "Issue properties",
                    h2 { "Properties" }
                    dl { dt { "Status" } dd { StatusBadge { status: current.status } } dt { "Project" } dd { Link { to: Route::ProjectDetail { project_id: issue.project_id }, "{project_name}" } } }
                }
            }
        }
    }
}
