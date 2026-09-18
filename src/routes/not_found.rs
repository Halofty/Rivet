use crate::{components::EmptyState, routes::Route};
use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));
    rsx! {
        document::Title { "Page not found · Rivet" }
        p { class: "eyebrow", "404" }
        EmptyState { title: "This path leads nowhere", description: format!("There is no page at {path}.") }
        Link { class: "button", to: Route::Projects {}, "Back to projects" }
    }
}
