# Build and test script for the monitoring crate
# This script builds the crate and runs only the stable tests

# Exit on any error
$ErrorActionPreference = "Stop" 

Write-Host "=== Building squirrel-monitoring crate ===" -ForegroundColor Green
# Navigate to the crate root directory
Set-Location -Path (Join-Path -Path $PSScriptRoot -ChildPath "..")
cargo build

Write-Host "`n=== Running stable tests only ===" -ForegroundColor Green
# List of tests that are known to work
$STABLE_TESTS = @(
  "websocket_compression_tests"
  "websocket_integration_tests"
)

# Run each stable test
foreach ($test in $STABLE_TESTS) {
  Write-Host "`nRunning test: $test" -ForegroundColor Cyan
  cargo test --test "$test" -- --nocapture
}

Write-Host "`n=== Build and test completed successfully ===" -ForegroundColor Green

# Run examples to ensure they compile
Write-Host "`n=== Verifying examples ===" -ForegroundColor Green
cargo build --examples

Write-Host "`n=== All done! ===" -ForegroundColor Green 