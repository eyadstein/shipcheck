/// Errors raised while loading rules.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A rule file could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// A rule file is not valid YAML or does not match the rule format.
    #[error("invalid rule file {origin}: {source}")]
    Yaml {
        origin: String,
        #[source]
        source: serde_yaml::Error,
    },
    /// A rule holds a regular expression that does not compile.
    #[error("rule {id} has an invalid pattern: {source}")]
    Pattern {
        id: String,
        #[source]
        source: regex::Error,
    },
    /// The rules directory does not exist.
    #[error("rules directory not found: {0}")]
    MissingRulesDir(String),
}

/// Result type used across the engine.
pub type Result<T> = std::result::Result<T, Error>;
