/// Override the DX development URL when running against an explicit backend.
pub fn configure_server_url() -> Result<(), Box<dyn std::error::Error>> {
    let value = match std::env::var("RIVET_SERVER_URL") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let url = dioxus::fullstack::reqwest::Url::parse(&value)?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("RIVET_SERVER_URL must be an absolute HTTP or HTTPS URL".into());
    }

    // Dioxus requires a static URL. This single startup allocation lives for the process.
    dioxus::fullstack::set_server_url(value.leak());
    Ok(())
}
