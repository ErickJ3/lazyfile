---
paths:
  - "**/*.rs"
---

# Error Handling

**Keyword definitions (RFC 2119):**
- **MUST** / **MUST NOT** — absolute requirement or prohibition.
- **NEVER** — synonym for MUST NOT, used for emphasis.
- **PREFER** — the default choice; deviate only with a documented reason.
- **AVOID** — acceptable in rare cases, but requires justification.

1. NEVER use `.unwrap()` or `.expect()` in application or library code. The ONLY
   exceptions are: (a) test code, (b) static values proven at compile time
   (e.g., `Regex::new` on a literal), (c) documented invariants with a
   `// INVARIANT:` comment explaining why the value is always `Some`/`Ok`.
2. NEVER use `panic!`, `unreachable!`, or `todo!` in production code paths.
3. All errors MUST use the `LazyFileError` enum defined in `src/error.rs` with
   `thiserror::Error`. Add new variants as needed for new error categories.
4. Error variants MUST carry structured context, not just string messages.
   Prefer `#[error("failed to list files on {remote}")]` with named fields
   over `#[error("{0}")]` with a bare `String`.
5. Use `#[from]` for direct 1:1 error conversions. Use explicit `map_err` with
   context when the conversion needs additional information.
6. Network errors MUST always include the endpoint or remote that failed. NEVER
   propagate a bare `reqwest::Error` without context about what operation failed.
7. Error types MUST implement `Send + Sync + 'static` to work across async
   boundaries.
8. AVOID `Box<dyn Error>` in the codebase. Use the concrete `LazyFileError` type.
   `anyhow` is acceptable only in test helpers.
