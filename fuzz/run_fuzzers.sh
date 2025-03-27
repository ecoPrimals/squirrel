#!/bin/bash
# Script to run all fuzzers for the Squirrel Plugin System

set -e

# Default values
NO_ASAN=0
RUNS=10000
MAX_LEN=10240
OUTPUT_DIR="fuzz_output"

# Process command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --no-asan)
      NO_ASAN=1
      shift
      ;;
    --runs)
      RUNS="$2"
      shift 2
      ;;
    --max-len)
      MAX_LEN="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--no-asan] [--runs N] [--max-len N] [--output-dir DIR]"
      exit 1
      ;;
  esac
done

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Set ASAN flag
ASAN_FLAG=""
if [ $NO_ASAN -eq 1 ]; then
  echo "Running without Address Sanitizer..."
  ASAN_FLAG="--sanitizer=none"
else
  echo "Running with Address Sanitizer..."
  echo "Note: If this fails, you might need to set up ASAN or run with --no-asan"
  echo "See docs/devtools/address_sanitizer_guide.md for setup instructions"
fi

# Set up environment for better debugging
export RUST_BACKTRACE=1

# Run the dynamic library fuzzer
echo "Running dynamic library fuzzer..."
cargo fuzz run dynamic_library $ASAN_FLAG -- -max_len=${MAX_LEN} -runs=${RUNS} -artifact_prefix="${OUTPUT_DIR}/dynamic_library_"

# Run the plugin command fuzzer
echo "Running plugin command fuzzer..."
cargo fuzz run plugin_command $ASAN_FLAG -- -max_len=${MAX_LEN} -runs=${RUNS} -artifact_prefix="${OUTPUT_DIR}/plugin_command_"

echo "Fuzzing complete!"
echo "Any crashes or hangs will be saved to ${OUTPUT_DIR}/ directory."
echo "Run with --no-asan to disable Address Sanitizer if you encounter ASAN-related errors." 