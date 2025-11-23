//! Authentication management for LazyFile.
//!
//! This module handles HTTP Basic Auth and Bearer token authentication
//! for the rclone RC daemon. Credentials can be stored securely in the
//! system keyring or configured per-remote.

pub mod credentials;
pub mod manager;

pub use credentials::{Credentials, CredentialsType};
pub use manager::{AuthManager, AuthMode};
