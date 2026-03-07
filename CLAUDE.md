# LazyFile — Coding Rules

> Complete rules in `.claude/rules/`. This file summarizes the most critical ones.

## CRITICAL: Error Handling

- NEVER: `.unwrap()`, `.expect()`, `panic!()`, `todo!()` in library/app code
- MUST: `thiserror::Error` enum (`LazyFileError`) for all error variants
- MUST: Propagate errors with `?`, network errors include endpoint context
- `anyhow` is acceptable only in tests and one-off scripts

## CRITICAL: Code Style

- RFC 430: `CamelCase` types, acronyms as words (`RcloneClient`, `HttpClient`)
- Lines <= 100 chars. Comments explain *why*, never *what*
- NEVER: glob imports (except `use super::*` in tests), decorative separators, commented-out code
- PREFER: `let-else`, `?`, exhaustive match, iterators, newtype pattern

## CRITICAL: Async

- `tokio` is the sole runtime. `#[tokio::main]` only in `main.rs`
- All public async functions return `Send` futures
- NEVER block the async runtime — use `spawn_blocking` for CPU-heavy work
- HTTP client calls MUST have timeouts configured

## CRITICAL: TUI Patterns

- UI rendering MUST be pure functions: `fn render(frame, area, state)` — no side effects
- State mutations happen ONLY in `Handler::handle_key`, never in render code
- Modal state uses `Option<ModalType>` — `None` means closed, `Some` means open
- NEVER use `println!`, `eprintln!`, `dbg!` — they corrupt the TUI. Use `tracing`

## Project Structure

| Module | Purpose |
|--------|---------|
| `app/state` | Application state (`App` struct, `Panel` enum) |
| `app/handler` | Keyboard event handling |
| `cli` | CLI argument parsing (clap) |
| `config` | Configuration constants |
| `error` | Error types (`LazyFileError`) |
| `launcher` | Terminal setup/teardown + event loop |
| `rclone/client` | HTTP client for rclone RC API |
| `rclone/types` | Request/response types for rclone API |
| `ui/layout` | Layout splitting |
| `ui/styles` | Shared styles and colors |
| `ui/widgets/` | Individual widget implementations |

## Observability

- Use `tracing` (not `log`) for all instrumentation
- Logs go to stderr (never stdout — TUI uses stdout)
- Structured fields: `tracing::debug!(remote = %name, "loading files")`
- Logging is off by default, enabled via `RUST_LOG=lazyfile=debug`

## Git Commits

- Format: `<type>(<scope>): <description>` (Conventional Commits)
- Types: `feat`, `fix`, `refactor`, `docs`, `test`, `ci`, `build`, `chore`, `perf`
- Scopes: `app`, `ui`, `rclone`, `cli`, `error`, `deps`, `ci`
- Run `just check` before every commit

## Build Commands (just)

- Full check: `just check` (lint + fmt + test)
- Test: `just test`
- Build: `just build-release`
- Lint: `just lint`
- Format: `just fmt`
- Audit: `just audit`
- Run with debug log: `just run-debug`

## Dependencies

- Approved: tokio, ratatui, crossterm, reqwest, serde, serde_json, clap, thiserror, anyhow, tracing, tracing-subscriber, dirs
- New deps MUST be justified (binary size, compile time, maintenance)
- `Cargo.lock` MUST be committed (this is a binary project)
