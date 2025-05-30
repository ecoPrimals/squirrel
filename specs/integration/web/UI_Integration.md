---
title: UI Integration Specification
version: 1.0.0
date: 2024-03-26
status: approved
---

# Web UI Integration Specification

## Overview

This document specifies how the Web team should interact with the migrated UI components in the `ui-web` crate. It outlines the integration points, architecture, and development workflow to ensure consistency and avoid duplication of effort.

## Architecture

### Current Structure

The UI has been migrated from `crates/web/static` to a dedicated `crates/ui-web` crate with the following structure:

```
crates/ui-web/
├── src/               # Rust code for UI crate
│   ├── lib.rs         # Library entry point
│   ├── components/    # UI component definitions
│   ├── api/           # API client implementation
│   └── assets/        # Static assets management
├── web/               # Web frontend
│   ├── index.html     # Main HTML file
│   ├── css/           # Stylesheets
│   ├── js/            # JavaScript files
│   └── assets/        # Images and other assets
├── build/             # Build scripts
├── build.rs           # Asset build script
└── dist/              # Built artifacts
```

### Server Integration

The web server in `crates/web` has been updated to serve UI assets from the `crates/ui-web/dist` directory:

```rust
// In crates/web/src/lib.rs
let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent().unwrap() // Go up to crates/
    .join("ui-web/dist");

if ui_dir.exists() {
    tracing::info!("Serving UI files from {:?}", ui_dir);
    app = app.nest_service("/", ServeDir::new(ui_dir));
} else {
    tracing::warn!("UI directory {:?} does not exist. UI will not be available.", ui_dir);
}
```

## Integration Requirements

### For Web Team

When working on the web server or UI components:

1. **Do Not Create Duplicate UI**: All web UI components should be built in the `crates/ui-web` crate, not in the `crates/web` crate.

2. **API Integration**: Use the API client abstractions in `crates/ui-web/src/api/` to communicate with the backend.

3. **Component Architecture**: Follow the component-based architecture established in the UI crate.

4. **Asset Management**: Add new assets to the appropriate directories in `crates/ui-web/web/`.

5. **Build Process**: Use the `build-assets.ps1` script during development or run `cargo build` in the `ui-web` crate to update the `dist` directory.

### Integration Points

The following integration points between the web server and UI should be maintained:

1. **API Communication**: 
   - REST API endpoints at `/api/*`
   - WebSocket connection at `/ws`
   - Authentication via JWT at `/api/auth/*`

2. **Plugin System**:
   - Plugin UI components at `/api/plugins/*`
   - Plugin asset serving

3. **Static Asset Serving**:
   - All static assets served from the root path

## Development Workflow

### Running the Web Server

For local development, you can use the combined build and run scripts from the workspace root:

**Windows:**
```powershell
.\build-and-run.ps1
```

**Unix/Linux/macOS:**
```bash
./build-and-run.sh
```

These scripts will:
1. Stop any existing web server processes for clean restarts
2. Build the UI assets from `crates/ui-web/web` into `crates/ui-web/dist`
3. Start the web server from the `crates/web` directory

This ensures a consistent development environment and prevents issues with stale processes.

### Adding New UI Features

1. Define API requirements in `crates/ui-web/src/api/`
2. Implement UI components in `crates/ui-web/web/js/`
3. Add styles in `crates/ui-web/web/css/`
4. Update HTML structure in `crates/ui-web/web/index.html`
5. Build assets using the build script
6. Test integration with the web server

### Modifying Existing Features

1. Locate the component in `crates/ui-web/web/`
2. Make necessary changes
3. Update API client if needed
4. Rebuild assets
5. Test the changes

## Testing

All UI changes should be tested for:

1. **Functionality**: Ensure features work as expected
2. **API Integration**: Verify communication with backend endpoints
3. **Responsiveness**: Test on different screen sizes
4. **Cross-browser Compatibility**: Test in major browsers

## Documentation

When adding or modifying UI components:

1. Update this specification if integration patterns change
2. Document API client usage in component code
3. Follow the `web-ui-strategy.md` for architectural guidance

## Future Enhancements

The UI will continue to evolve according to the [UI Migration Plan](../ui/ui-migration-plan.md):

1. **API Client Abstraction**: More robust typesafe API clients
2. **Component Architecture**: Refactoring to a more modular structure
3. **Enhanced Styling**: Improved visual design and user experience
4. **Testing Framework**: Comprehensive UI test suite

## Conclusion

By following this specification, the Web team will maintain a clean separation between the web server and UI components, allowing for better maintainability, reuse, and alignment with the broader UI strategy. All web UI development should occur in the dedicated `ui-web` crate to avoid duplication of effort and ensure consistency.

---

Last Updated: March 26, 2024 