fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        let state = rivet::server::AppState::from_env().await?;
        Ok(rivet::server::router(state))
    });

    #[cfg(not(feature = "server"))]
    {
        #[cfg(feature = "desktop")]
        rivet::platform::desktop::configure_server_url()?;

        dioxus::launch(rivet::App);
        Ok(())
    }
}
