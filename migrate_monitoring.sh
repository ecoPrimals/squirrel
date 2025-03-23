#!/bin/bash

# Script to migrate monitoring code from src/ to crates/monitoring/src/
echo "Starting migration of monitoring code..."

# Create backup
mkdir -p src_backup
cp -r src/* src_backup/

# Move files from src to crates/monitoring/src
for dir in src/*/; do
  if [ -d "$dir" ]; then
    module=$(basename "$dir")
    echo "Processing module: $module"
    mkdir -p "crates/monitoring/src/$module"
    find "$dir" -type f -exec cp {} "crates/monitoring/src/$module/" \;
  fi
done

echo "Migration complete!" 