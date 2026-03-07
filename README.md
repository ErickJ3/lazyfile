# LazyFile

A terminal file browser for rclone remotes.

![Tests](https://img.shields.io/github/actions/workflow/status/ErickJ3/lazyfile/test.yml?branch=main&label=test)
![Release](https://img.shields.io/github/actions/workflow/status/ErickJ3/lazyfile/release.yml?branch=main&label=release)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

![LazyFile Screenshot](./assets/screenshot.png)

## What is this?

LazyFile is a two-panel TUI that talks to rclone's JSON-RPC API. Remotes on the left, their contents on the right, vim-style keys to move around. Basically lazygit for cloud storage.

Built with Rust, ratatui, and reqwest.

## Features

Browse rclone remotes, navigate directories, and manage remote configs -- all from your terminal.

Keybindings:

- `k` / `Up` -- move up
- `j` / `Down` -- move down
- `Enter` -- open remote or directory
- `Backspace` -- go back
- `Tab` -- switch panels
- `a` -- add remote
- `e` -- edit remote
- `d` -- delete remote (asks for confirmation)
- `q` -- quit

## Installation

You need Rust 1.70+, and rclone with at least one remote configured.

```bash
git clone https://github.com/ErickJ3/lazyfile.git
cd lazyfile
cargo build --release
```

Binary ends up at `target/release/lazyfile`.

## Getting started

### 1. Start the rclone daemon

In a separate terminal:

```bash
rclone rcd --rc-addr localhost:5572 --rc-no-auth
```

That starts rclone's RC server without auth, which is fine for local use.

Auth support (`--rc-user` / `--rc-pass`) isn't implemented in LazyFile yet. If you start rclone with auth enabled, LazyFile won't be able to connect.

### 2. Run LazyFile

```bash
lazyfile
```

By default it connects to `localhost:5572`. To change that:

```bash
lazyfile --host localhost --port 8080
```

If rclone is on a remote machine:

```bash
# On the remote server
rclone rcd --rc-addr 0.0.0.0:5572 --rc-no-auth

# On your machine
lazyfile --host remote-server --port 5572
```

## Usage

The left panel shows your rclone remotes (gdrive, dropbox, s3, etc.). The right panel shows files in whichever remote you've selected.

1. `j`/`k` to pick a remote
2. `Enter` to open it
3. Navigate files in the right panel
4. `Enter` to open directories, `Backspace` to go back
5. `Tab` to switch between panels

### Managing remotes

With the remote list focused:

- `a` opens a modal to create a new remote (name, type, path)
- `e` opens an edit modal for the selected remote
- `d` asks for confirmation, then deletes

### Status bar

Shows the current `remote:path` and connection status.

### Troubleshooting

**"403 Forbidden" on startup:** rclone is running with auth enabled. Restart it with `--rc-no-auth`.

If something else is wrong:

1. Check rclone is actually running: `curl http://localhost:5572/config/listremotes`
2. Check port isn't taken: `lsof -i :5572`
3. Check rclone has remotes: `rclone config show`
4. Run with trace logging: `RUST_LOG=lazyfile=trace lazyfile`

## Logging

Logging is off by default (it would mess up the TUI). Turn it on with `RUST_LOG`:

```bash
# Debug level -- good for troubleshooting
RUST_LOG=lazyfile=debug lazyfile

# Trace level -- very verbose, includes HTTP requests
RUST_LOG=lazyfile=trace lazyfile

# Just one module
RUST_LOG=lazyfile::rclone::client=trace lazyfile
```

Logs go to stderr, so you can capture them separately:

```bash
RUST_LOG=lazyfile=trace lazyfile 2> lazyfile_debug.log
```

## Development

```bash
cargo build                   # debug build
cargo build --release         # release build
cargo test                    # run tests
cargo clippy -- -D warnings   # lint
cargo doc --open              # generate docs
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

Short version: fork, branch, make changes, run `cargo clippy` and `cargo fmt`, open a PR.

Things that could use help:
- File operations (copy, move, delete) -- the rclone API endpoints are defined but not wired up
- Search/filter within remotes
- Config file support
- Auth support for rclone RC
- Bug reports and fixes

## Roadmap

What's working now: remote browsing and remote management (create/edit/delete).

Planned (roughly in order of priority):
- Auth support for rclone RC -- this is the main blocker for real-world use
- File operations (copy, move, delete)
- Search and filter
- Multi-file selection
- Directory sync
- Custom keybindings, themes, config file -- the nice-to-haves

## License

MIT. See LICENSE.
