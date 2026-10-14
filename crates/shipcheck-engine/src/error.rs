/// Errors raised while loading rules and project files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file could not be read or written.
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// A file is not valid YAML or does not match the expected format.
    #[error("invalid YAML in {origin}: {source}")]
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
    /// A file is not valid JSON.
    #[error("invalid JSON in {origin}: {source}")]
    Json {
        origin: String,
        #[source]
        source: serde_json::Error,
    },
    /// A file parses but holds values that make no sense.
    #[error("invalid {origin}: {message}")]
    Invalid { origin: String, message: String },
}

/// Result type used across the engine.
pub type Result<T> = std::result::Result<T, Error>;
