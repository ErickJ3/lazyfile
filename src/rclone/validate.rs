//! Input validation for values sent to the rclone RC API.
//!
//! The client is the single choke point: every method taking a
//! host, remote name, or path validates it here before building a
//! request, so no caller can bypass the checks.

use crate::error::{LazyFileError, Result};

fn invalid(field: &'static str, reason: &'static str) -> LazyFileError {
    LazyFileError::InvalidInput { field, reason }
}

/// Validates a daemon host before it is embedded in the base URL.
///
/// Accepts hostnames and IPv4 addresses: alphanumerics, `.` and `-`.
/// IPv6 literals are not supported; they would need bracket syntax
/// in the URL.
///
/// # Errors
/// Returns `InvalidInput` if the host is empty or contains other
/// characters.
pub(crate) fn validate_host(host: &str) -> Result<()> {
    if host.is_empty() {
        return Err(invalid("host", "must not be empty"));
    }
    if !host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
    {
        return Err(invalid(
            "host",
            "only letters, digits, '.' and '-' are allowed",
        ));
    }
    Ok(())
}

/// Validates a remote name before it is used in an fs string.
///
/// Accepts alphanumerics, `-`, `_`, `.` and spaces. Rejects `:` and
/// `/` (fs-string and path separators) and control characters.
///
/// # Errors
/// Returns `InvalidInput` if the name is empty or contains rejected
/// characters.
pub(crate) fn validate_remote_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(invalid("remote name", "must not be empty"));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
    {
        return Err(invalid(
            "remote name",
            "only letters, digits, '-', '_', '.' and spaces are allowed",
        ));
    }
    Ok(())
}

/// Validates a path before it is sent to the rclone API.
///
/// An empty path is the remote root and is valid. Rejects control
/// characters and `..` path segments (traversal out of the remote).
///
/// # Errors
/// Returns `InvalidInput` on control characters or `..` segments.
pub(crate) fn validate_path(path: &str) -> Result<()> {
    if path.chars().any(char::is_control) {
        return Err(invalid("path", "control characters are not allowed"));
    }
    if path.split('/').any(|segment| segment == "..") {
        return Err(invalid("path", "'..' path segments are not allowed"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_hostname_and_ipv4() {
        assert!(validate_host("localhost").is_ok());
        assert!(validate_host("127.0.0.1").is_ok());
        assert!(validate_host("my-host.example.com").is_ok());
    }

    #[test]
    fn rejects_empty_host() {
        assert!(validate_host("").is_err());
    }

    #[test]
    fn rejects_host_with_scheme_or_port() {
        assert!(validate_host("http://evil").is_err());
        assert!(validate_host("host:5572").is_err());
        assert!(validate_host("host/path").is_err());
    }

    #[test]
    fn accepts_remote_name_with_dots_and_spaces() {
        assert!(validate_remote_name("gdrive").is_ok());
        assert!(validate_remote_name("my remote.backup").is_ok());
        assert!(validate_remote_name("s3_bucket-2").is_ok());
    }

    #[test]
    fn rejects_empty_remote_name() {
        assert!(validate_remote_name("").is_err());
    }

    #[test]
    fn rejects_remote_name_with_separator() {
        assert!(validate_remote_name("a:b").is_err());
        assert!(validate_remote_name("a/b").is_err());
    }

    #[test]
    fn rejects_remote_name_with_control_chars() {
        assert!(validate_remote_name("bad\nname").is_err());
        assert!(validate_remote_name("bad\u{1b}name").is_err());
    }

    #[test]
    fn accepts_empty_and_nested_paths() {
        assert!(validate_path("").is_ok());
        assert!(validate_path("docs/reports/2024").is_ok());
        assert!(validate_path("file.with.dots.txt").is_ok());
        assert!(validate_path("dir with spaces/file").is_ok());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_path("../evil").is_err());
        assert!(validate_path("docs/../../etc").is_err());
        assert!(validate_path("..").is_err());
    }

    #[test]
    fn accepts_dotfiles_and_double_dot_names() {
        assert!(validate_path(".hidden").is_ok());
        assert!(validate_path("archive..old/file").is_ok());
    }

    #[test]
    fn rejects_path_with_control_chars() {
        assert!(validate_path("bad\npath").is_err());
        assert!(validate_path("bad\u{0}path").is_err());
    }
}
