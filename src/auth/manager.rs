//! Authentication manager for handling credentials storage and retrieval.

use super::credentials::Credentials;
use crate::error::{LazyFileError, Result};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Authentication mode configuration.
#[derive(Debug, Clone, Copy)]
pub enum AuthMode {
    /// Require login on application startup
    #[allow(dead_code)]
    RequireOnStartup,
    /// Only prompt for login when receiving 401 Unauthorized
    #[allow(dead_code)]
    OnDemand,
    /// Both modes: require on startup and on-demand if new auth needed
    Both,
}

/// Manages authentication credentials for rclone daemon and remotes.
#[derive(Debug)]
pub struct AuthManager {
    /// Global daemon credentials
    daemon_credentials: Option<Credentials>,
    /// Per-remote credentials
    #[allow(dead_code)]
    remote_credentials: HashMap<String, Credentials>,
    /// Authentication mode
    auth_mode: AuthMode,
}

impl AuthManager {
    /// Create a new AuthManager.
    pub fn new(auth_mode: AuthMode) -> Self {
        Self {
            daemon_credentials: None,
            remote_credentials: HashMap::new(),
            auth_mode,
        }
    }

    /// Set global daemon credentials.
    pub fn set_daemon_credentials(&mut self, credentials: Credentials) -> Result<()> {
        debug!("Setting daemon credentials for authentication");

        if let Err(e) = self.store_in_keyring(&credentials) {
            error!("Failed to store credentials in keyring: {}", e);
        } else {
            info!("Credentials stored in system keyring");
        }

        self.daemon_credentials = Some(credentials);
        Ok(())
    }

    /// Set credentials for a specific remote.
    #[allow(dead_code)]
    pub fn set_remote_credentials(&mut self, remote: &str, credentials: Credentials) -> Result<()> {
        debug!("Setting credentials for remote: {}", remote);

        if let Err(e) = self.store_in_keyring(&credentials) {
            error!("Failed to store credentials in keyring: {}", e);
        } else {
            info!(
                "Credentials stored in system keyring for remote: {}",
                remote
            );
        }

        self.remote_credentials
            .insert(remote.to_string(), credentials);
        Ok(())
    }

    /// Get daemon credentials if available.
    #[allow(dead_code)]
    pub fn get_daemon_credentials(&self) -> Option<&Credentials> {
        self.daemon_credentials.as_ref()
    }

    /// Get credentials for a specific remote.
    #[allow(dead_code)]
    pub fn get_remote_credentials(&self, remote: &str) -> Option<&Credentials> {
        self.remote_credentials.get(remote)
    }

    /// Load credentials from keyring.
    #[allow(dead_code)]
    pub fn load_from_keyring(&mut self, keyring_key: &str) -> Result<Option<Credentials>> {
        debug!(
            "Attempting to load credentials from keyring: {}",
            keyring_key
        );

        match keyring::Entry::new("LazyFile", keyring_key) {
            Ok(entry) => match entry.get_password() {
                Ok(password) => {
                    info!("Loaded credentials from keyring: {}", keyring_key);
                    if let Ok(cred) = serde_json::from_str::<Credentials>(&password) {
                        return Ok(Some(cred));
                    }
                    Ok(None)
                }
                Err(_) => {
                    debug!("No credentials found in keyring: {}", keyring_key);
                    Ok(None)
                }
            },
            Err(e) => {
                error!("Keyring error: {}", e);
                Err(LazyFileError::Keyring(e.to_string()))
            }
        }
    }

    /// Store credentials in keyring.
    fn store_in_keyring(&self, credentials: &Credentials) -> Result<()> {
        let keyring_key = credentials.keyring_key();

        match keyring::Entry::new("LazyFile", &keyring_key) {
            Ok(entry) => {
                let serialized = serde_json::to_string(credentials)?;
                entry
                    .set_password(&serialized)
                    .map_err(|e| LazyFileError::Keyring(e.to_string()))?;
                Ok(())
            }
            Err(e) => Err(LazyFileError::Keyring(e.to_string())),
        }
    }

    /// Clear credentials from keyring.
    #[allow(dead_code)]
    pub fn clear_from_keyring(&self, keyring_key: &str) -> Result<()> {
        debug!("Clearing credentials from keyring: {}", keyring_key);

        match keyring::Entry::new("LazyFile", keyring_key) {
            Ok(entry) => {
                entry
                    .delete_credential()
                    .map_err(|e| LazyFileError::Keyring(e.to_string()))?;
                Ok(())
            }
            Err(e) => Err(LazyFileError::Keyring(e.to_string())),
        }
    }

    /// Check if authentication is required based on mode.
    pub fn should_require_auth_on_startup(&self) -> bool {
        matches!(self.auth_mode, AuthMode::RequireOnStartup | AuthMode::Both)
    }

    /// Check if on-demand authentication is enabled.
    #[allow(dead_code)]
    pub fn should_auth_on_demand(&self) -> bool {
        matches!(self.auth_mode, AuthMode::OnDemand | AuthMode::Both)
    }

    /// Check if daemon has credentials configured.
    #[allow(dead_code)]
    pub fn has_daemon_credentials(&self) -> bool {
        self.daemon_credentials.is_some()
    }

    /// Get authentication mode.
    #[allow(dead_code)]
    pub fn auth_mode(&self) -> AuthMode {
        self.auth_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_startup_mode() {
        let manager = AuthManager::new(AuthMode::RequireOnStartup);
        assert!(manager.should_require_auth_on_startup());
        assert!(!manager.should_auth_on_demand());
    }

    #[test]
    fn test_auth_manager_on_demand_mode() {
        let manager = AuthManager::new(AuthMode::OnDemand);
        assert!(!manager.should_require_auth_on_startup());
        assert!(manager.should_auth_on_demand());
    }

    #[test]
    fn test_auth_manager_both_mode() {
        let manager = AuthManager::new(AuthMode::Both);
        assert!(manager.should_require_auth_on_startup());
        assert!(manager.should_auth_on_demand());
    }

    #[test]
    fn test_set_daemon_credentials() {
        let mut manager = AuthManager::new(AuthMode::RequireOnStartup);
        let creds = Credentials::basic("user".to_string(), "pass".to_string(), None);

        manager.set_daemon_credentials(creds.clone()).unwrap();
        assert!(manager.has_daemon_credentials());
        assert_eq!(manager.get_daemon_credentials().unwrap().username, "user");
    }

    #[test]
    fn test_set_remote_credentials() {
        let mut manager = AuthManager::new(AuthMode::RequireOnStartup);
        let creds = Credentials::basic(
            "user".to_string(),
            "pass".to_string(),
            Some("gdrive".to_string()),
        );

        manager
            .set_remote_credentials("gdrive", creds.clone())
            .unwrap();
        assert!(manager.get_remote_credentials("gdrive").is_some());
    }
}
