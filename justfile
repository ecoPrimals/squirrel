# SPDX-License-Identifier: AGPL-3.0-only
# Squirrel AI Primal — build automation

set dotenv-load := false

# Default recipe
default: check

# Full quality gate (CI equivalent)
ci: fmt-check clippy test doc

# Check compilation (fast feedback)
check:
    cargo check --workspace --all-features

# Format check (no modification)
fmt-check:
    cargo fmt --all -- --check

# Auto-format
fmt:
    cargo fmt --all

# Clippy with full pedantic + nursery policy
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --workspace

# Build documentation
doc:
    cargo doc --workspace --no-deps

# Coverage report (requires cargo-llvm-cov)
coverage:
    cargo llvm-cov --workspace --ignore-filename-regex 'target|archive'

# Coverage with HTML report
coverage-html:
    cargo llvm-cov --workspace --ignore-filename-regex 'target|archive' --html
    @echo "Report: target/llvm-cov/html/index.html"

# Build release binary (UniBin)
build-release:
    cargo build --release -p squirrel

# Build ecoBin (static musl x86_64)
build-ecobin:
    cargo build --release --target x86_64-unknown-linux-musl -p squirrel

# Build ecoBin for aarch64 (cross-compile)
build-ecobin-arm:
    cargo build --release --target aarch64-unknown-linux-musl -p squirrel

# Build all ecoBin targets
build-ecobin-all: build-ecobin build-ecobin-arm

# Run the server
run-server *ARGS:
    cargo run -p squirrel -- server {{ARGS}}

# Run doctor diagnostics
doctor:
    cargo run -p squirrel -- doctor --comprehensive

# Dependency audit
audit:
    cargo deny check

# Check for future-incompatible code
future-compat:
    cargo report future-incompatibilities

# Clean build artifacts
clean:
    cargo clean

# Count lines per file (check 1000-line limit)
line-check:
    @find . -name "*.rs" -not -path "*/target/*" -not -path "*/archive/*" \
        -exec wc -l {} + | sort -rn | head -20
