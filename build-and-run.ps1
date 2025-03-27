#!/usr/bin/env pwsh
# Script to build UI assets and run the web server with proper cleanup

# Stop any existing web server processes
Write-Host "Stopping any existing web server processes..." -ForegroundColor Yellow
taskkill /f /im web_server.exe 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "Existing web server process terminated." -ForegroundColor Green
} else {
    Write-Host "No existing web server process found." -ForegroundColor Cyan
}

# Navigate to the UI web directory and build assets
Write-Host "Building UI assets..." -ForegroundColor Yellow
Push-Location "crates/ui-web"
./build-assets.ps1
$buildExitCode = $LASTEXITCODE
Pop-Location

if ($buildExitCode -ne 0) {
    Write-Host "Failed to build UI assets. Exiting." -ForegroundColor Red
    exit 1
}
Write-Host "UI assets built successfully." -ForegroundColor Green

# Navigate to the web crate directory and run the server
Write-Host "Starting web server..." -ForegroundColor Yellow
Push-Location "crates/web"
cargo run --bin web_server
Pop-Location 