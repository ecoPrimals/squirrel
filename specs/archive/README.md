# Specifications Archive

This directory contains archived specification files that are no longer actively used in the project but are preserved for historical reference.

## Directory Structure

Files are organized in date-stamped directories (YYYY-MM-DD format) that indicate when they were archived. This allows for easy tracking of when specifications were superseded by newer versions.

```
archive/
├── README.md
├── 2024-07-30/
│   ├── ratatui-upgrade-guide.md
│   ├── protocol-widget-upgrade-example.md
│   └── ...
├── 2024-08-15/
│   ├── ...
└── ...
```

## Archival Process

Files are moved to this directory when:
1. They have been fully implemented and are no longer needed for active development
2. They have been superseded by newer specification documents
3. They contain outdated information that is no longer relevant but may have historical value

## How to Archive Files

Use the `scripts/archive-specs.ps1` script to archive files:

```powershell
.\scripts\archive-specs.ps1 specs/ui/file-to-archive.md specs/ui/another-file.md
```

This script will:
1. Create a date-stamped directory in the archive folder if it doesn't exist
2. Move the specified files to that directory
3. Log the archival operation

## Accessing Archived Specifications

To reference archived specifications:
1. Navigate to the specs/archive directory
2. Look for the appropriate date directory
3. Open the file you need to reference

## Note on Archived Content

While the information in these files might be outdated, they can be valuable for:
- Understanding historical design decisions
- Tracing the evolution of features
- Providing context for why certain implementation choices were made

For current specifications, please refer to the active documents in the main specs directory. 