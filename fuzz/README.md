# Squirrel Plugin System - Fuzzing Infrastructure

This directory contains the fuzzing infrastructure for the Squirrel Plugin System. Fuzzing is an automated testing technique that repeatedly runs a program with different inputs to discover potential issues.

## Fuzzing Targets

The Squirrel Plugin System currently has two main fuzzing targets:

1. **Dynamic Library Fuzzer (`dynamic_library`)**: Tests the plugin system's ability to safely load and process dynamic libraries.

2. **Plugin Command Fuzzer (`plugin_command`)**: Tests the plugin system's command execution pipeline.

## Corpus Management

- `fuzz/corpus/dynamic_library/`: Contains seed files for the dynamic library fuzzer
- `fuzz/corpus/plugin_command/`: Contains seed files for the plugin command fuzzer

## Running the Fuzzers

### Prerequisites

Ensure you have installed `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

### Windows

```powershell
.\fuzz\run_fuzzers.ps1
```

### Linux/macOS

```bash
chmod +x ./fuzz/run_fuzzers.sh
./fuzz/run_fuzzers.sh
```

### Manual Running

```bash
# Run the dynamic library fuzzer
cargo fuzz run dynamic_library

# Run the plugin command fuzzer
cargo fuzz run plugin_command
```

## CI/CD Integration

Fuzzing is integrated into the CI/CD pipeline as defined in `fuzz/ci_fuzzing.yml`.

## Adding New Fuzzers

To add a new fuzzer:

1. Create a new file in the `fuzz/fuzz_targets/` directory
2. Use the `fuzz_target!` macro to define the entry point
3. Add the target to `fuzz/Cargo.toml` as a `[[bin]]` entry
4. Add seed files to the corpus directory

For more detailed information, see the [Fuzzing Guide](../docs/plugins/fuzzing_guide.md). 