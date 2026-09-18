use crate::api::{ApiError, echo_message, get_status};
use dioxus::prelude::*;

#[component]
pub fn Connection() -> Element {
    // Form state lives outside the remounted loader boundary, so retry keeps the draft.
    let mut attempt = use_signal(|| 0_u32);
    let mut message = use_signal(String::new);
    let mut send = use_action(echo_message);
    let pending = send.pending();
    let response = send.value();
    rsx! {
        document::Title { "Connection check · Rivet" }
        div { class: "page-heading", div { p { class: "eyebrow", "DEVELOPMENT" } h1 { "Connection check" } p { class: "lead", "Verify the shared backend. Messages are not saved." } } }
        section { class: "form-panel",
            ErrorBoundary { key: "{attempt}", handle_error: move |_| rsx! {
                p { role: "alert", class: "error-message", "Could not load the server. Check that it is running, then retry." }
                button { onclick: move |_| attempt += 1, "Retry connection" }
            },
                SuspenseBoundary { fallback: |_| rsx! { p { role: "status", "Connecting to Rivet..." } }, ServerStatus {} }
            }
            form { class: "connection-form", onsubmit: move |event| { event.prevent_default(); if !send.pending() { send.call(message()); } },
                label { r#for: "message", "Message" }
                input { id: "message", name: "message", value: "{message}", placeholder: "Hello, Rivet", disabled: pending,
                    oninput: move |event| { message.set(event.value()); send.reset(); },
                }
                button { r#type: "submit", disabled: pending, if pending { "Sending..." } else { "Send message" } }
            }
            div { class: "feedback", aria_live: "polite",
                match response {
                    Some(Ok(reply)) => rsx! { p { role: "status", "{reply.read()}" } },
                    Some(Err(error)) => {
                        let text = error.downcast_ref::<ApiError>().map(ToString::to_string).unwrap_or_else(|| "The request failed. Please try again.".to_owned());
                        rsx! { p { class: "error-message", role: "alert", "{text}" } }
                    }
                    None => rsx! {},
                }
            }
        }
    }
}

#[component]
fn ServerStatus() -> Element {
    let mut status = use_loader(get_status)?;
    rsx! {
        div { class: "status-row", h2 { "{status.read().application} is connected" }
            button { class: "secondary", disabled: status.loading(), onclick: move |_| status.restart(), if status.loading() { "Checking..." } else { "Check again" } }
        }
    }
}
