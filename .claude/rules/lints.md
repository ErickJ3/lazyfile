---
paths:
  - "**/*.rs"
  - "clippy.toml"
---

# Lints

1. Clippy configuration lives in `clippy.toml` at the project root. Key
   thresholds: `too-many-arguments-threshold = 7`,
   `cognitive-complexity-threshold = 30`, `too-many-lines-threshold = 200`.
2. CI runs `cargo clippy -- -D warnings`. No clippy warnings in main.
3. PREFER `#[expect(lint)]` over `#[allow(lint)]` (Rust 1.81+). `expect` will
   warn when the lint suppression becomes unnecessary.
4. Every `#[expect(...)]` or `#[allow(...)]` MUST have a reason string:
   `#[expect(clippy::too_many_arguments, reason = "render widget params")]`.
5. NEVER suppress `clippy::correctness` lints. NEVER suppress `dead_code` —
   delete unused code instead.
6. `allow-expect-in-tests`, `allow-unwrap-in-tests`, and
   `allow-panic-in-tests` are enabled in `clippy.toml`.
