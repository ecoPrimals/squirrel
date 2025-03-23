# PowerShell script to migrate monitoring code from src/ to crates/monitoring/src/
Write-Host "Starting migration of monitoring code..."

# Create backup
if (-not (Test-Path "src_backup")) {
    New-Item -Path "src_backup" -ItemType Directory | Out-Null
}
Copy-Item -Path "src\*" -Destination "src_backup" -Recurse -Force

# Move files from src to crates/monitoring/src
$dirs = Get-ChildItem -Path "src" -Directory
foreach ($dir in $dirs) {
    $module = $dir.Name
    Write-Host "Processing module: $module"
    
    # Ensure directory exists
    $targetDir = "crates\monitoring\src\$module"
    if (-not (Test-Path $targetDir)) {
        New-Item -Path $targetDir -ItemType Directory | Out-Null
    }
    
    # Copy all files
    $files = Get-ChildItem -Path $dir.FullName -File
    foreach ($file in $files) {
        Copy-Item -Path $file.FullName -Destination "$targetDir\" -Force
    }
}

Write-Host "Migration complete!" 