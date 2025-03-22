# Squirrel Web Interface

The web interface for the Squirrel system, providing HTTP API access to Squirrel's functionality.

## Features

- RESTful API endpoints for interacting with the Squirrel system
- Authentication and authorization with JWT
- Job management functionality
- Health check endpoints

## Build Instructions

The web crate supports two different build modes through feature flags:

### Mock Database Mode (Default)

The default build uses the `mock-db` feature, which provides in-memory storage without requiring a database setup:

```bash
# Build with default features (mock-db)
cargo build -p squirrel-web
```

This mode is ideal for:
- Development without database setup
- Quick testing and prototyping
- CI/CD pipelines

### Database Mode

To build with actual database support (SQLite), use the `db` feature:

```bash
# Build with database support
cargo build -p squirrel-web --no-default-features --features db
```

This mode requires:
- A SQLite database set up with the appropriate schema
- SQLx offline mode preparation or direct database access

## Running with SQLx Offline Mode

If you want to build with database mode but don't have a database available, you can use SQLx offline mode:

```bash
# Prepare SQLx data for offline mode
./prepare.ps1  # Windows
./prepare.sh   # Linux/Mac

# Build with offline mode enabled
$env:SQLX_OFFLINE="true"; cargo build -p squirrel-web --no-default-features --features db
```

## Setup Database

To set up the SQLite database for development:

```bash
# Windows
./setup-db.ps1

# Linux/Mac
chmod +x setup-db.sh
./setup-db.sh
```

## Testing

Run tests with:

```bash
# Test with mock database
cargo test -p squirrel-web

# Test with real database
cargo test -p squirrel-web --no-default-features --features db
```

## API Documentation

Comprehensive API documentation is available in the `/specs/web/API.md` file. 