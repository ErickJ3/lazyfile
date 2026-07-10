//! LazyFile - TUI file manager for cloud storage using rclone.

use clap::Parser;
use lazyfile::app::App;
use lazyfile::cli::Args;
use lazyfile::error;
use lazyfile::launcher;
use lazyfile::rclone::RcloneClient;

#[tokio::main]
async fn main() -> error::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::debug!("Starting LazyFile");

    let client = RcloneClient::new(&args.host, args.port)?;
    let mut app = App::new(client);
    app.load_remotes().await?;

    launcher::start(app).await
}
