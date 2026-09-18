use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        document::Title { "Rivet" }
        document::Stylesheet { href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
