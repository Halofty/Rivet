use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn EmptyState(title: String, description: String) -> Element {
    rsx! { div { class: "empty-state", span { class: "empty-mark", aria_hidden: "true", "◇" } h2 { "{title}" } p { "{description}" } } }
}

#[component]
pub fn MissingRecord(kind: String) -> Element {
    rsx! {
        document::Title { "Not found · Rivet" }
        EmptyState { title: format!("{kind} not found"), description: "This link does not match an item in the preview workspace." }
        Link { class: "button secondary", to: Route::Projects {}, "Back to projects" }
    }
}
