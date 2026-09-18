use crate::{
    api::auth_status,
    components::{AccountMenu, AuthPage},
    demo,
    models::User,
    routes::Route,
};
use dioxus::prelude::*;

/// Router layout: checks the session before rendering any workspace page, so every
/// route (and deep link) shows sign-in first and then the requested page.
#[component]
pub fn AppShell() -> Element {
    // Remounting the boundary retries a failed initial status check.
    let mut attempt = use_signal(|| 0_u32);
    rsx! {
        ErrorBoundary { key: "{attempt}",
            handle_error: move |_| rsx! {
                div { class: "auth-screen",
                    main { class: "auth-card", id: "main-content",
                        h1 { "Rivet is unavailable" }
                        p { role: "alert", class: "error-message", "Could not reach the server. Check that it is running, then retry." }
                        button { onclick: move |_| attempt += 1, "Retry" }
                    }
                }
            },
            SuspenseBoundary { fallback: |_| rsx! { p { class: "loading", role: "status", "Loading Rivet..." } },
                SessionGate {}
            }
        }
    }
}

#[component]
fn SessionGate() -> Element {
    let mut status = use_loader(auth_status)?;
    let current = status.read().clone();
    match current.user {
        Some(user) => rsx! { Workspace { user, on_signed_out: move |_| status.restart() } },
        None => rsx! {
            AuthPage { registration_open: current.registration_open, on_signed_in: move |_| status.restart() }
        },
    }
}

#[component]
fn Workspace(user: User, on_signed_out: EventHandler<()>) -> Element {
    let route = use_route::<Route>();
    let selected_project = match &route {
        Route::ProjectDetail { project_id } => Some(*project_id),
        Route::IssueDetail { issue_id } => demo::issue(*issue_id).map(|issue| issue.project_id),
        _ => None,
    };
    let location = match &route {
        Route::Home {} => "Overview",
        Route::Projects {} => "Projects",
        Route::ProjectDetail { .. } => "Project board",
        Route::IssueDetail { .. } => "Issue details",
        Route::Connection {} => "Connection check",
        Route::NotFound { .. } => "Page not found",
    };
    let initial = user
        .email
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to content" }
        div { class: "workspace",
            aside { class: "sidebar",
                Link { class: "brand", to: Route::Home {}, span { class: "brand-mark", aria_hidden: "true", "r" } "rivet" }
                div { class: "workspace-name", span { class: "workspace-avatar", aria_hidden: "true", "{initial}" } "Personal workspace" }
                nav { aria_label: "Workspace navigation",
                    Link { class: if matches!(route, Route::Home {}) { "nav-link active" } else { "nav-link" }, to: Route::Home {}, aria_current: if matches!(route, Route::Home {}) { "page" } else { "false" }, span { class: "nav-symbol", aria_hidden: "true", "◈" } "Overview" }
                    Link { class: if matches!(route, Route::Projects {}) { "nav-link active" } else { "nav-link" }, to: Route::Projects {}, aria_current: if matches!(route, Route::Projects {}) { "page" } else { "false" }, span { class: "nav-symbol", aria_hidden: "true", "▦" } "All projects" }
                }
                div { class: "sidebar-label", "PROJECTS" }
                nav { class: "project-nav", aria_label: "Project navigation",
                    for project in demo::PROJECTS {
                        Link { key: "{project.id}", class: if selected_project == Some(project.id) { "nav-link active" } else { "nav-link" }, to: Route::ProjectDetail { project_id: project.id }, aria_current: if selected_project == Some(project.id) { "true" } else { "false" },
                            span { class: "project-dot {project.color}", aria_hidden: "true" }
                            "{project.name}"
                        }
                    }
                }
                Link { class: "nav-link add-project", to: Route::Projects {}, "+ New project" }
                div { class: "sidebar-footer",
                    Link { class: "nav-link", to: Route::Connection {}, "Connection check" }
                    AccountMenu { user, on_signed_out }
                }
            }
            div { class: "main-area",
                header { class: "topbar",
                    div { class: "breadcrumbs", span { "Workspace" } span { aria_hidden: "true", "/" } strong { "{location}" } }
                    span { class: "preview-pill", "Preview workspace" }
                }
                main { id: "main-content", tabindex: "-1", class: "page-content",
                    div { class: "preview-notice", "Sample data · Form previews are not saved." }
                    Outlet::<Route> {}
                }
            }
        }
    }
}
