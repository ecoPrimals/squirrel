# PowerShell script to run all fuzzers for the Squirrel Plugin System
param (
    [switch]$NoAsan = $false,
    [int]$Runs = 10000,
    [int]$MaxLen = 10240,
    [string]$OutputDir = "fuzz_output"
)

# Create output directory
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Set environment variables for better debugging
$env:RUST_BACKTRACE = "1"

# Determine if we should use ASAN
$AsanFlag = ""
if ($NoAsan) {
    Write-Host "Running without Address Sanitizer..."
    $AsanFlag = "--sanitizer=none"
} else {
    Write-Host "Running with Address Sanitizer..."
    Write-Host "Note: If this fails, you might need to set up ASAN or run with -NoAsan"
    Write-Host "See docs/devtools/address_sanitizer_guide.md for setup instructions"
}

# Run the dynamic library fuzzer
Write-Host "Running dynamic library fuzzer..."
cargo fuzz run dynamic_library $AsanFlag -- -max_len=$MaxLen -runs=$Runs -artifact_prefix="$OutputDir/dynamic_library_"

# Run the plugin command fuzzer
Write-Host "Running plugin command fuzzer..."
cargo fuzz run plugin_command $AsanFlag -- -max_len=$MaxLen -runs=$Runs -artifact_prefix="$OutputDir/plugin_command_"

Write-Host "Fuzzing complete!"
Write-Host "Any crashes or hangs will be saved to $OutputDir/ directory."
Write-Host "Run with -NoAsan to disable Address Sanitizer if you encounter ASAN-related errors." 