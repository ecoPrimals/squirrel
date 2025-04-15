#!/bin/bash
set -e

# Install required Python packages for the task client
echo "Installing required Python packages..."

# Check if pip is available
if ! command -v pip &> /dev/null; then
    echo "pip could not be found. Please install Python and pip first."
    exit 1
fi

# Install required packages
pip install grpcio grpcio-tools

echo "Python dependencies installed successfully." 