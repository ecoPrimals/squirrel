# Web <-> MCP gRPC Integration Testing Plan

## 1. Goal

Verify reliable, correct, and resilient gRPC communication between the `squirrel-web` frontend server and the `squirrel-mcp` backend service, specifically focusing on interactions initiated via the `squirrel-web` API.

## 2. Scope

Initial testing will focus on the core command execution workflow:

- **Command Execution:** Sending a command request from `web_server` to `mcp_server` via the `/api/commands` endpoint.
- **Status Retrieval:** Querying the status of an executed command via the `/api/commands/:id` endpoint.
- **Basic Error Handling:** Verifying appropriate error responses for invalid commands or communication failures.

Future scope can include:
- WebSocket event propagation from MCP through the web server.
- Testing other MCP gRPC services if exposed via the web API (e.g., sync service).
- More complex command scenarios (long-running tasks, cancellation).

## 3. Environment Setup

Integration tests require both the `web_server` and the `mcp_server` (gRPC) to be running concurrently.

- **Process Management:**
    - Tests should ideally manage the lifecycle of both server processes.
    - Use `std::process::Command::new("cargo").args(["run", "-p", ...])` to start each server in the background.
    - Ensure processes are properly terminated after tests (e.g., using `child.kill()`). Consider using libraries like `test-context` or custom setup/teardown logic if process management becomes complex.
- **Ports:**
    - Use dynamically assigned ports for both servers during testing to prevent conflicts with running instances or parallel tests. (`TcpListener::bind("127.0.0.1:0")`)
    - The test setup needs to capture the assigned ports.
- **Web Server Configuration:**
    - The `web_server` needs to be configured to connect to the **test instance** of the `mcp_server`.
    - Create a test-specific configuration (`Config`) within the test setup, overriding the default `mcp.host` and `mcp.port` with the dynamically assigned port of the test `mcp_server`.
    - Pass this test `Config` when calling `create_app` in the test helper (`spawn_app`).
- **MCP Server:**
    - Assume an executable target exists for the MCP gRPC server (e.g., `cargo run -p squirrel-mcp --bin mcp_server`). Verify the exact command.
    - Ensure the MCP server can also be configured to listen on a dynamic port passed via arguments or environment variables.

## 4. Key Test Cases

**(Initial Set)**

1.  **`test_execute_valid_command`**
    - Setup: Start `mcp_server`, start `web_server` configured to connect to `mcp_server`.
    - Action: Send `POST /api/commands` with a valid command name (e.g., `echo`, needs verification) and parameters.
    - Verify:
        - `web_server` returns HTTP 2xx success.
        - Response body (JSON) indicates success (e.g., `{"success": true, ...}`).
        - Response body contains a non-empty string `command_id` (e.g., `{"data": {"command_id": "..."}}`).
        - (Optional) Check `mcp_server` logs/state to confirm command reception/execution.

2.  **`test_get_command_status`**
    - Setup: Execute a valid command as in `test_execute_valid_command`, obtain `command_id`.
    - Action: Send `GET /api/commands/{command_id}`.
    - Verify:
        - `web_server` returns HTTP 2xx success.
        - Response body contains the correct command details, including the ID and a valid status (e.g., `Queued`, `Running`, `Completed`).
        - (Optional) Poll status until `Completed` (for short commands like `echo`).

3.  **`test_execute_invalid_command`**
    - Setup: Start servers as above.
    - Action: Send `POST /api/commands` with a non-existent command name.
    - Verify:
        - `web_server` returns an appropriate HTTP error status (e.g., 400 Bad Request or 500 Internal Server Error depending on implementation).
        - Response body indicates failure with a relevant error message.

4.  **`test_mcp_server_unavailable`**
    - Setup: Start `web_server` only, configured to connect to a port where `mcp_server` is *not* running.
    - Action: Send `POST /api/commands`.
    - Verify:
        - `web_server` returns an appropriate HTTP error status (e.g., 500 Internal Server Error or 503 Service Unavailable).
        - Response body indicates MCP connection failure.

## 5. Tooling

- **Test Runner:** `tokio::test` attribute for async test functions.
- **HTTP Client:** `reqwest` crate for making API calls to the `web_server`.
- **Process Management:** `std::process::Command` for starting/stopping server processes.
- **Assertions:** Standard `assert!` macros, potentially `assert_eq!` for specific response fields.
- **Logging:** `tracing` crate. Configure test logging to capture server and test output.
- **Configuration:** `config` crate (if used by servers) or manual struct creation for test configurations.

## 6. Implementation Strategy

1.  Create helper functions for starting/stopping `web_server` and `mcp_server` with dynamic ports and appropriate configuration.
2.  Implement `test_execute_valid_command` first, refining server management and basic assertions.
3.  Implement `test_get_command_status`.
4.  Implement error case tests (`test_execute_invalid_command`, `test_mcp_server_unavailable`).
5.  Refactor common setup/assertions into helper functions. 