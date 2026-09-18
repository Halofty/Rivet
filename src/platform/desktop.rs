use dioxus::fullstack::{
    get_server_url,
    reqwest::{
        Url,
        header::{AUTHORIZATION, HeaderMap, HeaderValue},
    },
    set_request_headers,
};

const CREDENTIAL_SERVICE: &str = "Rivet";
const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// Override the DX development URL when running against an explicit backend.
///
/// Session tokens travel with every request, so a non-loopback backend must use HTTPS.
pub fn configure_server_url() -> Result<(), Box<dyn std::error::Error>> {
    let value = match std::env::var("RIVET_SERVER_URL") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return restore_session(),
        Err(error) => return Err(error.into()),
    };
    let url = Url::parse(&value)?;
    let Some(host) = url.host_str() else {
        return Err("RIVET_SERVER_URL must be an absolute HTTP or HTTPS URL".into());
    };
    match url.scheme() {
        "https" => {}
        "http" if LOOPBACK_HOSTS.contains(&host) => {}
        "http" => {
            return Err("RIVET_SERVER_URL must use HTTPS unless it is a loopback address".into());
        }
        _ => return Err("RIVET_SERVER_URL must be an absolute HTTP or HTTPS URL".into()),
    }

    // Dioxus requires a static URL. This single startup allocation lives for the process.
    dioxus::fullstack::set_server_url(value.trim_end_matches('/').to_owned().leak());
    restore_session()
}

/// Reattaches a token saved by a previous sign-in. A missing or unreadable credential
/// simply leaves the app signed out.
fn restore_session() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(token) = credential().and_then(|entry| entry.get_password()) {
        attach_token(&token);
    }
    Ok(())
}

/// Saves the desktop session token in the OS credential store (Windows Credential
/// Manager) and sends it with every later request.
pub fn store_session(token: &str) -> Result<(), String> {
    credential()
        .and_then(|entry| entry.set_password(token))
        .map_err(|error| format!("Signed in, but the session could not be saved: {error}"))?;
    attach_token(token);
    Ok(())
}

/// Forgets the local token even if the server could not be reached to end the session.
pub fn clear_session() {
    set_request_headers(HeaderMap::new());
    if let Ok(entry) = credential() {
        let _ = entry.delete_credential();
    }
}

fn attach_token(token: &str) {
    if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, value);
        set_request_headers(headers);
    }
}

/// One credential per backend, so tokens for different servers never mix.
fn credential() -> keyring::Result<keyring::Entry> {
    let server = match get_server_url() {
        "" => "default",
        url => url,
    };
    keyring::Entry::new(CREDENTIAL_SERVICE, server)
}
