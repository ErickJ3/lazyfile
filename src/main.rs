//! LazyFile - TUI file manager for cloud storage using rclone.

mod app;
mod auth;
mod cli;
mod config;
mod error;
mod launcher;
mod rclone;
mod ui;

use app::App;
use auth::AuthManager;
use clap::Parser;
use cli::Args;
use rclone::RcloneClient;

#[tokio::main]
async fn main() -> error::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::debug!("Starting LazyFile");

    let client = RcloneClient::new(&args.host, args.port);
    let auth_manager = AuthManager::new(auth::AuthMode::Both);
    let app = App::new(client, auth_manager);

    launcher::start(app).await
}
