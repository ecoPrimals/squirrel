#!/bin/bash
# Script to run all fuzzers for the Squirrel Plugin System

set -e

# Create output directory
mkdir -p fuzz_output

# Build the fuzzers
echo "Building fuzzers..."
cargo build --release --bin fuzz_dynamic_library
cargo build --release --bin fuzz_plugin_command

# Run the dynamic library fuzzer
echo "Running dynamic library fuzzer..."
RUST_BACKTRACE=1 ./target/release/fuzz_dynamic_library fuzz/corpus/dynamic_library -max_len=10240 -runs=10000 -artifact_prefix=fuzz_output/dynamic_library_

# Run the plugin command fuzzer
echo "Running plugin command fuzzer..."
RUST_BACKTRACE=1 ./target/release/fuzz_plugin_command fuzz/corpus/plugin_command -max_len=10240 -runs=10000 -artifact_prefix=fuzz_output/plugin_command_

echo "Fuzzing complete!"
echo "Any crashes or hangs will be saved to fuzz_output/ directory." 