//! Rclone API method names.

/// List all configured remotes.
pub const LIST_REMOTES: &str = "config/listremotes";
/// List files in a path.
pub const LIST_FILES: &str = "operations/list";
/// Create a directory.
pub const MKDIR: &str = "operations/mkdir";
/// Delete a file.
pub const DELETE_FILE: &str = "operations/deletefile";
/// Delete a directory and contents.
pub const PURGE: &str = "operations/purge";
/// Copy a file.
pub const COPY_FILE: &str = "operations/copyfile";
/// Move a file.
pub const MOVE_FILE: &str = "operations/movefile";
/// Create a remote configuration.
pub const CONFIG_CREATE: &str = "config/create";
/// Update a remote configuration.
pub const CONFIG_UPDATE: &str = "config/update";
/// Delete a remote configuration.
pub const CONFIG_DELETE: &str = "config/delete";
/// Sync/copy a directory.
pub const SYNC_COPY: &str = "sync/copy";
