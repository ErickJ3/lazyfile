---
paths:
  - "**/*.rs"
---

# Observability

## Tracing

1. Use `tracing` (not `log`) for all instrumentation. `println!`, `eprintln!`,
   and `dbg!` are forbidden — they corrupt the TUI display.
2. All log output goes to stderr (configured in `main.rs`). The TUI owns stdout.
3. Logging is disabled by default. Users enable it via `RUST_LOG` env var.
4. Use structured fields, not string interpolation:
   `tracing::info!(remote = %name, file_count = items.len(), "loaded files")` not
   `tracing::info!("loaded {} files from {}", items.len(), name)`.
5. Error-level events MUST include the error as a field:
   `tracing::error!(error = %e, "failed to list remotes")`.
6. Log level guidelines:
   - `trace` — HTTP request/response bodies, detailed parsing
   - `debug` — navigation events, state changes, modal open/close
   - `info` — remote loaded, files loaded, remote created/deleted
   - `warn` — degraded states (connection retry, unexpected response format)
   - `error` — operation failures (API errors, network failures)
