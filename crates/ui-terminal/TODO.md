# Terminal UI TODO List

## Critical Fixes

- [x] Fix compilation errors in the monitoring crate - add missing sysinfo trait imports:
  - [x] Add `use sysinfo::SystemExt;` to required files
  - [x] Add `use sysinfo::ProcessExt;` to required files
  - [x] Add `use sysinfo::NetworksExt;` or fix `new_with_refreshed_list()` calls

- [x] Fix Protocol Widget implementation errors:
  - [x] Fix Frame generic parameter usage
  - [x] Add missing type annotations in render methods
  - [x] Fix ChartWidget::new parameter mismatch
  - [x] Fix unused imports and variables

## Testing

- [x] Add basic integration tests for dashboard core and UI components
- [ ] Expand test coverage for all UI widgets
- [ ] Add more comprehensive integration tests between components
- [x] Create test fixtures for metrics simulation

## Feature Completion

- [x] Complete the Protocol tab
  - [x] Add protocol status visualization
  - [x] Implement transaction monitoring
  - [x] Add protocol metrics visualization
  - [x] Add MCP-specific metrics to Protocol Widget

- [ ] Complete the Alerts tab
  - [ ] Add alerts list view
  - [ ] Implement alert filtering
  - [ ] Add alert details panel
  - [ ] Implement alert acknowledgment 

- [ ] Complete the Network tab
  - [ ] Add network throughput charts
  - [ ] Implement connection monitoring
  - [ ] Add packet analysis view

- [ ] Complete the Tools tab
  - [ ] Implement log viewer
  - [ ] Add configuration editor
  - [ ] Add system actions panel

## Integration

- [x] Prepare Protocol Widget for MCP integration
- [x] Structure Protocol Adapter for real MCP metrics 
- [ ] Connect to actual MCP crate for metrics collection
- [ ] Implement proper error handling for integration points
- [ ] Add configurable metrics polling rate

## UI Improvements

- [ ] Add theme support
  - [ ] Create light and dark themes
  - [ ] Implement theme switching
  - [ ] Add custom theme loader

- [ ] Enhance widget functionality
  - [ ] Add zoom support to charts
  - [ ] Implement scrollable tables
  - [ ] Add interactive elements to widgets

- [ ] Improve help system
  - [ ] Create context-sensitive help
  - [ ] Add keyboard shortcut reference
  - [ ] Implement tutorial mode

## Configuration

- [ ] Add configuration file support
  - [ ] Implement config file loading/saving
  - [ ] Add user preferences
  - [ ] Create default configuration

- [ ] Enhance CLI options
  - [ ] Add output format options
  - [ ] Implement logging level control
  - [ ] Add debug mode flag

## Documentation

- [x] Update README with new features
- [x] Document protocol metrics in dashboard_api.md
- [ ] Create user guide for terminal UI
- [ ] Document keyboard shortcuts

## Advanced Features

- [ ] Add export functionality
  - [ ] CSV export
  - [ ] JSON export
  - [ ] Snapshot functionality

- [ ] Implement advanced alerting system
  - [ ] Threshold-based alerts
  - [ ] Alert notifications
  - [ ] Alert history and reporting

## Performance Optimizations

- [ ] Optimize rendering pipeline
- [ ] Reduce memory usage for large datasets
- [ ] Improve update frequency for real-time monitoring
- [ ] Add caching for historical data

## Release Preparation

- [ ] Version bump to 0.1.0
- [ ] Create release notes
- [ ] Perform final testing
- [ ] Update changelog 

## Next Steps (Priority Order)

1. Connect Protocol Widget to real MCP metrics from the MCP crate
2. Complete the Alerts tab functionality 
3. Enhance test coverage across all components
4. Implement configuration file support
5. Add theme support and widget interactivity 