check: lint fmt-check test

test:
    cargo test --lib --verbose

# Integration tests need an rclone daemon on localhost:5572 and run
# single-threaded: they share the daemon config and a /tmp directory.
test-integration:
    cargo test --verbose -- --ignored --test-threads=1

test-all:
    cargo test --verbose -- --include-ignored --test-threads=1

lint:
    cargo clippy -- -D warnings

fmt-check:
    cargo fmt -- --check

fmt:
    cargo fmt

build:
    cargo build

build-release:
    cargo build --release

run *ARGS:
    cargo run -- {{ARGS}}

run-debug *ARGS:
    RUST_LOG=lazyfile=debug cargo run -- {{ARGS}} 2> lazyfile_debug.log

run-trace *ARGS:
    RUST_LOG=lazyfile=trace cargo run -- {{ARGS}} 2> lazyfile_trace.log

audit:
    cargo audit

clean:
    cargo clean

doc:
    cargo doc --open
