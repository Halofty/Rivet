use crate::{components::ProjectList, demo, routes::Route};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        document::Title { "Overview · Rivet" }
        section { class: "welcome",
            p { class: "eyebrow", "A LITTLE CLARITY GOES A LONG WAY" }
            h1 { "Good work starts" br {} "with a clear next step." }
            p { class: "lead", "Give your ideas a home. Break them into small pieces. Keep moving, one issue at a time." }
            Link { class: "button", to: Route::Projects {}, "Explore projects" span { aria_hidden: "true", " →" } }
        }
        div { class: "section-heading", h2 { "Your projects" } Link { to: Route::Projects {}, "View all →" } }
        ProjectList { projects: demo::PROJECTS.to_vec() }
        div { class: "workspace-note", span { aria_hidden: "true", "◇" } p { "A simple workflow. Todo, In Progress, Done. Just enough structure to keep your work moving." } }
    }
}
