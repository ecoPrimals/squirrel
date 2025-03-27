# PowerShell script to run all fuzzers for the Squirrel Plugin System

# Create output directory
New-Item -ItemType Directory -Path "fuzz_output" -Force | Out-Null

# Build the fuzzers
Write-Host "Building fuzzers..."
cargo build --release --bin fuzz_dynamic_library
cargo build --release --bin fuzz_plugin_command

# Set environment variables for better debugging
$env:RUST_BACKTRACE = "1"

# Run the dynamic library fuzzer
Write-Host "Running dynamic library fuzzer..."
New-Item -ItemType Directory -Path "fuzz/corpus/dynamic_library" -Force | Out-Null
& "./target/release/fuzz_dynamic_library.exe" "fuzz/corpus/dynamic_library" "-max_len=10240" "-runs=10000" "-artifact_prefix=fuzz_output/dynamic_library_"

# Run the plugin command fuzzer
Write-Host "Running plugin command fuzzer..."
New-Item -ItemType Directory -Path "fuzz/corpus/plugin_command" -Force | Out-Null
& "./target/release/fuzz_plugin_command.exe" "fuzz/corpus/plugin_command" "-max_len=10240" "-runs=10000" "-artifact_prefix=fuzz_output/plugin_command_"

Write-Host "Fuzzing complete!"
Write-Host "Any crashes or hangs will be saved to fuzz_output/ directory." 