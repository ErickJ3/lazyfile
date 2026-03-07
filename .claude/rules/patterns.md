---
paths:
  - "**/*.rs"
---

# Patterns

## Prefer

1. PREFER the newtype pattern for domain concepts where a typed wrapper adds
   clarity (e.g., `struct RemoteName(String)` instead of passing raw `String`).
2. PREFER the builder pattern for types with 3+ optional configuration fields.
3. PREFER iterators and `collect()` over manual loops with `push()`.
4. PREFER `let-else` for early returns on pattern match failure.
5. PREFER `?` over `match expr { Ok(v) => v, Err(e) => return Err(e) }`.
6. PREFER `impl Into<T>` / `impl AsRef<T>` for ergonomic public APIs that
   accept multiple input types.

## Avoid

7. AVOID `clone()` to satisfy the borrow checker. Restructure code to avoid
   the need. If `clone()` is necessary, add a comment explaining why.
8. AVOID stringly-typed APIs. Use enums for known sets of values (e.g.,
   `Panel::Remotes` not `"remotes"`).
9. AVOID deeply nested generics. Introduce type aliases or wrapper types.

## TUI-Specific Patterns

10. UI rendering functions MUST be pure: `fn render(frame, area, data)`. They
    read state but NEVER mutate it. All state mutations happen in the handler.
11. Modal state uses `Option<ModalType>`. `None` = closed, `Some(modal)` = open.
    NEVER use a boolean flag alongside a modal struct.
12. Widget rendering MUST NOT call async functions or perform I/O. Data loading
    happens in the handler, rendering only displays pre-loaded data.
13. Key handling follows a priority chain: active modal first, then panel-specific
    keys, then global keys (quit, help, tab).
