#!/usr/bin/env bash
# Script to run all monitoring crate tests with detailed reporting

# Set colors for output
COLOR_RESET="\033[0m"
COLOR_RED="\033[31m"
COLOR_GREEN="\033[32m"
COLOR_YELLOW="\033[33m"
COLOR_BLUE="\033[34m"
COLOR_MAGENTA="\033[35m"
COLOR_CYAN="\033[36m"
COLOR_WHITE="\033[37m"

# Helper functions
print_header() {
    echo -e "\n${COLOR_BLUE}=== $1 ===${COLOR_RESET}\n"
}

print_success() {
    echo -e "${COLOR_GREEN}$1${COLOR_RESET}"
}

print_error() {
    echo -e "${COLOR_RED}$1${COLOR_RESET}"
}

print_warning() {
    echo -e "${COLOR_YELLOW}$1${COLOR_RESET}"
}

print_info() {
    echo -e "${COLOR_CYAN}$1${COLOR_RESET}"
}

# Test categories and their patterns
declare -a TEST_CATEGORIES=(
    "Component Tests:test_*:Tests for individual monitoring components"
    "Integration Tests:integration_*:Tests for component interactions"
    "Reliability Tests:reliability_*:Tests for system reliability and failure handling"
    "End-to-End Tests:end_to_end_*:Tests for complete monitoring workflows"
    "WebSocket Tests:websocket_*:Tests for WebSocket communication"
)

# Summary counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Main script
print_header "Squirrel Monitoring Test Suite"
echo "Starting comprehensive test run..."

# Run each test category
for category_info in "${TEST_CATEGORIES[@]}"; do
    # Split the category info by colon
    IFS=':' read -r NAME PATTERN DESCRIPTION <<< "$category_info"
    
    print_header "$NAME"
    echo "$DESCRIPTION"
    echo ""
    
    # Run cargo test with the specific pattern
    OUTPUT=$(cargo test --package squirrel-monitoring -- --test "$PATTERN" --nocapture 2>&1)
    EXIT_CODE=$?
    
    # Check if the test succeeded
    if [ $EXIT_CODE -eq 0 ]; then
        # Count passed tests by looking for "test result: ok" lines
        if [[ "$OUTPUT" =~ "test result: ok. "([0-9]+)" passed" ]]; then
            PASSED=${BASH_REMATCH[1]}
            TOTAL_TESTS=$((TOTAL_TESTS + PASSED))
            PASSED_TESTS=$((PASSED_TESTS + PASSED))
            print_success "✅ All tests in category passed ($PASSED tests)"
        else
            print_warning "⚠️ No tests were run in this category"
        fi
    else
        # Count failed tests by looking for "test result: FAILED" lines
        if [[ "$OUTPUT" =~ "test result: FAILED. "([0-9]+)" passed; "([0-9]+)" failed" ]]; then
            PASSED=${BASH_REMATCH[1]}
            FAILED=${BASH_REMATCH[2]}
            TOTAL_TESTS=$((TOTAL_TESTS + PASSED + FAILED))
            PASSED_TESTS=$((PASSED_TESTS + PASSED))
            FAILED_TESTS=$((FAILED_TESTS + FAILED))
            print_error "❌ Some tests in category failed: $PASSED passed, $FAILED failed"
        else
            print_error "❌ Tests failed to run in this category"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
    
    echo ""
done

# Print summary
print_header "Test Summary"
if [ $FAILED_TESTS -eq 0 ]; then
    print_success "✅ All tests passed: $PASSED_TESTS / $TOTAL_TESTS"
else
    print_error "❌ Some tests failed: $PASSED_TESTS passed, $FAILED_TESTS failed out of $TOTAL_TESTS total"
fi

# Recommend next steps
print_header "Next Steps"
if [ $FAILED_TESTS -eq 0 ]; then
    print_info "➡️ Run longer stability tests with: cargo test --package squirrel-monitoring -- --test reliability_test --nocapture -- --ignored"
    print_info "➡️ Measure performance with benchmarks: cargo bench --package squirrel-monitoring"
    print_info "➡️ Generate test coverage report: cargo tarpaulin --out Html --output-dir ./target/tarpaulin"
else
    print_warning "➡️ Fix failing tests by examining the errors above"
    print_warning "➡️ Re-run specific failed test categories for more detailed output"
fi

# Exit with the number of failed tests as the exit code
exit $FAILED_TESTS 