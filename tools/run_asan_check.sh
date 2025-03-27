#!/usr/bin/env bash
# Script to run Address Sanitizer checks on binaries without fuzzing

set -e

# Default values
BINARY_PATH=""
TEST_ARGS=""
ITERATIONS=1
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --binary|-b)
      BINARY_PATH="$2"
      shift 2
      ;;
    --args|-a)
      TEST_ARGS="$2"
      shift 2
      ;;
    --iterations|-i)
      ITERATIONS="$2"
      shift 2
      ;;
    --verbose|-v)
      VERBOSE=true
      shift
      ;;
    --help|-h)
      echo "ASAN Binary Checker for Squirrel Plugin System"
      echo "==============================================="
      echo ""
      echo "Usage:"
      echo "  ./tools/run_asan_check.sh --binary <path_to_binary> [--args '<args>'] [--iterations <num>] [--verbose]"
      echo ""
      echo "Parameters:"
      echo "  --binary, -b     : Path to the binary executable to test"
      echo "  --args, -a       : Arguments to pass to the binary (optional)"
      echo "  --iterations, -i : Number of times to run the binary (default: 1)"
      echo "  --verbose, -v    : Show detailed output"
      echo "  --help, -h       : Show this help message"
      echo ""
      echo "Example:"
      echo "  ./tools/run_asan_check.sh --binary ./target/debug/plugin_host --args '--load-plugin ./plugins/test.so' --iterations 5"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Run './tools/run_asan_check.sh --help' for usage information."
      exit 1
      ;;
  esac
done

# Show usage if no binary path provided
if [[ -z "$BINARY_PATH" ]]; then
  echo "Error: No binary path specified."
  echo "Run './tools/run_asan_check.sh --help' for usage information."
  exit 1
fi

# Check if the binary exists
if [[ ! -f "$BINARY_PATH" ]]; then
  echo "Error: Binary not found at path: $BINARY_PATH"
  exit 1
fi

# Check if the binary is executable
if [[ ! -x "$BINARY_PATH" ]]; then
  echo "Error: Binary is not executable: $BINARY_PATH"
  echo "Try running: chmod +x $BINARY_PATH"
  exit 1
fi

# Set up ASAN environment
# Check for LLVM installation
if [[ "$(uname)" == "Darwin" ]]; then
  # macOS
  if command -v brew &> /dev/null; then
    LLVM_PATH="$(brew --prefix llvm)/bin"
    if [[ -f "$LLVM_PATH/llvm-symbolizer" ]]; then
      export ASAN_SYMBOLIZER_PATH="$LLVM_PATH/llvm-symbolizer"
      export PATH="$LLVM_PATH:$PATH"
    else
      echo "Warning: LLVM symbolizer not found via Homebrew."
      echo "Address sanitizer reports may not show symbolized stack traces."
      echo "See docs/devtools/address_sanitizer_guide.md for setup instructions."
    fi
  else
    echo "Warning: Homebrew not found, can't locate LLVM symbolizer automatically."
    echo "See docs/devtools/address_sanitizer_guide.md for setup instructions."
  fi
else
  # Linux
  if command -v llvm-symbolizer &> /dev/null; then
    export ASAN_SYMBOLIZER_PATH="$(which llvm-symbolizer)"
  elif [[ -f "/usr/bin/llvm-symbolizer" ]]; then
    export ASAN_SYMBOLIZER_PATH="/usr/bin/llvm-symbolizer"
  elif [[ -f "/usr/lib/llvm-*/bin/llvm-symbolizer" ]]; then
    # Try to find the most recent LLVM installation
    SYMBOLIZER_PATH=$(ls -1 /usr/lib/llvm-*/bin/llvm-symbolizer | sort -V | tail -1)
    export ASAN_SYMBOLIZER_PATH="$SYMBOLIZER_PATH"
  else
    echo "Warning: LLVM symbolizer not found."
    echo "Address sanitizer reports may not show symbolized stack traces."
    echo "See docs/devtools/address_sanitizer_guide.md for setup instructions."
  fi
fi

# Set ASAN options
export ASAN_OPTIONS="symbolize=1:detect_leaks=0:abort_on_error=1:print_stacktrace=1"

# Check if running with Rust nightly
RUST_VERSION=$(rustc --version)
if [[ ! "$RUST_VERSION" =~ "nightly" ]]; then
  echo "Warning: You are not using Rust nightly, which is required for ASAN."
  echo "Run 'rustup default nightly' to switch to nightly."
fi

# Function to run the binary with ASAN
run_with_asan() {
  local iteration=$1
  
  echo -e "\033[36mRunning iteration $iteration with ASAN...\033[0m"
  
  if [[ "$VERBOSE" == "true" ]]; then
    echo -e "\033[90mCommand: $BINARY_PATH $TEST_ARGS\033[0m"
    echo -e "\033[90mASAN_OPTIONS: $ASAN_OPTIONS\033[0m"
  fi
  
  # Run the binary and capture stdout, stderr, and exit code
  set +e
  if [[ "$VERBOSE" == "true" ]]; then
    output=$("$BINARY_PATH" $TEST_ARGS 2>&1)
    exit_code=$?
  else
    output=$("$BINARY_PATH" $TEST_ARGS 2>&1)
    exit_code=$?
  fi
  set -e
  
  if [[ $exit_code -ne 0 ]]; then
    echo -e "\033[31mASAN detected issues (Exit code: $exit_code):\033[0m"
    
    # Check if the output contains ASAN reports
    if echo "$output" | grep -q "AddressSanitizer"; then
      echo -e "\033[31m$output\033[0m"
    else
      echo -e "\033[33mProcess failed but no ASAN report found in output.\033[0m"
      if [[ "$VERBOSE" == "true" ]]; then
        echo -e "\033[90mFull output:\033[0m"
        echo "$output"
      fi
    fi
    return 1
  else
    if [[ "$VERBOSE" == "true" ]]; then
      echo -e "\033[90mFull output:\033[0m"
      echo "$output"
    fi
    return 0
  fi
}

# Main execution
echo -e "\033[32mStarting ASAN checks for: $BINARY_PATH\033[0m"
echo -e "\033[32mRunning $ITERATIONS iteration(s)...\033[0m"

success_count=0
failure_count=0

for (( i=1; i<=$ITERATIONS; i++ )); do
  if run_with_asan $i; then
    success_count=$((success_count + 1))
    echo -e "\033[32m✓ Iteration $i completed successfully (no ASAN issues detected)\033[0m"
  else
    failure_count=$((failure_count + 1))
    echo -e "\033[31m✗ Iteration $i failed (ASAN issues detected)\033[0m"
  fi
  
  # Add a separator between iterations
  if [[ $i -lt $ITERATIONS ]]; then
    echo -e "\033[90m-----------------------------------------\033[0m"
  fi
done

# Summary
echo ""
echo -e "\033[36mASAN Check Summary:\033[0m"
echo -e "\033[36m  Total Iterations: $ITERATIONS\033[0m"

if [[ $success_count -eq $ITERATIONS ]]; then
  echo -e "\033[32m  Successful: $success_count\033[0m"
else
  echo -e "\033[36m  Successful: $success_count\033[0m"
fi

if [[ $failure_count -gt 0 ]]; then
  echo -e "\033[31m  Failed: $failure_count\033[0m"
else
  echo -e "\033[36m  Failed: $failure_count\033[0m"
fi

# Return appropriate exit code
if [[ $failure_count -gt 0 ]]; then
  exit 1
else
  exit 0
fi 