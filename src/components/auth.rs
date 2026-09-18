use crate::{
    api::{Credentials, login, logout, register},
    models::{ClientKind, User, normalize_email, validate_password},
};
use dioxus::prelude::*;

const CLIENT: ClientKind = if cfg!(all(feature = "desktop", not(feature = "server"))) {
    ClientKind::Desktop
} else {
    ClientKind::Web
};

/// Sign-in form, or first-account setup while the server allows registration.
#[component]
pub fn AuthPage(registration_open: bool, on_signed_in: EventHandler<()>) -> Element {
    let mut creating = use_signal(|| registration_open);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut pending = use_signal(|| false);
    let create = creating() && registration_open;

    let submit = move |event: FormEvent| {
        event.prevent_default();
        if pending() {
            return;
        }
        if let Err(message) = check(create, &email(), &password(), &confirm()) {
            error.set(Some(message));
            return;
        }
        error.set(None);
        pending.set(true);
        let credentials = Credentials {
            email: email(),
            password: password(),
            client: CLIENT,
        };
        spawn(async move {
            let result = if create {
                register(credentials).await
            } else {
                login(credentials).await
            };
            pending.set(false);
            password.set(String::new());
            confirm.set(String::new());
            match result {
                Ok(_signed_in) => {
                    #[cfg(all(feature = "desktop", not(feature = "server")))]
                    if let Some(token) = &_signed_in.token
                        && let Err(message) = crate::platform::desktop::store_session(token)
                    {
                        error.set(Some(message));
                        return;
                    }
                    on_signed_in.call(());
                }
                Err(failure) => error.set(Some(failure.to_string())),
            }
        });
    };

    let (heading, lead, action) = if create {
        (
            "Create your account",
            "Your projects stay private to this account on every device.",
            "Create account",
        )
    } else {
        (
            "Sign in to Rivet",
            "Use the same account on the web and the desktop app.",
            "Sign in",
        )
    };
    rsx! {
        document::Title { "{heading} · Rivet" }
        div { class: "auth-screen",
            main { class: "auth-card", id: "main-content",
                div { class: "brand", span { class: "brand-mark", aria_hidden: "true", "r" } "rivet" }
                h1 { "{heading}" }
                p { class: "lead", "{lead}" }
                form { class: "auth-form", onsubmit: submit,
                    label { r#for: "auth-email", "Email" }
                    input { id: "auth-email", r#type: "email", name: "email", autocomplete: "username", value: "{email}", disabled: pending(),
                        oninput: move |event| email.set(event.value()),
                    }
                    label { r#for: "auth-password", "Password" }
                    input { id: "auth-password", r#type: "password", name: "password", autocomplete: if create { "new-password" } else { "current-password" }, value: "{password}", disabled: pending(),
                        oninput: move |event| password.set(event.value()),
                    }
                    if create {
                        p { class: "field-hint", "At least 12 characters." }
                        label { r#for: "auth-confirm", "Confirm password" }
                        input { id: "auth-confirm", r#type: "password", name: "confirm", autocomplete: "new-password", value: "{confirm}", disabled: pending(),
                            oninput: move |event| confirm.set(event.value()),
                        }
                    }
                    if let Some(message) = error() { p { class: "error-message", role: "alert", "{message}" } }
                    button { r#type: "submit", disabled: pending(), if pending() { "Please wait..." } else { "{action}" } }
                }
                if registration_open {
                    button { class: "link-button", r#type: "button", disabled: pending(),
                        onclick: move |_| { creating.toggle(); error.set(None); },
                        if create { "I already have an account" } else { "Create an account" }
                    }
                }
            }
        }
    }
}

fn check(create: bool, email: &str, password: &str, confirm: &str) -> Result<(), String> {
    normalize_email(email).map_err(|error| error.message)?;
    if password.is_empty() {
        return Err("Enter your password.".to_owned());
    }
    if create {
        validate_password(password).map_err(|error| error.message)?;
        if password != confirm {
            return Err("The passwords do not match.".to_owned());
        }
    }
    Ok(())
}

/// The signed-in account and a sign-out action for the sidebar.
#[component]
pub fn AccountMenu(user: User, on_signed_out: EventHandler<()>) -> Element {
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        div { class: "account-menu",
            span { class: "account-email", title: "{user.email}", "{user.email}" }
            button { class: "secondary", r#type: "button", disabled: pending(),
                onclick: move |_| {
                    pending.set(true);
                    spawn(async move {
                        let result = logout().await;
                        // A desktop token is forgotten locally even if the server is unreachable.
                        #[cfg(all(feature = "desktop", not(feature = "server")))]
                        crate::platform::desktop::clear_session();
                        pending.set(false);
                        match result {
                            Ok(()) => on_signed_out.call(()),
                            Err(_) if CLIENT == ClientKind::Desktop => on_signed_out.call(()),
                            Err(failure) => error.set(Some(failure.to_string())),
                        }
                    });
                },
                if pending() { "Signing out..." } else { "Sign out" }
            }
            if let Some(message) = error() { p { class: "error-message", role: "alert", "{message}" } }
        }
    }
}
