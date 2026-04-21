use crate::prelude::*;

/// qBittorrent connection options shared by any command that talks to qBittorrent.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QbitOptions {
    /// qBittorrent API base URL
    ///
    /// The base URL for your qBittorrent instance
    ///
    /// Examples: `http://localhost:8080`, `http://qbit`, `https://qbit.example.com`
    ///
    /// Or, the proxy URL with key if using [qui reverse proxy](https://getqui.com/docs/features/reverse-proxy)
    ///
    /// Examples:
    /// - `http://localhost:7476/proxy/YOUR_CLIENT_PROXY_KEY`
    /// - `https://qui.example.com/proxy/YOUR_CLIENT_PROXY_KEY`
    #[arg(long)]
    pub qbit_url: Option<String>,

    /// qBittorrent username.
    ///
    /// Not required when using qui reverse proxy.
    #[arg(long)]
    pub qbit_username: Option<String>,

    /// qBittorrent password.
    ///
    /// Not required when using qui reverse proxy.
    #[arg(long)]
    pub qbit_password: Option<String>,
}

impl QbitOptions {
    /// Push [`OptionRule`] violations for missing connection fields.
    pub(crate) fn validate_connection(&self, errors: &mut Vec<OptionRule>) {
        if self.qbit_url.is_none() {
            errors.push(OptionRule::NotSet("qBittorrent URL".to_owned()));
        }
        if self.requires_credentials() {
            if self.qbit_username.is_none() {
                errors.push(OptionRule::NotSet("qBittorrent username".to_owned()));
            }
            if self.qbit_password.is_none() {
                errors.push(OptionRule::NotSet("qBittorrent password".to_owned()));
            }
        }
    }

    /// Whether [`qbit_username`](Self::qbit_username) and
    /// [`qbit_password`](Self::qbit_password) must be set.
    ///
    /// - Returns `false` if using [qui reverse proxy](https://getqui.com/docs/features/reverse-proxy)
    /// - Returns `true` otherwise
    fn requires_credentials(&self) -> bool {
        !self
            .qbit_url
            .as_ref()
            .is_some_and(|url| url.contains("/proxy/"))
    }

    /// Create a [`QbitOptions`] with mock values for testing.
    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            qbit_url: Some("http://127.0.0.1:8080".to_owned()),
            qbit_username: Some("user".to_owned()),
            qbit_password: Some("hunter2".to_owned()),
        }
    }
}

impl OptionsContract for QbitOptions {
    type Partial = QbitOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(url) = &self.qbit_url
            && !url.starts_with("https://")
            && !url.starts_with("http://")
        {
            errors.push(OptionRule::UrlNotHttp(
                "qBittorrent URL".to_owned(),
                url.clone(),
            ));
        }
        if let Some(url) = &self.qbit_url
            && url.ends_with('/')
        {
            errors.push(OptionRule::UrlInvalidSuffix(
                "qBittorrent URL".to_owned(),
                url.clone(),
            ));
        }
    }
}
