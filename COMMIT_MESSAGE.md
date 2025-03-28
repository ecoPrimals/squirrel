feat(ui-terminal): enhance CompressedTimeSeries with robust base point tracking and new features

This commit significantly improves the CompressedTimeSeries component with:

1. Fixed issues:
- Added has_base_point field to properly track base point status
- Updated add() method to correctly handle first point vs. delta points
- Fixed points() method to properly check has_base_point
- Enhanced clear() method to properly reset all state
- Removed Debug trait requirement from generic type parameter T

2. New features:
- Added multiple resampling strategies (EvenlySpaced, LargestValues, SignificantChanges)
- Implemented statistics calculation (min, max, avg, count)
- Added time range analysis functionality
- Enhanced memory optimization with delta encoding (~66% reduction)

3. Documentation updates:
- Updated terminal-ui-compression-updates.md with comprehensive details
- Enhanced testing-plan.md with test cases for new functionality
- Updated IMPLEMENTATION_PROGRESS.md with performance optimization section
- Created COMPRESSED_TIMESERIES_CHECKLIST.md to track implementation status
- Updated UI_IMPLEMENTATION_STATUS.md and TERMINAL_UI_TASKS.md

4. Archival improvements:
- Created scripts/archive-specs.ps1 for archiving outdated specifications
- Added specs/archive structure with README.md
- Archived obsolete specification files

All tests now pass with the fixed implementation, resolving critical issues with 
time series data storage and providing enhanced analysis capabilities.

State: InProgress -> Complete
Components: ui-terminal/util
Migration: None required, changes maintain backward compatibility 