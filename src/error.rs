//! Error types for LazyFile.

use thiserror::Error;

/// LazyFile error type.
#[derive(Error, Debug)]
pub enum LazyFileError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Rclone API error.
    #[error("rclone API error on {endpoint}: {message}")]
    RcloneApi {
        endpoint: &'static str,
        message: String,
    },

    /// HTTP request error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, LazyFileError>;
