use super::users::Registration;

const PUBLIC_ORIGIN_VAR: &str = "RIVET_PUBLIC_ORIGIN";
const ALLOW_REGISTRATION_VAR: &str = "RIVET_ALLOW_REGISTRATION";
const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{PUBLIC_ORIGIN_VAR} must look like https://rivet.example.com (no path), got `{0}`")]
    PublicOrigin(String),
    #[error("{ALLOW_REGISTRATION_VAR} must be `true` or `false`, got `{0}`")]
    AllowRegistration(String),
    #[error("{0} is not valid Unicode")]
    NotUnicode(&'static str),
}

/// Deployment settings that affect request trust. Without a public origin the server
/// only answers loopback host names, which blocks DNS-rebinding attacks from web pages.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub registration: Registration,
    public_origin: Option<PublicOrigin>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PublicOrigin {
    /// Lowercase `scheme://host[:port]`, compared exactly with the `Origin` header.
    origin: String,
    host: String,
    https: bool,
}

impl ServerConfig {
    /// Loopback-only access with first-user registration.
    pub fn local() -> Self {
        Self {
            registration: Registration::FirstUserOnly,
            public_origin: None,
        }
    }

    pub fn with_public_origin(mut self, origin: &str) -> Result<Self, ConfigError> {
        self.public_origin = Some(parse_public_origin(origin)?);
        Ok(self)
    }

    /// Reads `RIVET_PUBLIC_ORIGIN` and `RIVET_ALLOW_REGISTRATION`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::local();
        if let Some(origin) = env_var(PUBLIC_ORIGIN_VAR)? {
            config = config.with_public_origin(&origin)?;
        }
        if let Some(value) = env_var(ALLOW_REGISTRATION_VAR)? {
            config.registration = match value.trim() {
                "true" | "1" => Registration::Open,
                "false" | "0" => Registration::FirstUserOnly,
                _ => return Err(ConfigError::AllowRegistration(value)),
            };
        }
        Ok(config)
    }

    /// Whether a `Host` header value names this server.
    pub fn host_allowed(&self, authority: &str) -> bool {
        let host = host_of(authority);
        is_loopback(&host)
            || self
                .public_origin
                .as_ref()
                .is_some_and(|public| public.host == host)
    }

    /// Whether a browser `Origin` may make state-changing requests.
    pub fn origin_allowed(&self, origin: &str) -> bool {
        let origin = origin.trim().to_ascii_lowercase();
        if self
            .public_origin
            .as_ref()
            .is_some_and(|public| public.origin == origin)
        {
            return true;
        }
        match origin.split_once("://") {
            Some(("http" | "https", authority)) => is_loopback(&host_of(authority)),
            _ => false,
        }
    }

    /// Whether a request arrived under a loopback host name, meaning it came from this machine
    /// rather than through the public origin.
    pub fn is_loopback_host(&self, authority: &str) -> bool {
        is_loopback(&host_of(authority))
    }

    /// Session cookies are marked `Secure` when the public origin uses HTTPS.
    pub fn secure_cookies(&self) -> bool {
        self.public_origin
            .as_ref()
            .is_some_and(|public| public.https)
    }
}

fn env_var(name: &'static str) -> Result<Option<String>, ConfigError> {
    match std::env::var(name) {
        Ok(value) if value.trim().is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode(name)),
    }
}

fn parse_public_origin(value: &str) -> Result<PublicOrigin, ConfigError> {
    let invalid = || ConfigError::PublicOrigin(value.to_owned());
    let origin = value.trim().trim_end_matches('/').to_ascii_lowercase();
    let (scheme, authority) = origin.split_once("://").ok_or_else(invalid)?;
    let https = match scheme {
        "https" => true,
        "http" => false,
        _ => return Err(invalid()),
    };
    if authority.is_empty()
        || authority
            .chars()
            .any(|c| matches!(c, '/' | '?' | '#' | '@') || c.is_whitespace())
    {
        return Err(invalid());
    }
    Ok(PublicOrigin {
        host: host_of(authority),
        origin,
        https,
    })
}

/// The lowercase host part of `host[:port]`, keeping IPv6 brackets.
fn host_of(authority: &str) -> String {
    let authority = authority.trim();
    let host = match authority.strip_prefix('[') {
        Some(rest) => rest
            .split_once(']')
            .map_or(authority, |(inner, _)| &authority[..inner.len() + 2]),
        None => authority.split(':').next().unwrap_or_default(),
    };
    host.to_ascii_lowercase()
}

fn is_loopback(host: &str) -> bool {
    LOOPBACK_HOSTS.contains(&host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_config_accepts_only_loopback_hosts_and_origins() {
        let config = ServerConfig::local();
        for host in ["localhost:8080", "127.0.0.1", "[::1]:8080", "LOCALHOST"] {
            assert!(config.host_allowed(host), "{host}");
        }
        for host in [
            "evil.example",
            "127.0.0.1.evil.example",
            "localhost.evil:80",
            "",
        ] {
            assert!(!config.host_allowed(host), "{host}");
        }
        assert!(config.origin_allowed("http://127.0.0.1:8080"));
        assert!(!config.origin_allowed("https://evil.example"));
        assert!(!config.origin_allowed("null"));
        assert!(!config.origin_allowed("file://localhost"));
        assert!(!config.secure_cookies());
    }

    #[test]
    fn public_origin_extends_trust_exactly() {
        let config = ServerConfig::local()
            .with_public_origin("https://Rivet.Example.com/")
            .unwrap();
        assert!(config.host_allowed("rivet.example.com"));
        assert!(config.host_allowed("rivet.example.com:443"));
        assert!(!config.is_loopback_host("rivet.example.com"));
        assert!(config.origin_allowed("https://rivet.example.com"));
        assert!(!config.origin_allowed("http://rivet.example.com"));
        assert!(!config.origin_allowed("https://rivet.example.com.evil.example"));
        assert!(config.secure_cookies());

        for invalid in [
            "rivet.example.com",
            "ftp://x",
            "https://",
            "https://x/path",
            "https://u@x",
        ] {
            assert!(
                ServerConfig::local().with_public_origin(invalid).is_err(),
                "{invalid}"
            );
        }
    }
}
