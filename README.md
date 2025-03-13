# Code Analysis and Reporting System

A comprehensive system for analyzing code, generating reports, and providing insights through a web interface. Built with Rust for performance and reliability.

## Project Structure

The project is organized as a Rust workspace with the following crates:

- `analysis`: Core code analysis functionality
- `reporting`: Report generation and management
- `mcp`: Machine Context Protocol implementation
- `web`: Web interface and API

## Features

- Code analysis with support for:
  - Security scanning
  - Performance analysis
  - Style checking
  - Best practices validation
- Report generation in multiple formats:
  - Markdown
  - HTML
  - PDF
  - JSON
- Machine Context Protocol (MCP) for client-server communication
- Web interface for:
  - Submitting analysis jobs
  - Tracking job progress
  - Viewing and downloading reports

## Prerequisites

- Rust (latest stable version)
- PostgreSQL (14.0 or later)
- OpenSSL development libraries
- Git

## Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd <repository-name>
   ```

2. Set up the database:
   ```bash
   # Create a new PostgreSQL database
   createdb code_analysis
   
   # Run migrations (once implemented)
   cargo run -p web -- migrate
   ```

3. Configure the environment:
   ```bash
   # Copy the example configuration
   cp config.example.toml config.toml
   
   # Edit the configuration file with your settings
   vim config.toml
   ```

4. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

1. Start the web server:
   ```bash
   cargo run -p web
   ```

2. Start the MCP server:
   ```bash
   cargo run -p mcp -- server
   ```

3. Access the web interface at `http://localhost:3000`

## API Documentation

The web interface provides a RESTful API:

- `POST /api/jobs`: Create a new analysis job
- `GET /api/jobs/:id`: Get job status
- `GET /api/jobs/:id/report`: Download job report

For detailed API documentation, see [API.md](docs/API.md).

## Development

1. Install development dependencies:
   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-audit cargo-watch
   ```

2. Run tests:
   ```bash
   cargo test --all-features
   ```

3. Run lints:
   ```bash
   cargo clippy --all-targets --all-features
   ```

4. Format code:
   ```bash
   cargo fmt --all
   ```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 