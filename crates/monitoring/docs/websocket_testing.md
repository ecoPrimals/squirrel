---
title: WebSocket Testing Guide
version: 1.0.0
last_updated: 2024-04-01
status: active
---

# WebSocket Testing Guide

This document explains how to use the enhanced WebSocket testing tools to thoroughly test the monitoring system's WebSocket implementation.

## Overview

The monitoring system includes an enhanced WebSocket testing framework designed to validate:

1. **Concurrent Connection Handling**: Tests the server's ability to handle multiple simultaneous client connections
2. **Subscription Management**: Verifies proper handling of component subscriptions
3. **Message Delivery**: Tests real-time message delivery from server to clients
4. **Reconnection Handling**: Validates automatic reconnection and recovery
5. **Error Management**: Tests error handling and recovery mechanisms
6. **Performance**: Measures message throughput and latency under load

## Test Components

### Enhanced WebSocket Test Client

The `enhanced_websocket_test` is a comprehensive WebSocket client simulation tool that:

- Creates multiple concurrent client connections
- Subscribes to different monitoring components
- Tracks message reception statistics
- Simulates random disconnections to test reconnection
- Measures performance metrics
- Generates detailed test reports

### Test Runner Scripts

Two platform-specific test runner scripts are provided:

1. **PowerShell Script** (`run_websocket_tests.ps1`): For Windows environments
2. **Bash Script** (`run_websocket_tests.sh`): For Linux/macOS environments

These scripts automate the test execution and generate detailed Markdown reports with charts.

## Running the Tests

### Prerequisites

Before running the tests, ensure:

1. The monitoring system is properly configured
2. You have required dependencies:
   - Rust and Cargo
   - PowerShell (Windows) or Bash (Linux/macOS)
   - WebSocket server available (either running or started by the script)

### Using the Test Runner Scripts

#### Windows (PowerShell)

```powershell
# Run with default settings
./scripts/run_websocket_tests.ps1

# Customize test parameters
./scripts/run_websocket_tests.ps1 -NumClients 30 -TestDurationSeconds 300 -ReportPath "./reports/websocket_test.md" -VerboseOutput
```

#### Linux/macOS (Bash)

```bash
# Make script executable
chmod +x ./scripts/run_websocket_tests.sh

# Run with default settings
./scripts/run_websocket_tests.sh

# Customize test parameters
./scripts/run_websocket_tests.sh --num-clients 30 --duration 300 --report "./reports/websocket_test.md" --verbose
```

### Running the Test Client Directly

```bash
# Run with default settings
cargo run --example enhanced_websocket_test

# Customize test parameters
cargo run --example enhanced_websocket_test -- --num-clients 20 --duration 180 --url ws://localhost:8765/ws

# Show help
cargo run --example enhanced_websocket_test -- --help
```

## Test Parameters

| Parameter | Description | Default Value |
|-----------|-------------|---------------|
| `--num-clients` | Number of concurrent client connections | 10 |
| `--duration` | Test duration in seconds | 120 |
| `--url` | WebSocket server URL | ws://localhost:8765/ws |
| `--report` | Path to save the report (script only) | ./websocket_test_report.md |
| `--verbose` | Show verbose output (script only) | false |
| `--no-chart` | Disable chart generation (bash script only) | false |

## Test Report Format

The generated test report includes:

1. **Test Configuration**: Details about the test setup
2. **Test Results Summary**: Key metrics from the test run
3. **Messages Per Component**: Breakdown of messages received by component
4. **Client Performance**: Statistics for each client connection
5. **Performance Analysis**: Key performance metrics and analysis
6. **Reconnection Performance**: Details about reconnection testing
7. **Performance Charts**: Visual representation of test results
8. **Conclusion**: Analysis of overall WebSocket performance

## Example Report

```markdown
# WebSocket Test Report

## Test Configuration
- **Date**: 2024-04-01 15:30:45
- **Number of Clients**: 20
- **Test Duration**: 180 seconds
- **Server Address**: ws://localhost:8765/ws

## Test Results Summary

| Metric | Value |
|--------|-------|
| Connections Successful | 45 |
| Connection Failures | 2 |
| Connections Closed | 45 |
| Total Messages Received | 5230 |
| Reconnection Tests | 8 |

## Messages Per Component
...

## Performance Analysis

| Metric | Value |
|--------|-------|
| Messages Per Second | 29.06 |
| Average Messages Per Client | 261.5 |
| Connection Success Rate | 95.74% |

## Conclusion

The WebSocket server performed **well** under load, with good message throughput and mostly reliable reconnections.
```

## Interpreting Test Results

### Key Metrics to Monitor

1. **Connection Success Rate**: Should be 95%+ for a healthy system
2. **Messages Per Second**: Higher is better; >50 is excellent
3. **Average Messages Per Client**: Indicates subscription management efficiency
4. **Reconnection Tests**: More tests provide higher confidence in reconnection logic

### Common Issues

| Issue | Possible Causes | Suggested Actions |
|-------|-----------------|-------------------|
| Low connection success rate | Network issues, server resource constraints | Check server resources, review error logs |
| Low message throughput | Inefficient message handling, serialization bottlenecks | Profile message processing, optimize payload size |
| Connection failures | Server overload, network issues | Check server logs, increase timeouts |
| Message distribution imbalance | Subscription handling issues | Review subscription logic |

## Advanced Testing Scenarios

### 1. High-Load Testing

Test with a large number of clients (50+) to identify scaling limits:

```powershell
./scripts/run_websocket_tests.ps1 -NumClients 50 -TestDurationSeconds 300
```

### 2. Long-Running Tests

Run extended tests to identify memory leaks or degradation over time:

```bash
./scripts/run_websocket_tests.sh --num-clients 10 --duration 3600
```

### 3. Component-Specific Testing

Modify the test client to focus on specific components for targeted testing.

### 4. Network Condition Simulation

Use a tool like [toxiproxy](https://github.com/Shopify/toxiproxy) to simulate poor network conditions:

```bash
# First set up toxiproxy, then run with custom URL
./scripts/run_websocket_tests.sh --url ws://localhost:8474/ws
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Test fails to connect | Ensure WebSocket server is running; check port and address |
| Script fails to parse output | Update regex patterns if test output format changes |
| Tests run slow | Check system resources; reduce number of clients |
| No report generated | Check directory permissions; ensure output path is valid |

## Developer Guide: Extending the Tests

### Adding New Test Scenarios

Modify `enhanced_websocket_test.rs` to add new test scenarios:

```rust
// Add new test case in handle_client_connection
// Example: Testing rapid subscription changes
for _ in 0..5 {
    // Subscribe then immediately unsubscribe
    let subscribe_msg = json!({"type": "subscribe", "componentId": component}).to_string();
    write.send(Message::Text(subscribe_msg)).await?;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let unsubscribe_msg = json!({"type": "unsubscribe", "componentId": component}).to_string();
    write.send(Message::Text(unsubscribe_msg)).await?;
}
```

### Adding Custom Metrics

To track additional metrics:

1. Add fields to the `TestStatistics` struct
2. Update collection logic in message handling
3. Modify report generation to include new metrics

### Customizing Test Reports

Edit the script files to modify report format and content:

- Add new sections to the report template
- Customize chart generation
- Include additional performance metrics

## Integration with CI/CD

The WebSocket tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
websocket-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v2
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Start WebSocket server
      run: cargo run --example dashboard_server &
    - name: Run WebSocket tests
      run: ./scripts/run_websocket_tests.sh --num-clients 10 --duration 120
    - name: Upload test report
      uses: actions/upload-artifact@v2
      with:
        name: websocket-test-report
        path: ./websocket_test_report.md
```

## Conclusion

The enhanced WebSocket testing framework provides comprehensive tools for validating the monitoring system's real-time communication capabilities. By regularly running these tests, you can ensure the WebSocket server remains performant and reliable even as the system evolves.

<version>1.0.0</version> 