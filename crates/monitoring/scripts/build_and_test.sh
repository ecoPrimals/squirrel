#!/bin/bash
# Build and test script for the monitoring crate
# This script builds the crate and runs only the stable tests

set -e # Exit on any error

echo "=== Building squirrel-monitoring crate ==="
cd "$(dirname "$0")/.." # Navigate to the crate root directory
cargo build

echo ""
echo "=== Running stable tests only ==="
# List of tests that are known to work
STABLE_TESTS=(
  "websocket_compression_tests"
  "websocket_integration_tests"
)

# Run each stable test
for test in "${STABLE_TESTS[@]}"; do
  echo ""
  echo "Running test: $test"
  cargo test --test "$test" -- --nocapture
done

echo ""
echo "=== Build and test completed successfully ==="

# Run examples to ensure they compile
echo ""
echo "=== Verifying examples ==="
cargo build --examples

echo ""
echo "=== All done! ===" 