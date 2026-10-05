//! Server settings read from environment variables.

/// Shortest API key the server accepts.
const MIN_KEY_LEN: usize = 16;
/// Address used when `BIND_ADDR` is not set.
const DEFAULT_BIND: &str = "127.0.0.1:8080";

/// Why the settings could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required variable is missing.
    #[error("environment variable {0} is not set")]
    Missing(&'static str),
    /// The API key is too short to be safe.
    #[error("SHIPCHECK_API_KEY must be at least {0} characters")]
    WeakKey(usize),
}

/// Everything the server needs to start.
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub bind_addr: String,
}

impl Config {
    /// Reads the settings from the process environment.
    ///
    /// # Errors
    /// Fails when `DATABASE_URL` or `SHIPCHECK_API_KEY` is missing, or the key is too short.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Reads the settings through a lookup function, which keeps tests off the real environment.
    ///
    /// # Errors
    /// Fails when `DATABASE_URL` or `SHIPCHECK_API_KEY` is missing, or the key is too short.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let database_url = get("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;
        let api_key = get("SHIPCHECK_API_KEY").ok_or(ConfigError::Missing("SHIPCHECK_API_KEY"))?;
        if api_key.len() < MIN_KEY_LEN {
            return Err(ConfigError::WeakKey(MIN_KEY_LEN));
        }
        let bind_addr = get("BIND_ADDR").unwrap_or_else(|| DEFAULT_BIND.to_owned());
        Ok(Self {
            database_url,
            api_key,
            bind_addr,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn config_from(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        Config::from_lookup(|key| map.get(key).cloned())
    }

    const KEY: &str = "0123456789abcdef0123";

    #[test]
    fn missing_database_url_is_reported() {
        let result = config_from(&[("SHIPCHECK_API_KEY", KEY)]);
        assert!(matches!(result, Err(ConfigError::Missing("DATABASE_URL"))));
    }

    #[test]
    fn missing_api_key_is_reported() {
        let result = config_from(&[("DATABASE_URL", "postgres://x")]);
        assert!(matches!(
            result,
            Err(ConfigError::Missing("SHIPCHECK_API_KEY"))
        ));
    }

    #[test]
    fn short_api_key_is_rejected() {
        let result = config_from(&[
            ("DATABASE_URL", "postgres://x"),
            ("SHIPCHECK_API_KEY", "short"),
        ]);
        assert!(matches!(result, Err(ConfigError::WeakKey(_))));
    }

    #[test]
    fn bind_address_has_a_default_and_can_be_overridden() {
        let base = [("DATABASE_URL", "postgres://x"), ("SHIPCHECK_API_KEY", KEY)];
        let default = config_from(&base).unwrap();
        assert_eq!(default.bind_addr, DEFAULT_BIND);
        let custom = config_from(&[base[0], base[1], ("BIND_ADDR", "0.0.0.0:9000")]).unwrap();
        assert_eq!(custom.bind_addr, "0.0.0.0:9000");
    }
}
