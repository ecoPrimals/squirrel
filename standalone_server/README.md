# Standalone Task Server

This is a standalone implementation of the Task Management Server that provides task management functionality via gRPC.

## Features

- Create, update, and manage tasks
- Assign tasks to agents
- Report task progress
- Complete or cancel tasks
- Query task status
- List tasks with filtering

## Building

```bash
cargo build
```

## Running the Server

```bash
cargo run --bin taskserver
```

By default, the server listens on `[::1]:50052`. You can change this with the `-a` or `--address` option:

```bash
cargo run --bin taskserver -- --address 0.0.0.0:50052
```

## Running the Test Client

A test client is included to verify the server's functionality:

```bash
cargo run --bin taskclient
```

## Automated Testing

You can run the automated test script:

```bash
./tests/run_test.sh
```

This will:
1. Build the server and client
2. Start the server (or use an existing one)
3. Run the test client
4. Verify the results

## Protocol

The task management protocol is defined in `proto/mcp_task.proto`, which includes:

- Task service definition
- Task data structures
- Request/response messages

## Implementation

The server is implemented in Rust using:
- `tokio` for async runtime
- `tonic` for gRPC
- `prost` for Protocol Buffers
- In-memory task storage (can be extended to use a database)

## License

This project is part of the Squirrel ecosystem. 