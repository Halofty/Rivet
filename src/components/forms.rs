use crate::models::{CreateIssueInput, CreateProjectInput, IssueStatus};
use dioxus::prelude::*;

/// UI-only draft; forms still preview locally until server flows arrive in milestones 3 and 4.
#[derive(Clone, PartialEq)]
pub struct IssueDraft {
    pub title: String,
    pub description: String,
    pub status: IssueStatus,
}

#[component]
pub fn ProjectForm(on_preview: EventHandler<String>, on_cancel: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        form { class: "form-panel", onsubmit: move |event| {
            event.prevent_default();
            match (CreateProjectInput { name: name(), description: description() }).validate() {
                Ok(valid) => { error.set(None); on_preview.call(valid.name().to_owned()); }
                Err(invalid) => error.set(Some(invalid.message)),
            }
        },
            h2 { "New project" }
            p { class: "muted", "Try the form. This preview will not create a project." }
            label { r#for: "project-name", "Project name" }
            input { id: "project-name", name: "name", value: "{name}", placeholder: "What are you working on?", oninput: move |event| name.set(event.value()) }
            label { r#for: "project-description", "Description" span { class: "optional", "Optional" } }
            textarea { id: "project-description", name: "description", rows: "3", value: "{description}", placeholder: "A little context goes a long way.", oninput: move |event| description.set(event.value()) }
            if let Some(message) = error() { p { class: "error-message", role: "alert", "{message}" } }
            div { class: "form-actions",
                button { r#type: "button", class: "secondary", onclick: move |_| on_cancel.call(()), "Cancel" }
                button { r#type: "submit", "Preview project" }
            }
        }
    }
}

#[component]
pub fn IssueForm(
    initial: IssueDraft,
    on_preview: EventHandler<IssueDraft>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| initial.title);
    let mut description = use_signal(|| initial.description);
    let mut status = use_signal(|| initial.status);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        form { class: "form-panel", onsubmit: move |event| {
            event.prevent_default();
            match (CreateIssueInput { title: title(), description: description() }).validate() {
                Ok(valid) => {
                    error.set(None);
                    on_preview.call(IssueDraft { title: valid.title().to_owned(), description: valid.description().to_owned(), status: status() });
                }
                Err(invalid) => error.set(Some(invalid.message)),
            }
        },
            p { class: "muted", "Changes are a local preview. Nothing is saved to the workspace." }
            label { r#for: "issue-title", "Title" }
            input { id: "issue-title", name: "title", value: "{title}", oninput: move |event| title.set(event.value()) }
            label { r#for: "issue-description", "Description" span { class: "optional", "Optional" } }
            textarea { id: "issue-description", name: "description", rows: "5", value: "{description}", oninput: move |event| description.set(event.value()) }
            label { r#for: "issue-status", "Status" }
            select { id: "issue-status", name: "status", value: status().as_str(), onchange: move |event| {
                if let Ok(next) = event.value().parse() { status.set(next); }
            },
                for choice in IssueStatus::ALL { option { key: "{choice.as_str()}", value: choice.as_str(), selected: choice == status(), "{choice.label()}" } }
            }
            if let Some(message) = error() { p { class: "error-message", role: "alert", "{message}" } }
            div { class: "form-actions",
                button { r#type: "button", class: "secondary", onclick: move |_| on_cancel.call(()), "Cancel" }
                button { r#type: "submit", "Preview issue" }
            }
        }
    }
}
