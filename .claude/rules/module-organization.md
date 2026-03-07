---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---

# Module Organization

## Structure

1. LazyFile is a single-crate project with `src/main.rs` (binary entry) and
   `src/lib.rs` (library facade).
2. Module boundaries map to concerns:
   - `app/` — application state and event handling
   - `cli.rs` — CLI argument parsing
   - `config/` — configuration constants and defaults
   - `error.rs` — error types
   - `launcher.rs` — terminal lifecycle and event loop
   - `rclone/` — rclone RC API client and types
   - `ui/` — layout, styles, and widget rendering

## File Organization

3. Files SHOULD NOT exceed 600 lines. When a file grows beyond this, split by
   logical concern into submodules.
4. Split strategy: extract types into `types.rs`, implementations into verb
   files, and re-export from the parent module.
5. `lib.rs` serves as the public API facade. Internal modules use `pub(crate)`
   visibility where possible.
6. PREFER flat module hierarchies. Three levels of nesting is the maximum:
   `crate::ui::widgets::file_list`.

## Visibility

7. Default to private. Use `pub(crate)` for crate-internal sharing. Use `pub`
   only for items that need to be accessed from `main.rs` or tests.
8. Test helpers MUST live in `#[cfg(test)] mod tests` or a `tests/common/`
   module. NEVER expose test utilities in production builds.
