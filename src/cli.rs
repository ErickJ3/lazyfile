//! Clap config
use crate::config::{RCLONE_HOST, RCLONE_PORT};
use clap::Parser;

/// LazyFile - TUI file manager for cloud storage via rclone.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// rclone daemon host address (default: "localhost")
    #[arg(long, default_value = RCLONE_HOST)]
    pub host: String,

    /// rclone daemon port (default: 5572)
    #[arg(long, default_value_t = RCLONE_PORT)]
    pub port: u16,
}
