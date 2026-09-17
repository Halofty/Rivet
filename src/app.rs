use dioxus::prelude::*;

use crate::api::{ApiError, echo_message, get_status};

/// Development scaffold for verifying the shared fullstack architecture.
#[component]
pub fn App() -> Element {
    let mut attempt = use_signal(|| 0_u32);

    rsx! {
        document::Title { "Rivet" }
        document::Stylesheet { href: asset!("/assets/main.css") }
        main {
            p { class: "eyebrow", "RIVET / MILESTONE 0" }
            h1 { "A foundation for your next project." }
            p { class: "intro", "The workspace starts here. First, a quick connection check." }
            section { class: "panel", aria_label: "Development connection check",
                ErrorBoundary {
                    key: "{attempt}",
                    handle_error: move |_| rsx! {
                        p { role: "alert", "Could not load the server. Check that it is running, then retry." }
                        button { onclick: move |_| attempt += 1, "Retry connection" }
                    },
                    SuspenseBoundary {
                        fallback: |_| rsx! { p { role: "status", "Connecting to Rivet..." } },
                        ConnectionCheck {}
                    }
                }
            }
            p { class: "footnote", "Development scaffold. Projects and issues arrive in the next milestones." }
        }
    }
}

#[component]
fn ConnectionCheck() -> Element {
    let mut status = use_loader(get_status)?;
    let mut message = use_signal(String::new);
    let mut send = use_action(echo_message);
    let pending = send.pending();
    let response = send.value();

    rsx! {
        div { class: "status-row",
            h2 { "{status.read().application} is connected" }
            button {
                class: "secondary",
                disabled: status.loading(),
                onclick: move |_| status.restart(),
                if status.loading() { "Checking..." } else { "Check again" }
            }
        }
        p { "Send a message to verify the connection. Nothing is saved." }
        form {
            onsubmit: move |event| {
                event.prevent_default();
                if !send.pending() {
                    send.call(message());
                }
            },
            label { r#for: "message", "Message" }
            input {
                id: "message",
                name: "message",
                value: "{message}",
                placeholder: "Hello, Rivet",
                disabled: pending,
                oninput: move |event| {
                    message.set(event.value());
                    send.reset();
                },
            }
            button {
                r#type: "submit",
                disabled: pending,
                if pending { "Sending..." } else { "Send message" }
            }
        }
        div { class: "feedback", aria_live: "polite",
            match response {
                Some(Ok(reply)) => rsx! { p { role: "status", "{reply.read()}" } },
                Some(Err(error)) => {
                    let text = error.downcast_ref::<ApiError>()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "The request failed. Please try again.".to_owned());
                    rsx! { p { role: "alert", "{text}" } }
                }
                None => rsx! {},
            }
        }
    }
}
