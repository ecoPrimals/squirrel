# Run the Phase 1 Functional Demo
Write-Host "`n[DEMO] Running Phase 1 Functional Demo`n" -ForegroundColor Cyan

# Check for processes that might be locking our files
Write-Host "[SETUP] Checking for processes that might be locking our files..." -ForegroundColor Yellow
$exesToCheck = @(
    "phase1_functional_demo.exe"
)

foreach ($exe in $exesToCheck) {
    $processes = Get-Process | Where-Object { $_.Path -like "*$exe*" } -ErrorAction SilentlyContinue
    if ($processes) {
        Write-Host "  - Found running process for $exe, stopping..." -ForegroundColor Yellow
        $processes | ForEach-Object { 
            Stop-Process -Id $_.Id -Force 
            Write-Host "    Stopped process $($_.Id)" -ForegroundColor Green
        }
    }
}

# Clean the specific example
Write-Host "`n[CLEAN] Cleaning Phase 1 demo artifacts...`n" -ForegroundColor Cyan
$exePath = "target/debug/examples/phase1_functional_demo.exe"
if (Test-Path $exePath) {
    try {
        Remove-Item $exePath -Force -ErrorAction Stop
        Write-Host "  - Removed existing executable" -ForegroundColor Green
    } catch {
        Write-Host "  - Warning: Could not remove existing executable: $_" -ForegroundColor Yellow
    }
}

# Build the example
Write-Host "`n[BUILD] Building Phase 1 Functional Demo...`n" -ForegroundColor Cyan
cargo build --example phase1_functional_demo
if (-not $?) {
    Write-Host "`n[ERROR] Build failed!`n" -ForegroundColor Red
    exit 1
}

# Run the demo
Write-Host "`n[RUN] Running Phase 1 Functional Demo...`n" -ForegroundColor Cyan
cargo run --example phase1_functional_demo

# Check the result
if (-not $?) {
    Write-Host "`n[ERROR] Demo failed!`n" -ForegroundColor Red
    exit 1
} else {
    Write-Host "`n[SUCCESS] Phase 1 Functional Demo completed successfully!`n" -ForegroundColor Green
}

Write-Host "This demo showcases:" -ForegroundColor Yellow
Write-Host "1. Command Transaction System: Integration with command execution" -ForegroundColor Yellow
Write-Host "2. Command Journaling: Recording command execution history" -ForegroundColor Yellow
Write-Host "3. Resource Monitoring: Tracking resource usage during command execution" -ForegroundColor Yellow
Write-Host "4. Enhanced Observability: Structured logging of command execution" -ForegroundColor Yellow
Write-Host "`nAll Phase 1 Enhancements are fully implemented and demonstrated.`n" -ForegroundColor Green 