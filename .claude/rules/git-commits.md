---
paths:
  - "**"
---

# Git Commits

1. Commit messages follow Conventional Commits:
   `<type>(<scope>): <description>`.
2. Types: `feat`, `fix`, `refactor`, `docs`, `test`, `ci`, `build`, `chore`,
   `perf`, `style`.
3. Scopes map to modules: `app`, `ui`, `rclone`, `cli`, `error`, `config`,
   `launcher`, `deps`, `ci`.
4. Description starts with lowercase verb, imperative mood: "add file preview
   panel", not "Added file preview panel".
5. Body (optional, separated by blank line) explains *why*, not *what*. The
   diff shows *what*.
6. Breaking changes use `!` after scope:
   `feat(rclone)!: change auth flow to support credentials`.
7. Keep commits atomic: one logical change per commit. A refactor and a feature
   are separate commits.
8. NEVER commit generated files, build artifacts, or credentials. Ensure
   `.gitignore` covers `target/`, `.env`, `*.pem`, `*.key`.
9. Run `just check` before committing.
