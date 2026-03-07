---
paths:
  - "**/*.rs"
---

# Async Patterns

## Runtime

1. Use `tokio` as the sole async runtime. `#[tokio::main]` goes only in
   `main.rs`.
2. The event loop in `launcher.rs` polls for terminal events with a timeout
   (`crossterm::event::poll`). Keep the poll interval short (100-250ms) to
   maintain UI responsiveness.

## HTTP Client

3. The `RcloneClient` uses `reqwest::Client` for HTTP requests. The client
   SHOULD be configured with a timeout (e.g., 15-30 seconds) to prevent
   hanging on unresponsive daemons.
4. All rclone API calls are `async`. They MUST be called from the handler,
   never from render functions.
5. Long-running operations (sync, copy large files) should provide feedback
   to the user via status updates, not block the UI.

## Send + Sync

6. All futures returned from public async functions MUST be `Send`.
7. AVOID holding non-Send types across `.await` points.

## Blocking Work

8. NEVER perform blocking computation on the async runtime. Use
   `tokio::task::spawn_blocking` for CPU-heavy work if needed.
