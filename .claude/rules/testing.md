---
paths:
  - "**/*.rs"
  - "tests/**"
---

# Testing

## Unit Tests

1. Unit tests live in `#[cfg(test)] mod tests` at the bottom of the file they
   test. Test function names describe behavior:
   `fn rejects_empty_remote_name()`, not `fn test_1()`.
2. Assert on behavior, not implementation details. Test the public API of
   modules, not private helper functions.
3. PREFER `assert_eq!` and `assert_ne!` with descriptive messages over bare
   `assert!`. Use `assert!(matches!(...))` for enum variant checks.

## Integration Tests

4. Integration tests live in `tests/` at the crate root.
5. Tests requiring a running rclone daemon MUST be gated with `#[ignore]` and
   documented. Run them with `cargo test -- --ignored`.
6. Use `tests/common/mod.rs` for shared test utilities.

## TUI Testing

7. Test state transitions (handler logic) independently from rendering.
   Create an `App` with mock data and assert state after key events.
8. PREFER testing the handler with synthetic `KeyEvent` values over
   end-to-end terminal tests.

## CI

9. CI MUST run: `cargo test --lib`, `cargo clippy -- -D warnings`,
   `cargo fmt -- --check`.
10. Security audit runs `cargo audit` (allowed to fail as warning).
