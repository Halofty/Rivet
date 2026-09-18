use crate::{
    components::{EmptyState, IssueBoard, IssueDraft, IssueForm, MissingRecord},
    demo::{self, PreviewProject},
    models::IssueStatus,
    routes::Route,
};
use dioxus::prelude::*;

#[component]
pub fn ProjectDetail(project_id: i64) -> Element {
    match demo::project(project_id) {
        Some(project) => rsx! { ProjectPage { key: "{project_id}", project } },
        None => rsx! { MissingRecord { kind: "Project" } },
    }
}

#[component]
fn ProjectPage(project: PreviewProject) -> Element {
    let mut creating = use_signal(|| false);
    let mut notice = use_signal(|| None::<String>);
    let issues = demo::project_issues(project.id);
    rsx! {
        document::Title { "{project.name} · Rivet" }
        Link { class: "back-link", to: Route::Projects {}, "← All projects" }
        div { class: "page-heading",
            div { div { class: "title-with-icon", span { class: "project-icon {project.color}", aria_hidden: "true", "{project.initials}" } h1 { "{project.name}" } } p { class: "lead", "{project.description}" } }
            button { disabled: creating(), onclick: move |_| { creating.set(true); notice.set(None); }, "+ New issue" }
        }
        if creating() {
            IssueForm {
                initial: IssueDraft { title: String::new(), description: String::new(), status: IssueStatus::Todo },
                on_preview: move |draft: IssueDraft| notice.set(Some(format!("Preview ready: {} · {}. No issue was created.", draft.title, draft.status.label()))),
                on_cancel: move |_| { creating.set(false); notice.set(None); },
            }
        }
        if let Some(message) = notice() { p { class: "notice", role: "status", "{message}" } }
        div { class: "board-toolbar", h2 { "Board" } span { class: "muted", "{issues.len()} sample issues" } }
        if issues.is_empty() { EmptyState { title: "A fresh start", description: "This project has no sample issues. Try the New issue form to preview your first step." } }
        IssueBoard { issues }
    }
}
