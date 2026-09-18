use crate::{
    components::{ProjectForm, ProjectList},
    demo,
};
use dioxus::prelude::*;

#[component]
pub fn Projects() -> Element {
    let mut creating = use_signal(|| false);
    let mut notice = use_signal(|| None::<String>);
    rsx! {
        document::Title { "Projects · Rivet" }
        div { class: "page-heading",
            div { p { class: "eyebrow", "A PLACE FOR EVERY IDEA" } h1 { "Projects" } p { class: "lead", "Small, focused spaces for the things you want to build." } }
            button { disabled: creating(), onclick: move |_| { notice.set(None); creating.set(true); }, "+ New project" }
        }
        if creating() {
            ProjectForm {
                on_preview: move |name| notice.set(Some(format!("Preview ready: {name}. No project was created."))),
                on_cancel: move |_| { creating.set(false); notice.set(None); },
            }
        }
        if let Some(message) = notice() { p { class: "notice", role: "status", "{message}" } }
        div { class: "section-heading", h2 { "All projects" } span { class: "muted", "{demo::PROJECTS.len()} sample projects" } }
        ProjectList { projects: demo::PROJECTS.to_vec() }
    }
}
