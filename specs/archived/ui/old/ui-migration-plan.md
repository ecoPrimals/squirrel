---
title: UI Migration Plan
version: 1.0.0
date: 2024-03-26
status: planning
---

# UI Migration Plan: Web UI Extraction and Reorganization

## Overview

This document outlines the plan for migrating the current web UI implementation from the `web` crate to a dedicated UI architecture. The migration will establish a clear separation between the web server/API implementation and the UI components, allowing for better maintainability, reuse, and alignment with the broader UI strategy.

## Current State

The current web UI implementation:

- Is embedded directly in the `web` crate's `static` directory
- Uses a simple HTML/CSS/JavaScript implementation
- Is served directly by the web server using the Axum ServeDir middleware
- Has tight coupling between the API server and UI implementation
- Lacks a formal build process for UI assets

## Target State

The target architecture after migration:

- Dedicated `squirrel-ui-web` crate for web UI components
- Clear separation between API server and UI implementation
- Modern build system for web assets with optimization
- Clear API client abstraction for UI-to-backend communication
- Proper code organization following frontend best practices
- Capability to share models and interfaces with other UI implementations

## Current Progress

As of the latest update, significant progress has been made on the UI migration:

- ✅ Created `ui-web` crate with proper directory structure
- ✅ Migrated static files from `web/static` to `ui-web/web`
- ✅ Implemented build system in `build.rs` to handle asset copying and generation
- ✅ Set up organized directories for CSS, JavaScript, and other assets
- ✅ Updated web server to serve UI from the new location
- ✅ Removed old UI files from the web crate
- ✅ Created build and run scripts for local development with clean process management
- ✅ Updated documentation for integration points and workflows

In progress:
- 🔄 API client abstraction implementation
- 🔄 Component-based architecture for JavaScript code
- 🔄 Enhanced styling and UX improvements

## Migration Phases

### Phase 1: Preparation (Week 1)

1. **Create UI Crate**
   - Create the `squirrel-ui-web` crate
   - Set up basic crate structure and dependencies
   - Configure build tools for UI assets
   - Establish CI/CD workflows for UI builds

2. **API Client Abstraction**
   - Define API client interfaces
   - Implement initial API client for core endpoints
   - Create shared models for API communication

3. **Documentation Update**
   - Document API endpoints used by UI
   - Create UI component architecture documentation
   - Define integration points with backend

**Deliverables:**
- New crate structure
- Initial API client implementation
- Updated documentation
- Build system configuration

### Phase 2: Component Migration (Week 2)

1. **Core Components**
   - Migrate base HTML/CSS structure to new crate
   - Refactor JavaScript into component-based architecture
   - Implement initial UI framework integration (if applicable)
   - Create shared layout components

2. **Feature Components**
   - Migrate command execution interface
   - Migrate status and health monitoring views
   - Migrate job management interface
   - Migrate logging and event views

3. **Testing Framework**
   - Set up testing infrastructure for UI components
   - Create initial component tests
   - Implement integration tests for API communication

**Deliverables:**
- Migrated UI components
- Component-based architecture
- Initial test suite
- Feature parity with current implementation

### Phase 3: Integration (Week 3)

1. **Backend Integration**
   - Modify web server to serve UI from new location
   - Update routing configuration
   - Implement API version compatibility handling
   - Create development mode for UI with hot reloading

2. **Authentication Integration**
   - Refactor authentication handling
   - Implement secure token storage
   - Add login UI improvements

3. **WebSocket Integration**
   - Refactor WebSocket connection handling
   - Implement reconnection logic
   - Add better event visualization

**Deliverables:**
- Integrated backend and UI
- Enhanced authentication flow
- Improved real-time functionality
- Development workflow documentation

### Phase 4: Cleanup and Optimization (Week 4)

1. **Remove Legacy Code**
   - Remove UI code from web crate
   - Update documentation references
   - Update build scripts

2. **Optimization**
   - Optimize asset loading
   - Implement code splitting
   - Add caching strategies
   - Improve load time performance

3. **Final Testing**
   - End-to-end testing of complete system
   - Cross-browser compatibility testing
   - Accessibility compliance testing
   - Performance testing

**Deliverables:**
- Cleaned codebase with no UI in web crate
- Optimized UI build
- Complete test coverage
- Performance metrics documentation

## Technical Details

### Directory Structure Changes

**Current:**
```
crates/web/
├── src/               # Rust code for web server
├── static/            # UI files (HTML, CSS, JS)
└── ...
```

**Target:**
```
crates/web/
├── src/               # Rust code for web server
└── ...

crates/squirrel-ui-web/
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
└── dist/              # Built artifacts
```

### Build System Changes

**Current:**
- ✅ Basic build script to copy files from `web/` to `dist/`
- ✅ PowerShell and Bash scripts for different environments
- ✅ Combined build-and-run scripts for development workflow
- 🔄 Future improvements for minification or optimization

**Target:**
- Modern frontend build system (e.g., Trunk, Webpack)
- Asset optimization and minification
- Source maps for development
- CSS preprocessing
- Module bundling
- Development server with hot reloading

### Web Server Changes

**Current:**
```rust
// Add static file serving for the UI
let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
if static_dir.exists() {
    app = app.nest_service("/", ServeDir::new(static_dir));
}
```

**Target:**
```rust
// Serve pre-built UI from the squirrel-ui-web crate
let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent().unwrap() // Go up to crates/
    .join("squirrel-ui-web/dist");

if ui_dir.exists() {
    app = app.nest_service("/", ServeDir::new(ui_dir));
} else {
    tracing::warn!("UI directory {:?} does not exist. UI will not be available.", ui_dir);
}
```

## API Changes

The migration will not change API endpoints, but will improve how the UI interacts with them:

1. **API Client Abstraction**
   - Create typed API client for UI-to-backend communication
   - Implement error handling and retry logic
   - Add request/response logging for debugging

2. **WebSocket Improvements**
   - Enhance reconnection logic
   - Add structured message types
   - Implement proper error handling

3. **Authentication Enhancements**
   - Improve token management
   - Add refresh token support
   - Implement secure storage

## Testing Strategy

The migration will include comprehensive testing:

1. **Component Tests**
   - Unit tests for UI components
   - Visual regression tests for UI appearance

2. **Integration Tests**
   - Tests for API client functionality
   - Authentication flow tests
   - WebSocket communication tests

3. **End-to-End Tests**
   - Complete user workflow tests
   - Cross-browser compatibility tests
   - Mobile responsiveness tests

## Risk Assessment and Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| API incompatibility | Medium | High | Detailed API documentation, version compatibility layer |
| Build system complexity | Medium | Medium | Start with simple build process, add complexity incrementally |
| Performance regression | Low | High | Performance testing before/after migration |
| Development workflow disruption | Medium | Medium | Clear documentation, development mode with hot reloading |
| Feature regression | Medium | High | Comprehensive test coverage, feature checklist |

## Backward Compatibility

During migration, backward compatibility will be maintained:

1. **Dual Deployment**
   - Keep existing UI in place until migration is complete
   - Implement feature flags for switching between implementations
   - Allow rollback to previous implementation if issues are found

2. **API Versioning**
   - Maintain compatibility with existing API endpoints
   - Use API versioning for any changes
   - Document API changes carefully

## Dependencies and Prerequisites

The migration depends on:

1. **Technical Requirements**
   - Frontend build tools (Trunk, npm, etc.)
   - API documentation completeness
   - Test infrastructure for UI components

2. **Process Requirements**
   - Agreement on UI architecture
   - Approval of migration plan
   - Allocation of development resources

## Post-Migration Tasks

After the migration is complete:

1. **Documentation**
   - ✅ Update all references to UI implementation
   - ✅ Create integration documentation
   - ✅ Document development workflow
   - 🔄 Create component documentation

2. **Training**
   - Train developers on new UI architecture
   - Create onboarding guides for UI development

3. **Monitoring**
   - Implement UI performance monitoring
   - Track usage patterns and errors
   - Gather user feedback

## Timeline and Milestones

| Week | Milestone | Completion Criteria |
|------|-----------|---------------------|
| 1 | Preparation | UI crate created, build system configured, API client defined |
| 2 | Component Migration | All UI components migrated, test coverage established |
| 3 | Integration | Backend integrated with new UI, all features working |
| 4 | Cleanup and Optimization | Legacy code removed, performance optimized, tests passing |

## Conclusion

This migration will establish a solid foundation for future UI development by properly separating concerns, implementing modern frontend practices, and aligning with the broader UI strategy. By following this plan, we will achieve a cleaner architecture, better maintainability, and improved development workflow while maintaining backward compatibility and minimizing disruption.

## References

- [Web UI Strategy](./web-ui-strategy.md)
- [Terminal UI Specifications](./README.md)
- [Component Architecture](./component-architecture.md)
- [Web API Documentation](../api/README.md) 