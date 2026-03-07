---
paths:
  - "**/*.rs"
---

# Code Style

1. Follow RFC 430: type names use `UpperCamelCase`, treating acronyms as words.
   Use `RcloneClient`, `HttpClient`, `TuiLayout` — NEVER `RCloneClient`,
   `HTTPClient`, `TUILayout`.
2. Function and variable names use `snake_case`. Module names use `snake_case`.
3. Line length MUST NOT exceed 100 characters. Break long chains with one method
   per line, aligned at the dot.
4. Use `rustfmt` defaults. NEVER add a `rustfmt.toml` override unless explicitly
   agreed upon.
5. Import ordering: std first, then external crates, then `crate::`/`super::`/
   `self::`. Separate each group with a blank line. NEVER use glob imports
   except `use super::*` in `#[cfg(test)] mod tests`.
6. PREFER `&str` over `&String`, `&[T]` over `&Vec<T>` in function parameters.
7. Constants use `SCREAMING_SNAKE_CASE`. Place module-wide constants at the top
   of the file after imports.
8. Boolean parameters MUST be replaced with enums when a function takes more
   than one. `fn render(focused: bool, selected: bool)` is forbidden — use
   typed options or a config struct.
9. PREFER exhaustive `match` over `if let` chains when matching on enums with
   3+ variants.
10. PREFER `Self` over repeating the type name inside `impl` blocks.
11. NEVER leave `dbg!()`, `println!()`, or `eprintln!()` in any code. They
    corrupt the TUI display. Use `tracing` for debug output.
12. NEVER leave commented-out code. Delete it; git history preserves everything.
