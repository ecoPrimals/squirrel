# Squirrel Plugin System Fuzzing Infrastructure

This directory contains the fuzzing infrastructure for the Squirrel Plugin System. The fuzzing infrastructure is designed to identify potential bugs and vulnerabilities in the plugin system by providing randomly generated input to key components.

## Fuzzing Targets

The system currently supports the following fuzzing targets:

1. **Dynamic Library Fuzzer**: Tests the dynamic loading of libraries as plugins
   - Located in `dynamic_library.rs`
   - Provides random byte sequences as potential plugin libraries
   - Tests the system's ability to safely handle invalid libraries

2. **Plugin Command Fuzzer**: Tests the plugin command execution system
   - Located in `plugin_command.rs`
   - Provides random JSON data as command arguments
   - Tests parameter validation and error handling in command processing

## Corpus Management

The fuzzing corpus is stored in the `corpus` directory, organized by fuzzing target:

- `corpus/dynamic_library/`: Seed files for the dynamic library fuzzer
- `corpus/plugin_command/`: Seed files for the plugin command fuzzer

## Running the Fuzzers

To run a fuzzer, use the following command:

```bash
cargo run --bin fuzz_dynamic_library  # To run the dynamic library fuzzer
cargo run --bin fuzz_plugin_command   # To run the plugin command fuzzer
```

For continuous fuzzing in a CI environment, use:

```bash
cargo fuzz run dynamic_library
cargo fuzz run plugin_command
```

## Structure-Aware Fuzzing

The fuzzing infrastructure uses structure-aware fuzzing techniques to generate inputs that are more likely to exercise interesting code paths:

- The dynamic library fuzzer uses seed files that contain valid PE/ELF headers
- The plugin command fuzzer uses seed files that contain valid JSON structures
- Custom parse functions transform random bytes into structured inputs

## Cross-Platform Support

The fuzzing infrastructure is designed to work on all major operating systems:

- Windows: Tests DLL loading behavior
- Linux: Tests shared object (.so) loading behavior
- macOS: Tests dynamic library (.dylib) loading behavior

## Integration with CI/CD

The fuzzing infrastructure is integrated with the CI/CD pipeline. The following checks are performed:

1. Regression testing: Ensuring previously identified bugs don't reappear
2. Coverage analysis: Tracking code coverage from fuzzing runs
3. Time-boxed fuzzing: Running fuzzers for a fixed amount of time on each commit
4. Artifact collection: Saving crash reproducer files for detailed analysis

## Adding New Fuzzers

To add a new fuzzer:

1. Create a new Rust file with your fuzzer implementation
2. Add appropriate seed files in the corpus directory
3. Add a new binary target in the `Cargo.toml` file
4. Update the CI configuration to include the new fuzzer

## Best Practices

When adding new features to the plugin system, consider:

1. Adding new fuzz targets for the feature
2. Adding seed files that exercise the new code paths
3. Updating existing fuzzers to cover the new functionality
4. Running fuzzers locally before submitting a pull request 