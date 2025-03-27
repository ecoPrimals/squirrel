#!/usr/bin/env bash
# Script to build the UI assets and prepare for distribution

# Create the dist directory if it doesn't exist
mkdir -p dist

# Copy all web files to the dist directory
echo "Copying web files to dist directory..."
cp -r web/* dist/

# Additional processing can be added here if needed
# For example: minification, bundling, etc.

echo "UI assets prepared in dist directory successfully!"
exit 0 # Explicitly return success exit code 