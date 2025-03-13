# Income Generation System Test Suite

This directory contains the test suite for the income generation system use cases. The tests are organized by use case and follow the standard Rust testing practices.

## Directory Structure

```
tests/
├── use_cases/
│   ├── mod.rs                      # Module organization
│   ├── code_understanding_tests.rs  # Tests for code understanding features
│   ├── project_analysis_tests.rs    # Tests for project analysis features
│   └── error_resolution_tests.rs    # Tests for error resolution features
└── README.md                        # This file
```

## Test Categories

### Code Understanding Tests
- Explanation quality validation
- Context-aware code analysis
- Suggestion relevance testing
- Help system completeness

### Project Analysis Tests
- Project structure detection
- Language identification
- Dependency analysis
- Module organization validation

### Error Resolution Tests
- Error explanation clarity
- Fix suggestion validation
- Interactive resolution workflow
- Resolution effectiveness metrics

## Running Tests

To run all tests:
```bash
cargo test
```

To run tests for a specific use case:
```bash
cargo test code_understanding  # For code understanding tests
cargo test project_analysis   # For project analysis tests
cargo test error_resolution  # For error resolution tests
```

## Test Metrics

Each test category includes specific metrics for validation:

1. Code Understanding
   - Completeness: 0.8+
   - Accuracy: 0.9+
   - Clarity: 0.85+

2. Project Analysis
   - Structure Detection: 0.9+
   - Language Detection: 0.95+
   - Dependency Analysis: 0.85+

3. Error Resolution
   - Explanation Clarity: 0.9+
   - Fix Validity: 0.85+
   - Resolution Success: 0.9+

## Contributing

When adding new tests:
1. Follow the existing module structure
2. Include appropriate test contexts
3. Add metrics for validation
4. Update this README as needed 