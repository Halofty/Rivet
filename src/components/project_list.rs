use crate::{demo::PreviewProject, routes::Route};
use dioxus::prelude::*;

#[component]
pub fn ProjectList(projects: Vec<PreviewProject>) -> Element {
    rsx! {
        div { class: "project-grid",
            for project in projects {
                Link { key: "{project.id}", class: "project-card", to: Route::ProjectDetail { project_id: project.id },
                    div { class: "project-card-top", span { class: "project-icon {project.color}", aria_hidden: "true", "{project.initials}" } span { class: "card-arrow", aria_hidden: "true", "↗" } }
                    h2 { "{project.name}" }
                    p { "{project.description}" }
                    span { class: "card-footer", "Open project" span { aria_hidden: "true", " →" } }
                }
            }
        }
    }
}
