/// Configuration for creating an Otari client.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// The API key (if not set, will try environment variable).
    pub api_key: Option<String>,

    /// The API base URL (for custom endpoints/proxies).
    pub api_base: Option<String>,

    /// Additional configuration.
    pub extra: std::collections::HashMap<String, String>,
}

impl Config {
    /// Create a new config with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Set the API base URL.
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = Some(api_base.into());
        self
    }

    /// Add an extra configuration value.
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}
