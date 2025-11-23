//! Error types for LazyFile.

use thiserror::Error;

/// LazyFile error type.
#[derive(Error, Debug)]
pub enum LazyFileError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Rclone API error.
    #[error("Rclone API error: {0}")]
    RcloneApi(String),

    /// HTTP request error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Terminal error (reserved for future use).
    #[error("Terminal error: {0}")]
    #[allow(dead_code)]
    Terminal(String),

    /// Configuration error (reserved for future use).
    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    Config(String),

    /// Tracing filter parse error.
    #[error("Tracing filter error: {0}")]
    TracingFilter(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    #[allow(dead_code)]
    Auth(String),

    /// Keyring error.
    #[error("Keyring error: {0}")]
    Keyring(String),

    /// Unauthorized (401) error.
    #[error("Unauthorized: authentication required")]
    Unauthorized,

    /// Generic error (reserved for future use).
    #[error("{0}")]
    #[allow(dead_code)]
    Other(String),
}

impl From<tracing_subscriber::filter::ParseError> for LazyFileError {
    fn from(err: tracing_subscriber::filter::ParseError) -> Self {
        LazyFileError::TracingFilter(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, LazyFileError>;
