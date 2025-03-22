## WebSocket Testing Tools

The monitoring crate includes comprehensive WebSocket testing tools to validate the dashboard's real-time communication capabilities:

### Enhanced WebSocket Test Client

The `enhanced_websocket_test` example provides a powerful tool for testing the WebSocket server with multiple clients, subscription management, and automatic reconnection:

```bash
# Run with default settings
cargo run --example enhanced_websocket_test

# Customize test parameters
cargo run --example enhanced_websocket_test -- --num-clients 20 --duration 180
```

### Test Runner Scripts

Platform-specific scripts are provided to automate test execution and generate detailed reports:

- **Windows**: `scripts/run_websocket_tests.ps1`
- **Linux/macOS**: `scripts/run_websocket_tests.sh`

### Documentation

For detailed information about the WebSocket testing tools, see:

- [WebSocket Testing Guide](docs/websocket_testing.md) 