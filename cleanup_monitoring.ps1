# PowerShell script to clean up monitoring code from src/ after migration
Write-Host "Starting cleanup of monitoring code from src directory..."

# Check if backup exists
if (-not (Test-Path "src_backup")) {
    Write-Host "Warning: No backup found. Please make sure to run the migration script first."
    exit 1
}

# Remove monitoring directories from src
$dirsToRemove = @(
    "src\alerts",
    "src\dashboard",
    "src\metrics",
    "src\network"
)

foreach ($dir in $dirsToRemove) {
    if (Test-Path $dir) {
        Write-Host "Removing directory: $dir"
        Remove-Item -Path $dir -Recurse -Force
    } else {
        Write-Host "Directory not found: $dir (already removed)"
    }
}

# Clean up the mod.rs file in src
if (Test-Path "src\mod.rs") {
    # We'll create a placeholder that indicates the code has been moved
    Write-Host "Updating src\mod.rs with placeholder"
    Set-Content -Path "src\mod.rs" -Value "// The monitoring code has been moved to crates/monitoring/src
// This file is kept for backward compatibility but all functionality is in the crates directory.
"
}

Write-Host "Cleanup complete!"
Write-Host ""
Write-Host "NOTE: A backup of the original files has been kept in the src_backup directory."
Write-Host "If you need to restore any files, they are available there." 