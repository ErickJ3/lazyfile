---
paths:
  - "**/Cargo.toml"
  - "**/Cargo.lock"
---

# Dependencies

1. `Cargo.lock` MUST be committed (this is a binary project).
2. Approved core dependencies:
   - Async runtime: `tokio` (with `full` features)
   - TUI: `ratatui`, `crossterm`
   - HTTP: `reqwest` (with `json` feature)
   - Serialization: `serde`, `serde_json`
   - CLI: `clap` (derive)
   - Error handling: `thiserror`, `anyhow`
   - Logging: `tracing`, `tracing-subscriber`
   - System: `dirs`
3. Adding a new dependency MUST be justified. Consider: binary size, compile
   time, maintenance status, supply chain risk.
4. PREFER crates with >1 maintainer and recent releases.
5. Run `cargo audit` periodically to check for known vulnerabilities.
6. AVOID pulling in heavy frameworks for simple tasks. If a feature can be
   implemented in <50 lines, prefer hand-rolling over adding a dependency.
