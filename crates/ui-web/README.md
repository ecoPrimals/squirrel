# Squirrel Web UI

This crate provides the web-based user interface for the Squirrel system. It's designed to work in conjunction with the `squirrel-web` crate, which provides the API server.

## Overview

The Squirrel Web UI is a browser-based interface that communicates with the Squirrel Web API. It provides a user-friendly way to interact with the Squirrel system, including:

- Executing commands
- Managing jobs
- Viewing system status
- Monitoring logs
- Authentication

## Architecture

The UI is built using a modular architecture:

- **Assets**: Static files like HTML, CSS, and JavaScript
- **Components**: Reusable UI components for building interfaces
- **API Client**: Typed client for communicating with the Squirrel Web API

### Directory Structure

```
src/
├── api/           # API client modules
├── components/    # UI component modules
├── assets/        # Asset management
└── lib.rs         # Main library entry point

web/
├── css/           # CSS stylesheets
├── js/            # JavaScript files
├── assets/        # Static assets (images, fonts, etc.)
└── index.html     # Main HTML template
```

## Usage

### Building the UI

To build the UI, run:

```bash
cargo build
```

### Running the UI with the Web Server

The UI is designed to be served by the `squirrel-web` server. To run the complete application:

1. Build the UI crate
2. Run the web server with the UI integrated

```bash
cargo run -p squirrel-web
```

### Development Mode

For development, you can use:

```bash
cargo watch -x 'run -p squirrel-web'
```

This will automatically rebuild and restart the server when changes are made.

## API Client

The API client provides a typed interface for communicating with the Squirrel Web API:

```rust
use squirrel_ui_web::api::ApiClient;

// Create a new API client
let client = ApiClient::new(ApiClientConfig::default());

// Initialize the client
client.init().await?;

// Access the commands API
let commands = client.commands().get_available_commands().await?;
```

## Components

The UI components provide a way to build interfaces:

```rust
use squirrel_ui_web::components::{Layout, create_header, create_footer};

// Create a layout
let mut layout = Layout::new(
    create_header(),
    create_navigation(),
    create_footer(),
);

// Set content
layout.set_content("Hello, world!");

// Render the layout
let html = layout.render();
```

## License

This project is licensed under [LICENSE INFORMATION]. 