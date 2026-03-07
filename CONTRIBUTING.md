# Contributing to LazyFile

## Getting started

1. Fork the repo and clone it
2. Create a branch (`feature/thing`, `fix/thing`, `docs/thing`)
3. Make your changes
4. Open a pull request

You'll need Rust 1.70+ and rclone with at least one test remote.

## Development setup

```bash
git clone https://github.com/<your-username>/lazyfile.git
cd lazyfile
cargo build
```

To run the app:

```bash
# Terminal 1: rclone daemon (no auth -- LazyFile doesn't support auth yet)
rclone rcd --rc-addr localhost:5572 --rc-no-auth

# Terminal 2
cargo run
```

Before committing, run all three:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## Pull requests

### Small changes

Fix a bug, update docs -- just open a PR. Make sure fmt/clippy/test pass.

### Larger changes

Open an issue first to discuss the approach. Once there's agreement, go ahead with the implementation.

### PR checklist

Before submitting:

- [ ] `cargo fmt` -- no formatting issues
- [ ] `cargo clippy -- -D warnings` -- no warnings
- [ ] `cargo test` -- tests pass
- [ ] Commit messages follow the format below

In the PR description, explain what it does and why. If it fixes an issue, mention it.

## Coding standards

### Style

- `cargo fmt` for formatting
- Follow [Rust naming conventions](https://rust-lang.org/api-guidelines/)
- Use the custom error types from `src/error.rs`, not `.unwrap()`
- Log with `tracing` macros (`debug!`, `info!`, `error!`), never `println!`

### Error handling

```rust
use crate::error::{LazyFileError, Result};

pub async fn operation() -> Result<Output> {
    if something_wrong {
        return Err(LazyFileError::Config("message".to_string()));
    }

    let response = client.send().await?;
    Ok(result)
}
```

### Logging

```rust
use tracing::{debug, info, error, trace};

fn my_function() {
    debug!("Starting operation");

    match result {
        Ok(value) => info!("Operation succeeded"),
        Err(e) => error!("Operation failed: {}", e),
    }
}
```

## Commit messages

Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Examples:

```
feat(rclone): add search functionality to file browser

Implement search within remotes to quickly find files.
Added search_files method to RcloneClient and integrated
with UI event handling.

Fixes #45
```

```
fix(rclone): handle daemon connection errors

Improve error messages when rclone daemon is unreachable.
Return specific RcloneApi error instead of generic error.
```

## Testing

```bash
cargo test                    # all tests
cargo test -- --nocapture     # with stdout
cargo test test_name          # specific test
```

For changes that touch rclone interaction, test manually too:

1. Start `rclone rcd --rc-addr localhost:5572 --rc-no-auth`
2. `cargo run` and check that remotes load, navigation works, and create/edit/delete (`a`/`e`/`d`) behave correctly
3. If you see "403 Forbidden", rclone is running with auth -- restart with `--rc-no-auth`

## Where to help

The rclone API endpoints for file operations are already defined in `src/rclone/commands.rs`:

```rust
pub const MKDIR: &str = "operations/mkdir";
pub const DELETE_FILE: &str = "operations/deletefile";
pub const COPY_FILE: &str = "operations/copyfile";
pub const MOVE_FILE: &str = "operations/movefile";
```

These aren't wired up to the UI or `RcloneClient` yet. That's probably the biggest area for contribution right now.

Beyond that: auth support for rclone RC is the biggest missing piece. Search/filter, config files, and custom keybindings would all be nice too. And bug fixes -- if you find one, please report it even if you don't have a fix.

## Questions?

Open an issue. Check existing issues first to avoid duplicates, and include your OS, Rust version, and any error output.
