#!/usr/bin/env pwsh
# Script to run Address Sanitizer checks on binaries without fuzzing

param(
    [Parameter(Mandatory=$false)]
    [string]$BinaryPath,

    [Parameter(Mandatory=$false)]
    [string]$TestArgs = "",

    [Parameter(Mandatory=$false)]
    [int]$Iterations = 1,

    [Parameter(Mandatory=$false)]
    [switch]$Verbose = $false
)

# Show usage if no binary path provided
if ([string]::IsNullOrEmpty($BinaryPath)) {
    Write-Host "ASAN Binary Checker for Squirrel Plugin System"
    Write-Host "==============================================="
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  ./tools/run_asan_check.ps1 -BinaryPath <path_to_binary> [-TestArgs '<args>'] [-Iterations <num>] [-Verbose]"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -BinaryPath   : Path to the binary executable to test"
    Write-Host "  -TestArgs     : Arguments to pass to the binary (optional)"
    Write-Host "  -Iterations   : Number of times to run the binary (default: 1)"
    Write-Host "  -Verbose      : Show detailed output"
    Write-Host ""
    Write-Host "Example:"
    Write-Host "  ./tools/run_asan_check.ps1 -BinaryPath './target/debug/plugin_host' -TestArgs '--load-plugin ./plugins/test.dll' -Iterations 5"
    exit 0
}

# Check if the binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Host "Error: Binary not found at path: $BinaryPath" -ForegroundColor Red
    exit 1
}

# Check for LLVM installation and set up ASAN environment
$llvmPath = "C:\Program Files\LLVM\bin"
if (-not (Test-Path "$llvmPath\llvm-symbolizer.exe")) {
    Write-Host "Warning: LLVM not found at expected location ($llvmPath)" -ForegroundColor Yellow
    Write-Host "Address sanitizer reports may not show symbolized stack traces." -ForegroundColor Yellow
    Write-Host "See docs/devtools/address_sanitizer_guide.md for setup instructions." -ForegroundColor Yellow
} else {
    $env:PATH += ";$llvmPath"
    $env:ASAN_SYMBOLIZER_PATH = "$llvmPath\llvm-symbolizer.exe"
}

# Set ASAN options
$env:ASAN_OPTIONS = "symbolize=1:detect_leaks=0:abort_on_error=1:print_stacktrace=1"

# Check if running with Rust nightly
$rustVersion = rustc --version
if (-not $rustVersion -like "*nightly*") {
    Write-Host "Warning: You are not using Rust nightly, which is required for ASAN." -ForegroundColor Yellow
    Write-Host "Run 'rustup default nightly' to switch to nightly." -ForegroundColor Yellow
}

# Helper function to run a command with ASAN
function Run-WithASAN {
    param (
        [string]$Command,
        [string]$Arguments,
        [int]$Iteration
    )
    
    Write-Host "Running iteration $Iteration with ASAN..." -ForegroundColor Cyan
    if ($Verbose) {
        Write-Host "Command: $Command $Arguments" -ForegroundColor Gray
        Write-Host "ASAN_OPTIONS: $env:ASAN_OPTIONS" -ForegroundColor Gray
    }
    
    try {
        # Use Start-Process to capture output and exit code
        $pinfo = New-Object System.Diagnostics.ProcessStartInfo
        $pinfo.FileName = $Command
        $pinfo.Arguments = $Arguments
        $pinfo.RedirectStandardError = $true
        $pinfo.RedirectStandardOutput = $true
        $pinfo.UseShellExecute = $false
        
        $p = New-Object System.Diagnostics.Process
        $p.StartInfo = $pinfo
        $p.Start() | Out-Null
        
        $stdout = $p.StandardOutput.ReadToEnd()
        $stderr = $p.StandardError.ReadToEnd()
        $p.WaitForExit()
        
        if ($p.ExitCode -ne 0) {
            Write-Host "ASAN detected issues (Exit code: $($p.ExitCode)):" -ForegroundColor Red
            
            # Check if stderr contains ASAN reports
            if ($stderr -match "AddressSanitizer") {
                Write-Host $stderr -ForegroundColor Red
            } else {
                Write-Host "Process failed but no ASAN report found in output." -ForegroundColor Yellow
                if ($Verbose) {
                    Write-Host "Standard Output:" -ForegroundColor Gray
                    Write-Host $stdout
                    Write-Host "Standard Error:" -ForegroundColor Gray
                    Write-Host $stderr
                }
            }
            return $false
        } else {
            if ($Verbose) {
                Write-Host "Standard Output:" -ForegroundColor Gray
                Write-Host $stdout
                Write-Host "Standard Error:" -ForegroundColor Gray
                Write-Host $stderr
            }
            return $true
        }
    } catch {
        Write-Host "Error running process: $_" -ForegroundColor Red
        return $false
    }
}

# Main execution
Write-Host "Starting ASAN checks for: $BinaryPath" -ForegroundColor Green
Write-Host "Running $Iterations iteration(s)..." -ForegroundColor Green

$successCount = 0
$failureCount = 0

for ($i = 1; $i -le $Iterations; $i++) {
    $result = Run-WithASAN -Command $BinaryPath -Arguments $TestArgs -Iteration $i
    
    if ($result) {
        $successCount++
        Write-Host "✓ Iteration $i completed successfully (no ASAN issues detected)" -ForegroundColor Green
    } else {
        $failureCount++
        Write-Host "✗ Iteration $i failed (ASAN issues detected)" -ForegroundColor Red
    }
    
    # Add a separator between iterations
    if ($i -lt $Iterations) {
        Write-Host "-----------------------------------------" -ForegroundColor Gray
    }
}

# Summary
Write-Host ""
Write-Host "ASAN Check Summary:" -ForegroundColor Cyan
Write-Host "  Total Iterations: $Iterations" -ForegroundColor Cyan
Write-Host "  Successful: $successCount" -ForegroundColor $(if ($successCount -eq $Iterations) { "Green" } else { "Cyan" })
Write-Host "  Failed: $failureCount" -ForegroundColor $(if ($failureCount -gt 0) { "Red" } else { "Cyan" })

# Return appropriate exit code
if ($failureCount -gt 0) {
    exit 1
} else {
    exit 0
} 