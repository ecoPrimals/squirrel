#!/bin/bash

# spec_validation.sh - Script to validate specs against the codebase
# Usage: ./spec_validation.sh [team_name]
# If team_name is provided, only checks specs for that team

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPECS_DIR="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(dirname "$SPECS_DIR")"
CODE_DIR="$REPO_ROOT/code"

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if given directory exists
function check_dir_exists() {
    if [ ! -d "$1" ]; then
        echo -e "${RED}ERROR: Directory $1 does not exist${NC}"
        return 1
    fi
    return 0
}

# Check for broken links in markdown files
function check_broken_links() {
    local dir=$1
    local broken_links=0
    
    echo -e "${BLUE}Checking for broken links in $dir${NC}"
    
    # Find all markdown files
    find "$dir" -name "*.md" | while read -r file; do
        # Extract all local markdown links [text](path/to/file.md)
        grep -o -E '\[.*?\]\([^)]*\.md[^)]*\)' "$file" | grep -v "http" | while read -r link; do
            # Extract the path from the link
            path=$(echo "$link" | sed -E 's/\[.*\]\(([^)]*)\)/\1/')
            # Handle relative links
            if [[ "$path" != /* ]]; then
                path="$(dirname "$file")/$path"
            fi
            path=$(echo "$path" | sed 's/#.*//') # Remove anchor
            
            # Check if the referenced file exists
            if [ ! -f "$path" ]; then
                echo -e "${YELLOW}Broken link in $file:${NC} $link -> $path"
                ((broken_links++))
            fi
        done
    done
    
    if [ $broken_links -eq 0 ]; then
        echo -e "${GREEN}No broken links found in $dir${NC}"
    else
        echo -e "${YELLOW}Found $broken_links broken links in $dir${NC}"
    fi
}

# Check for file references in markdown that don't exist in the codebase
function check_code_references() {
    local dir=$1
    local missing_refs=0
    
    echo -e "${BLUE}Checking code references in $dir${NC}"
    
    # Find all markdown files
    find "$dir" -name "*.md" | while read -r file; do
        # Extract all code file references that look like code/path/to/file.rs or similar
        grep -o -E 'code\/[a-zA-Z0-9_\-\/\.]+\.[a-zA-Z0-9]+' "$file" | while read -r ref; do
            # Check if the referenced file exists
            if [ ! -f "$REPO_ROOT/$ref" ]; then
                echo -e "${YELLOW}Missing code reference in $file:${NC} $ref"
                ((missing_refs++))
            fi
        done
    done
    
    if [ $missing_refs -eq 0 ]; then
        echo -e "${GREEN}No missing code references found in $dir${NC}"
    else
        echo -e "${YELLOW}Found $missing_refs missing code references in $dir${NC}"
    fi
}

# Check for outdated last_updated dates (older than 30 days)
function check_outdated_dates() {
    local dir=$1
    local outdated=0
    local thirty_days_ago=$(date -d "30 days ago" +%s)
    
    echo -e "${BLUE}Checking for outdated dates in $dir${NC}"
    
    # Find all markdown files
    find "$dir" -name "*.md" | while read -r file; do
        # Extract the last_updated date if it exists
        date_line=$(grep -o -E 'last_updated: [0-9]{4}-[0-9]{2}-[0-9]{2}' "$file" | head -1)
        if [ -n "$date_line" ]; then
            date_str=$(echo "$date_line" | sed -E 's/last_updated: (.*)/\1/')
            date_seconds=$(date -d "$date_str" +%s)
            
            if [ $date_seconds -lt $thirty_days_ago ]; then
                echo -e "${YELLOW}Outdated last_updated date in $file:${NC} $date_str"
                ((outdated++))
            fi
        else
            echo -e "${YELLOW}Missing last_updated date in $file${NC}"
            ((outdated++))
        fi
    done
    
    if [ $outdated -eq 0 ]; then
        echo -e "${GREEN}No outdated dates found in $dir${NC}"
    else
        echo -e "${YELLOW}Found $outdated outdated or missing dates in $dir${NC}"
    fi
}

# Check for inconsistent implementation percentages
function check_implementation_percentages() {
    local dir=$1
    local inconsistencies=0
    
    echo -e "${BLUE}Checking implementation percentages in $dir${NC}"
    
    # First extract all percentages from SPECS.md
    specs_percentages=$(grep -o -E '[A-Za-z/]+: [0-9]{1,3}% Complete' "$SPECS_DIR/SPECS.md" | sed -E 's/([A-Za-z\/]+): ([0-9]{1,3})% Complete/\1|\2/')
    
    # Find all markdown files with status or completion percentages
    find "$dir" -name "*.md" | xargs grep -l -E 'Status: .*Complete|Completion: [0-9]{1,3}%|[0-9]{1,3}% Complete' | while read -r file; do
        component=$(basename "$(dirname "$file")")/$(basename "$file" .md)
        
        # Extract percentages from the file
        file_percentage=$(grep -o -E 'Status:.*([0-9]{1,3})% Complete|Completion: ([0-9]{1,3})%|([0-9]{1,3})% Complete' "$file" | grep -o -E '[0-9]{1,3}' | head -1)
        
        if [ -n "$file_percentage" ]; then
            # Check if this component is mentioned in SPECS.md
            specs_percentage=$(echo "$specs_percentages" | grep -E "^$component|" | cut -d'|' -f2)
            
            if [ -n "$specs_percentage" ] && [ "$file_percentage" != "$specs_percentage" ]; then
                echo -e "${YELLOW}Inconsistent percentage for $component:${NC} SPECS.md: $specs_percentage%, $file: $file_percentage%"
                ((inconsistencies++))
            fi
        fi
    done
    
    if [ $inconsistencies -eq 0 ]; then
        echo -e "${GREEN}No inconsistent percentages found in $dir${NC}"
    else
        echo -e "${YELLOW}Found $inconsistencies inconsistent percentages in $dir${NC}"
    fi
}

# Main function
function main() {
    local team_name=$1
    
    # Check if repository structure is valid
    check_dir_exists "$SPECS_DIR" || exit 1
    check_dir_exists "$CODE_DIR" || exit 1
    
    echo -e "${BLUE}Validating specs against codebase...${NC}"
    echo -e "${BLUE}Specs directory: $SPECS_DIR${NC}"
    echo -e "${BLUE}Code directory: $CODE_DIR${NC}"
    echo ""
    
    # Determine which directories to check based on team_name
    if [ -z "$team_name" ]; then
        # Check all specs
        check_broken_links "$SPECS_DIR"
        check_code_references "$SPECS_DIR"
        check_outdated_dates "$SPECS_DIR"
        check_implementation_percentages "$SPECS_DIR"
    else
        # Check specific team's specs
        case "$team_name" in
            core)
                check_broken_links "$SPECS_DIR/core"
                check_code_references "$SPECS_DIR/core"
                check_outdated_dates "$SPECS_DIR/core"
                check_implementation_percentages "$SPECS_DIR/core"
                ;;
            integration)
                check_broken_links "$SPECS_DIR/integration"
                check_code_references "$SPECS_DIR/integration"
                check_outdated_dates "$SPECS_DIR/integration"
                check_implementation_percentages "$SPECS_DIR/integration"
                ;;
            services)
                check_broken_links "$SPECS_DIR/services"
                check_code_references "$SPECS_DIR/services"
                check_outdated_dates "$SPECS_DIR/services"
                check_implementation_percentages "$SPECS_DIR/services"
                ;;
            tools)
                check_broken_links "$SPECS_DIR/tools"
                check_code_references "$SPECS_DIR/tools"
                check_outdated_dates "$SPECS_DIR/tools"
                check_implementation_percentages "$SPECS_DIR/tools"
                ;;
            ui)
                check_broken_links "$SPECS_DIR/ui"
                check_code_references "$SPECS_DIR/ui"
                check_outdated_dates "$SPECS_DIR/ui"
                check_implementation_percentages "$SPECS_DIR/ui"
                ;;
            *)
                echo -e "${RED}Invalid team name: $team_name${NC}"
                echo "Valid team names: core, integration, services, tools, ui"
                exit 1
                ;;
        esac
    fi
    
    echo ""
    echo -e "${GREEN}Specification validation complete${NC}"
}

# Call main function with arguments
main "$1" 