#!/usr/bin/env pwsh
# Script to run all monitoring crate tests with detailed reporting

# Set colors for output
$colorReset = "`e[0m"
$colorRed = "`e[31m"
$colorGreen = "`e[32m"
$colorYellow = "`e[33m"
$colorBlue = "`e[34m"
$colorMagenta = "`e[35m"
$colorCyan = "`e[36m"
$colorWhite = "`e[37m"

# Helper functions
function Print-Header($text) {
    Write-Host "`n$colorBlue=== $text ===$colorReset`n"
}

function Print-Success($text) {
    Write-Host "$colorGreen$text$colorReset"
}

function Print-Error($text) {
    Write-Host "$colorRed$text$colorReset"
}

function Print-Warning($text) {
    Write-Host "$colorYellow$text$colorReset"
}

function Print-Info($text) {
    Write-Host "$colorCyan$text$colorReset"
}

# Test categories
$testCategories = @(
    @{
        Name = "Component Tests"
        Pattern = "test_*"
        Description = "Tests for individual monitoring components"
    },
    @{
        Name = "Integration Tests"
        Pattern = "integration_*"
        Description = "Tests for component interactions"
    },
    @{
        Name = "Reliability Tests"
        Pattern = "reliability_*"
        Description = "Tests for system reliability and failure handling"
    },
    @{
        Name = "End-to-End Tests"
        Pattern = "end_to_end_*"
        Description = "Tests for complete monitoring workflows"
    },
    @{
        Name = "WebSocket Tests"
        Pattern = "websocket_*"
        Description = "Tests for WebSocket communication"
    }
)

# Summary counters
$totalTests = 0
$passedTests = 0
$failedTests = 0

# Main script
Print-Header "Squirrel Monitoring Test Suite"
Write-Host "Starting comprehensive test run..."

# Run each test category
foreach ($category in $testCategories) {
    Print-Header $category.Name
    Write-Host $category.Description
    Write-Host ""
    
    try {
        # Run cargo test with the specific pattern
        $result = cargo test --package squirrel-monitoring -- --test $category.Pattern --nocapture 2>&1
        
        # Check if the test succeeded
        if ($LASTEXITCODE -eq 0) {
            # Count passed tests by looking for "test result: ok" lines
            $testResults = $result | Select-String -Pattern "test result: ok. (\d+) passed" -AllMatches
            if ($testResults.Matches.Count -gt 0) {
                $passed = [int]$testResults.Matches[0].Groups[1].Value
                $totalTests += $passed
                $passedTests += $passed
                Print-Success "✅ All tests in category passed ($passed tests)"
            } else {
                Print-Warning "⚠️ No tests were run in this category"
            }
        } else {
            # Count failed tests by looking for "test result: FAILED" lines
            $testResults = $result | Select-String -Pattern "test result: FAILED. (\d+) passed; (\d+) failed" -AllMatches
            if ($testResults.Matches.Count -gt 0) {
                $passed = [int]$testResults.Matches[0].Groups[1].Value
                $failed = [int]$testResults.Matches[0].Groups[2].Value
                $totalTests += ($passed + $failed)
                $passedTests += $passed
                $failedTests += $failed
                Print-Error "❌ Some tests in category failed: $passed passed, $failed failed"
            } else {
                Print-Error "❌ Tests failed to run in this category"
                $failedTests += 1
            }
        }
    } catch {
        Print-Error "❌ Error running tests: $_"
        $failedTests += 1
    }
    
    Write-Host ""
}

# Print summary
Print-Header "Test Summary"
if ($failedTests -eq 0) {
    Print-Success "✅ All tests passed: $passedTests / $totalTests"
} else {
    Print-Error "❌ Some tests failed: $passedTests passed, $failedTests failed out of $totalTests total"
}

# Recommend next steps
Print-Header "Next Steps"
if ($failedTests -eq 0) {
    Print-Info "➡️ Run longer stability tests with: cargo test --package squirrel-monitoring -- --test reliability_test --nocapture -- --ignored"
    Print-Info "➡️ Measure performance with benchmarks: cargo bench --package squirrel-monitoring"
    Print-Info "➡️ Generate test coverage report: cargo tarpaulin --out Html --output-dir ./target/tarpaulin"
} else {
    Print-Warning "➡️ Fix failing tests by examining the errors above"
    Print-Warning "➡️ Re-run specific failed test categories for more detailed output"
}

# Exit with the number of failed tests as the exit code
exit $failedTests 