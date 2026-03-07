---
paths:
  - "**/*.rs"
---

# Security

## Network Safety

1. All HTTP requests to the rclone daemon MUST have a timeout configured on the
   `reqwest::Client`. NEVER use an unbounded request that can hang forever.
2. The rclone daemon URL is constructed from user-provided host/port. Validate
   that host is a valid hostname or IP. NEVER interpolate user input into URLs
   without validation.
3. Future authentication support (HTTP Basic Auth, Bearer tokens) MUST NOT log
   or display credentials in error messages or tracing spans.

## Input Validation

4. Remote names, file paths, and user input from modals MUST be validated before
   sending to the rclone API. Reject empty strings, path traversal attempts
   (`../`), and control characters.
5. Responses from the rclone daemon MUST be validated. NEVER trust that the
   JSON structure matches expectations — handle parse failures gracefully.

## Credentials

6. When auth support is added, credentials MUST NOT appear in logs, error
   messages, or tracing spans. Implement `Debug` manually for types containing
   secrets, redacting sensitive fields.
7. NEVER hardcode credentials. Read from environment variables, config files,
   or credential helpers.
