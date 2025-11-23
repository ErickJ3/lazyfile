//! Credential types and structures for authentication.

use base64::Engine;
use serde::{Deserialize, Serialize};

/// Types of authentication supported by LazyFile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CredentialsType {
    /// HTTP Basic Authentication (username:password encoded in base64)
    Basic,
    /// Bearer token authentication
    Bearer,
}

/// Represents authentication credentials for rclone RC or individual remotes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Type of authentication
    pub auth_type: CredentialsType,
    /// Username (for Basic Auth) or empty for Bearer
    pub username: String,
    /// Password (for Basic Auth) or token (for Bearer)
    pub password: String,
    /// Optional: specific remote this credential is for (None = global daemon auth)
    pub remote: Option<String>,
}

impl Credentials {
    /// Create new Basic Auth credentials.
    pub fn basic(username: String, password: String, remote: Option<String>) -> Self {
        Self {
            auth_type: CredentialsType::Basic,
            username,
            password,
            remote,
        }
    }

    /// Create new Bearer token credentials.
    pub fn bearer(token: String, remote: Option<String>) -> Self {
        Self {
            auth_type: CredentialsType::Bearer,
            username: String::new(),
            password: token,
            remote,
        }
    }

    /// Get the Authorization header value for this credential.
    pub fn auth_header(&self) -> String {
        match self.auth_type {
            CredentialsType::Basic => {
                let combined = format!("{}:{}", self.username, self.password);
                let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
                format!("Basic {}", encoded)
            }
            CredentialsType::Bearer => {
                format!("Bearer {}", self.password)
            }
        }
    }

    /// Get a unique key for storing in keyring.
    pub fn keyring_key(&self) -> String {
        match &self.remote {
            Some(remote) => format!("lazyfile-{}", remote),
            None => "lazyfile-daemon".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_header() {
        let creds = Credentials::basic("user".to_string(), "pass".to_string(), None);
        let header = creds.auth_header();
        assert!(header.starts_with("Basic "));
        assert_eq!(header, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn test_bearer_auth_header() {
        let creds = Credentials::bearer("mytoken123".to_string(), None);
        let header = creds.auth_header();
        assert_eq!(header, "Bearer mytoken123");
    }

    #[test]
    fn test_keyring_key_global() {
        let creds = Credentials::basic("user".to_string(), "pass".to_string(), None);
        assert_eq!(creds.keyring_key(), "lazyfile-daemon");
    }

    #[test]
    fn test_keyring_key_remote() {
        let creds = Credentials::basic(
            "user".to_string(),
            "pass".to_string(),
            Some("gdrive".to_string()),
        );
        assert_eq!(creds.keyring_key(), "lazyfile-gdrive");
    }
}
