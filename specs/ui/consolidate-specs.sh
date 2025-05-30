#!/bin/bash

# UI Specification Consolidation Script
# Version: 1.0.0
# Date: 2024-08-30

# Terminal colors for better readability
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== UI Specification Consolidation ===${NC}"
echo -e "${BLUE}Starting consolidation at $(date) ===${NC}"
echo ""

# Create directory structure if it doesn't exist
echo -e "${BLUE}Creating directory structure...${NC}"
mkdir -p core testing implementation archived
echo -e "${GREEN}Directory structure created.${NC}"
echo ""

# Move testing files to the testing directory
echo -e "${BLUE}Moving testing documentation...${NC}"
mv -v TESTING_STATUS.md testing/
mv -v testing-strategy.md testing/testing-strategy-original.md
echo -e "${GREEN}Testing files moved. New consolidated testing files have been created in the testing/ directory.${NC}"
echo ""

# Move files to be archived to the archived directory
echo -e "${BLUE}Moving files to archive...${NC}"

# Function to archive a file with ARCHIVED_ prefix
archive_file() {
    if [ -f "$1" ]; then
        target="archived/ARCHIVED_$(basename $1)"
        cp -v "$1" "$target"
        echo "Archived: $1 -> $target"
    else
        echo -e "${YELLOW}Warning: File $1 not found for archiving${NC}"
    fi
}

# Archive testing related files
archive_file "VITEST_TO_JEST_MIGRATION.md"
archive_file "VITEST_TO_JEST_SPRINT_READINESS.md"
archive_file "VITEST_TO_JEST_MIGRATION_SUMMARY.md"
archive_file "TESTING_FRAMEWORK_MIGRATION_SUMMARY.md"
archive_file "TESTING_CONSOLIDATION.md"
archive_file "testing-strategy-update.md"

# Archive implementation related files
archive_file "implementation-progress-update.md"
archive_file "unified-ui-integration.md"
archive_file "react-component-specs.md"
archive_file "react-implementation.md" 
archive_file "WEB_CONSOLIDATION.md"
archive_file "WEB_DEPRECATION_STEPS.md"
archive_file "MIGRATION_PLAN_WEB_TO_TAURI.md"

# Archive TUI related files if not actively maintained
archive_file "terminal-ui-strategy.md"
archive_file "tui-component-specs.md"
archive_file "TERMINAL_UI_SUMMARY.md"
archive_file "TERMINAL_UI_TASKS.md"

# Archive other files
archive_file "05-dashboard.md"
archive_file "dashboard_integration.md"
archive_file "data_integration_plan.md"
archive_file "DEMO_SYSTEM.md"

echo -e "${GREEN}Files archived.${NC}"
echo ""

# Process implementation plan into separate documents
echo -e "${BLUE}Processing implementation-plan-performance-plugin.md...${NC}"
if [ -f "implementation-plan-performance-plugin.md" ]; then
    echo "Creating initial implementation files..."
    # Create stub files - these would need to be manually edited later
    echo "# Performance Monitoring Implementation Guide" > implementation/PERFORMANCE_MONITORING.md
    echo "# Plugin Management Implementation Guide" > implementation/PLUGIN_MANAGEMENT.md
    
    # Move original file to reference
    cp -v "implementation-plan-performance-plugin.md" "implementation/implementation-plan-original.md"
    echo -e "${GREEN}Implementation plan processed. The new files in the implementation/ directory need manual editing to complete the split.${NC}"
else
    echo -e "${YELLOW}Warning: implementation-plan-performance-plugin.md not found${NC}"
fi
echo ""

# Process web bridge implementation
echo -e "${BLUE}Processing web bridge files...${NC}"
if [ -f "web_bridge_implementation.md" ]; then
    echo "Creating WEB_BRIDGE.md..."
    cp -v "web_bridge_implementation.md" "implementation/WEB_BRIDGE.md"
    echo -e "${GREEN}Web bridge implementation processed.${NC}"
else
    echo -e "${YELLOW}Warning: web_bridge_implementation.md not found${NC}"
fi
echo ""

# Process core architecture files
echo -e "${BLUE}Processing architecture files...${NC}"
if [ -f "tauri-react-architecture.md" ]; then
    echo "Creating UI_ARCHITECTURE.md..."
    cp -v "tauri-react-architecture.md" "core/UI_ARCHITECTURE.md"
    echo -e "${GREEN}Architecture documentation processed.${NC}"
else
    echo -e "${YELLOW}Warning: tauri-react-architecture.md not found${NC}"
fi
echo ""

# Update UI_DEVELOPMENT_STATUS.md
echo -e "${BLUE}Processing UI_DEVELOPMENT_STATUS.md...${NC}"
if [ -f "UI_DEVELOPMENT_STATUS.md" ]; then
    echo "Moving UI_DEVELOPMENT_STATUS.md to core/..."
    cp -v "UI_DEVELOPMENT_STATUS.md" "core/UI_DEVELOPMENT_STATUS.md"
    echo -e "${GREEN}UI development status processed.${NC}"
else
    echo -e "${YELLOW}Warning: UI_DEVELOPMENT_STATUS.md not found${NC}"
fi
echo ""

echo -e "${BLUE}=== Consolidation Summary ===${NC}"
echo "The following directories were created:"
echo "- specs/ui/core/ - Core architecture and status documentation"
echo "- specs/ui/testing/ - Testing strategy, status, and patterns"
echo "- specs/ui/implementation/ - Feature implementation guides"
echo "- specs/ui/archived/ - Archived documents for reference"
echo ""
echo "The following files were created or moved:"
ls -la core/ testing/ implementation/ | grep -v "^total" | grep -v "^\."
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Review all the new files and make sure they contain the correct content"
echo "2. Manually edit the split implementation files to ensure they contain the right sections"
echo "3. Update any cross-references between files"
echo "4. Remove the original files once you've verified the new structure is complete"
echo ""
echo -e "${BLUE}=== Consolidation completed at $(date) ===${NC}" 