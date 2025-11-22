# LazyFile

A simple terminal UI for browsing and interacting with cloud storage via rclone.

![Tests](https://img.shields.io/github/actions/workflow/status/ErickJ3/lazyfile/test.yml?branch=main&label=test)
![Release](https://img.shields.io/github/actions/workflow/status/ErickJ3/lazyfile/release.yml?branch=main&label=release)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

![LazyFile Screenshot](./scrshot.png)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Starting the Application](#starting-the-application)
- [Usage](#usage)
- [Logging and Debugging](#logging-and-debugging)
- [Development](#development)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [Getting Help](#getting-help)
- [License](#license)

## Overview

LazyFile provides a lightweight terminal user interface for interacting with cloud storage systems via rclone. Instead of typing complex rclone commands or navigating nested command-line arguments, LazyFile presents an intuitive two-panel file browser directly in your terminal.

The project is built in Rust using ratatui for the terminal interface and rclone's JSON-RPC API for file operations. It emphasizes simplicity, reliability, and developer experience.

## Features

**Core Features:**

- Browse configured rclone remotes from a simple list
- Navigate remote filesystems with keyboard navigation
- View file names and directory structure
- Switch focus between remote and file list panels
- Intuitive keybindings inspired by vim and lazygit

**Keybindings:**

- `k` or `↑` - Navigate up
- `j` or `↓` - Navigate down
- `Enter` - Open remote or directory
- `Backspace` - Go back to parent directory or remotes list
- `Tab` - Switch focus between panels
- `a` - Add a new remote
- `e` - Edit the selected remote
- `d` - Delete the selected remote (with confirmation)
- `q` - Quit application

## Installation

### Prerequisites

- Rust 1.70 or later
- rclone configured with at least one remote
- rclone daemon running on localhost:5572

### Build from Source

Clone the repository and build with cargo:

```bash
git clone https://github.com/ErickJ3/lazyfile.git
cd lazyfile
cargo build --release
```

The binary will be available at `target/release/lazyfile`.

## Starting the Application

### Step 1: Start the rclone daemon

In a separate terminal, start the rclone RC (Remote Control) server. LazyFile communicates with rclone via its JSON-RPC API.

**Option A: Without authentication (Recommended for local use)**

```bash
rclone rcd --rc-addr localhost:5572 --rc-no-auth
```

This is the simplest option for local development and personal use. The server runs without authentication requirements.

**Option B: With authentication (For remote/secure access)**

```bash
rclone rcd --rc-addr localhost:5572 --rc-user your_username --rc-pass your_password
```

Note: Authentication support in LazyFile is not yet implemented. If you start rclone with authentication, LazyFile will not be able to connect. This is a planned feature.

### Step 2: Run LazyFile

In another terminal, run the application:

```bash
lazyfile
```

### Configuration

If you need to use a different host or port, edit the configuration in `src/config/mod.rs`:

```rust
pub const RCLONE_HOST: &str = "localhost";
pub const RCLONE_PORT: u16 = 5572;
```

Then rebuild the application with `cargo build --release`.

## Usage

### Basic Navigation

The interface consists of two panels:

**Left Panel:** List of configured rclone remotes (gdrive, dropbox, s3, etc.)
**Right Panel:** Files and directories in the selected remote/path

### Workflow

1. Start the application with `lazyfile`
2. Use `j`/`k` to navigate the remote list
3. Press `Enter` to select a remote and view its contents
4. Navigate files with `j`/`k` in the right panel
5. Press `Enter` to open directories
6. Press `Backspace` to go back to the parent directory
7. Press `Tab` to switch focus between panels
8. Press `q` to quit

### Remote Management

With the remote list focused (press `Tab` to switch), you can manage remotes:

**Add a new remote:**

- Press `a` to open the create modal
- Enter the remote name, type (e.g., `local`, `drive`, `s3`), and path/configuration
- Press `Enter` to create or `Esc` to cancel

**Edit an existing remote:**

- Select a remote and press `e` to open the edit modal
- Modify the configuration fields
- Press `Enter` to save or `Esc` to cancel

**Delete a remote:**

- Select a remote and press `d` to open a confirmation dialog
- Press `y` to confirm deletion or `n` to cancel

### Status Bar

The status bar at the bottom displays:

- Current remote and path (format: `remote:path` or `remote:` for root)
- Connection status to rclone daemon

### Troubleshooting

**Application won't start or shows "403 Forbidden":**

This typically means rclone RC requires authentication. LazyFile does not yet support authenticated RC servers.

1. Ensure rclone daemon is running with `--rc-no-auth`:

   ```bash
   rclone rcd --rc-addr localhost:5572 --rc-no-auth
   ```

2. Check that port 5572 is not in use: `lsof -i :5572` or `netstat -an | grep 5572`

3. Verify rclone has configured remotes: `rclone config file` and `rclone config show`

4. Test rclone daemon manually:

   ```bash
   curl http://localhost:5572/config/listremotes
   ```

   Should return JSON with your remotes, not a 403 error.

5. Run LazyFile with trace logging to see connection details:

   ```bash
   RUST_LOG=lazyfile=trace lazyfile
   ```

**Errors loading remotes or files:**

1. Confirm rclone daemon is running and accessible
2. Check that authentication is disabled (`--rc-no-auth` flag used)
3. Verify the remote is properly configured: `rclone config show`
4. Check rclone logs for API errors
5. Run with debug logging to identify the issue: `RUST_LOG=lazyfile=debug lazyfile`

## Logging and Debugging

LazyFile uses structured logging with the `tracing` crate. **Logging is disabled by default** to avoid interfering with the TUI interface, keeping it clean like other TUI tools (`lazygit`, `lazydocker`).

### Enabling Logging

Set the `RUST_LOG` environment variable to enable logging:

**Debug level (recommended for troubleshooting):**

```bash
RUST_LOG=lazyfile=debug lazyfile
```

**Trace level (very verbose, includes network requests and detailed operations):**

```bash
RUST_LOG=lazyfile=trace lazyfile
```

**Specific component logging:**

```bash
# Only log rclone client operations
RUST_LOG=lazyfile::rclone::client=trace lazyfile

# Only log event handling
RUST_LOG=lazyfile::app::handler=debug lazyfile
```

### Capturing Logs to a File

Since logs go to stderr, you can redirect them separately:

```bash
lazyfile 2> lazyfile_debug.log
```

Or combine with a logging level:

```bash
RUST_LOG=lazyfile=trace lazyfile 2> lazyfile_debug.log
```

## Development

### Building

```bash
cargo build
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Linting and Code Quality

LazyFile uses clippy for linting. Run clippy checks:

```bash
cargo clippy -- -D warnings
```

Generate documentation:

```bash
cargo doc --open
```

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to report issues, submit pull requests, and follow the project's code standards.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes and test them
4. Run `cargo clippy` and `cargo fmt` to ensure code quality
5. Commit with a clear message
6. Push to your fork and submit a pull request

### Areas for Contribution

- File operations (copy, move, delete)
- Search and filter functionality
- Configuration file support
- Additional keybinding customization
- Performance optimizations
- Documentation improvements
- Bug reports and fixes

## Roadmap

Current implementation provides remote browsing and remote management. Future versions will include:

- **Authentication support for rclone RC** (HTTP Basic Auth / Bearer tokens)
- File operations (copy, move, delete)
- Search and filter functionality within remotes
- Multiple file selection
- Directory synchronization
- Custom keybindings via config file
- Customizable themes
- Configuration file for host/port settings
- Performance optimizations for large directories

## Getting Help

- Check the troubleshooting section above
- Review [CONTRIBUTING.md](CONTRIBUTING.md) for bug reporting guidelines
- Run with logging enabled to diagnose issues: `RUST_LOG=lazyfile=trace lazyfile`
- Check existing GitHub issues for similar problems

## License

LazyFile is released under the MIT License. See LICENSE file for details.
