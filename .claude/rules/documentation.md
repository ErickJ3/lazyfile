---
paths:
  - "**/*.rs"
---

# Documentation

1. All `pub` items MUST have `///` doc comments. `pub(crate)` items SHOULD have
   doc comments if their purpose is not obvious from the name and signature.
2. Doc comments use third-person declarative voice: "Returns the list of
   remotes" not "Return the list of remotes" or "This function returns...".
3. Include `# Errors` section in doc comments for functions that return
   `Result`, listing the error conditions.
4. Include `# Panics` section if a function can panic (should be extremely rare
   per error-handling rules).
5. Module-level docs (`//!`) MUST exist in each module's root file explaining
   the module's purpose.
6. NEVER use doc comment decorators (separator lines, box-drawing characters,
   equals signs, or visual dividers).
7. AVOID restating the type signature in prose. "Takes a `&str` and returns a
   `bool`" adds nothing.
8. Document design rationale in `//` comments (not doc comments) when the
   implementation choice is non-obvious. Explain *why*, not *what*.
