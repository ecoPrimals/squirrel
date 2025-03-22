# Build and test script for Squirrel workspace
Write-Host "`n[BUILD] Building all crates...`n" -ForegroundColor Cyan

# Run cargo build for all workspace crates
cargo build --workspace
if (-not $?) {
    Write-Host "`n[ERROR] Build failed!`n" -ForegroundColor Red
    exit 1
}

Write-Host "`n[SUCCESS] Build successful!`n" -ForegroundColor Green
Write-Host "[TEST] Running all tests...`n" -ForegroundColor Cyan

# Run cargo test for all workspace crates with verbose output
cargo test --workspace -- --nocapture --show-output
if (-not $?) {
    Write-Host "`n[ERROR] Some tests failed!`n" -ForegroundColor Red
    exit 1
}

Write-Host "`n[SUCCESS] All tests passed!`n" -ForegroundColor Green 