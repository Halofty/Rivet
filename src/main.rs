fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "server")]
    dioxus::serve(|| async { Ok(rivet::server::router()) });

    #[cfg(not(feature = "server"))]
    {
        #[cfg(feature = "desktop")]
        rivet::platform::desktop::configure_server_url()?;

        dioxus::launch(rivet::App);
        Ok(())
    }
}
