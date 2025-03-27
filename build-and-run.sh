#!/usr/bin/env bash
# Script to build UI assets and run the web server with proper cleanup

# Stop any existing web server processes
echo -e "\033[33mStopping any existing web server processes...\033[0m"
pkill -f web_server || echo -e "\033[36mNo existing web server process found.\033[0m"

# Navigate to the UI web directory and build assets
echo -e "\033[33mBuilding UI assets...\033[0m"
pushd crates/ui-web > /dev/null
./build-assets.sh
BUILD_EXIT_CODE=$?
popd > /dev/null

if [ $BUILD_EXIT_CODE -ne 0 ]; then
    echo -e "\033[31mFailed to build UI assets. Exiting.\033[0m"
    exit 1
fi
echo -e "\033[32mUI assets built successfully.\033[0m"

# Navigate to the web crate directory and run the server
echo -e "\033[33mStarting web server...\033[0m"
pushd crates/web > /dev/null
cargo run --bin web_server
popd > /dev/null 