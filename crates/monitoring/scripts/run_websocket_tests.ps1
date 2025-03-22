# Enhanced WebSocket Test Runner
# This script runs the WebSocket test and generates a performance report

# Configuration parameters
param (
    [int]$NumClients = 20,
    [int]$TestDurationSeconds = 180,
    [string]$ReportPath = "./websocket_test_report.md",
    [switch]$GenerateChart = $true,
    [switch]$VerboseOutput = $false
)

# Display test configuration
Write-Host "=================================="
Write-Host "WebSocket Test Configuration"
Write-Host "=================================="
Write-Host "Number of Clients: $NumClients"
Write-Host "Test Duration: $TestDurationSeconds seconds"
Write-Host "Report Path: $ReportPath"
Write-Host "Generate Chart: $GenerateChart"
Write-Host "Verbose Output: $VerboseOutput"
Write-Host "=================================="

# Check if server is running
$serverRunning = $false
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    # Attempt to connect to the WebSocket server
    $tcpClient.Connect("localhost", 8765)
    $serverRunning = $true
    $tcpClient.Close()
    Write-Host "✅ WebSocket server is running on port 8765"
} catch {
    Write-Host "❌ WebSocket server is not running on port 8765"
    
    # Ask if user wants to start server
    $startServer = Read-Host "Do you want to start the WebSocket server? (y/n)"
    if ($startServer -eq "y") {
        Write-Host "Starting WebSocket server in a new window..."
        Start-Process -FilePath "cargo" -ArgumentList "run --example dashboard_server" -NoNewWindow
        
        Write-Host "Waiting for server to start..."
        Start-Sleep -Seconds 5
    } else {
        Write-Host "Exiting..."
        exit
    }
}

# Create output directory if it doesn't exist
$outputDir = Split-Path -Parent $ReportPath
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

# Get timestamp for the report
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

# Run the WebSocket test with specified parameters
Write-Host "Starting WebSocket test with $NumClients clients for $TestDurationSeconds seconds..."
$testOutput = & cargo run --example enhanced_websocket_test -- --num-clients $NumClients --duration $TestDurationSeconds 2>&1

# Save raw output to file
$rawOutputPath = $ReportPath -replace "\.md$", "_raw.txt"
$testOutput | Out-File -FilePath $rawOutputPath

# Extract test statistics
$stats = @{
    ConnectionsSuccessful = 0
    ConnectionFailures = 0
    ConnectionsClosed = 0
    TotalMessagesReceived = 0
    ReconnectionTests = 0
    MessagesPerComponent = @{}
    ClientStats = @{}
}

$inStats = $false
$inMessagesPerComponent = $false
$inClientStats = $false

foreach ($line in $testOutput) {
    # Extract relevant statistics
    if ($line -match "Connections successful: (\d+)") {
        $stats.ConnectionsSuccessful = [int]$Matches[1]
    }
    elseif ($line -match "Connection failures: (\d+)") {
        $stats.ConnectionFailures = [int]$Matches[1]
    }
    elseif ($line -match "Connections closed: (\d+)") {
        $stats.ConnectionsClosed = [int]$Matches[1]
    }
    elseif ($line -match "Total messages received: (\d+)") {
        $stats.TotalMessagesReceived = [int]$Matches[1]
    }
    elseif ($line -match "Reconnection tests: (\d+)") {
        $stats.ReconnectionTests = [int]$Matches[1]
    }
    
    # Track message sections
    if ($line -match "--- Messages Per Component ---") {
        $inMessagesPerComponent = $true
        $inClientStats = $false
        continue
    }
    elseif ($line -match "--- Client Statistics ---") {
        $inMessagesPerComponent = $false
        $inClientStats = $true
        continue
    }
    elseif ($line -match "============================") {
        $inMessagesPerComponent = $false
        $inClientStats = $false
        continue
    }
    
    # Parse messages per component
    if ($inMessagesPerComponent -and $line -match "([^:]+): (\d+)") {
        $componentName = $Matches[1].Trim()
        $messageCount = [int]$Matches[2]
        $stats.MessagesPerComponent[$componentName] = $messageCount
    }
    
    # Parse client statistics
    if ($inClientStats -and $line -match "client-(\d+): (\d+) attempts, (\d+) messages, (\d+) subscriptions") {
        $clientId = [int]$Matches[1]
        $attempts = [int]$Matches[2]
        $messages = [int]$Matches[3]
        $subscriptions = [int]$Matches[4]
        
        $stats.ClientStats["client-$clientId"] = @{
            ConnectionAttempts = $attempts
            MessagesReceived = $messages
            ComponentsSubscribed = $subscriptions
        }
    }
}

# Generate Markdown report
$report = @"
# WebSocket Test Report

## Test Configuration
- **Date**: $timestamp
- **Number of Clients**: $NumClients
- **Test Duration**: $TestDurationSeconds seconds
- **Server Address**: ws://localhost:8765/ws

## Test Results Summary

| Metric | Value |
|--------|-------|
| Connections Successful | $($stats.ConnectionsSuccessful) |
| Connection Failures | $($stats.ConnectionFailures) |
| Connections Closed | $($stats.ConnectionsClosed) |
| Total Messages Received | $($stats.TotalMessagesReceived) |
| Reconnection Tests | $($stats.ReconnectionTests) |

## Messages Per Component

| Component | Message Count |
|-----------|--------------|
"@

# Add message per component stats
foreach ($component in $stats.MessagesPerComponent.Keys | Sort-Object) {
    $count = $stats.MessagesPerComponent[$component]
    $report += "| $component | $count |`n"
}

$report += @"

## Client Performance

| Client | Connection Attempts | Messages Received | Components Subscribed |
|--------|---------------------|-------------------|----------------------|
"@

# Add client stats
foreach ($client in $stats.ClientStats.Keys | Sort-Object) {
    $clientStat = $stats.ClientStats[$client]
    $report += "| $client | $($clientStat.ConnectionAttempts) | $($clientStat.MessagesReceived) | $($clientStat.ComponentsSubscribed) |`n"
}

# Calculate performance metrics
$avgMessagesPerClient = if ($stats.ClientStats.Count -gt 0) {
    [math]::Round($stats.TotalMessagesReceived / $stats.ClientStats.Count, 2)
} else {
    0
}

$connectionSuccessRate = if (($stats.ConnectionsSuccessful + $stats.ConnectionFailures) -gt 0) {
    [math]::Round(($stats.ConnectionsSuccessful / ($stats.ConnectionsSuccessful + $stats.ConnectionFailures)) * 100, 2)
} else {
    0
}

$messagesPerSecond = [math]::Round($stats.TotalMessagesReceived / $TestDurationSeconds, 2)

$report += @"

## Performance Analysis

| Metric | Value |
|--------|-------|
| Messages Per Second | $messagesPerSecond |
| Average Messages Per Client | $avgMessagesPerClient |
| Connection Success Rate | $connectionSuccessRate% |

## Reconnection Performance

The test included $($stats.ReconnectionTests) simulated disconnections to test the reconnection logic.
The connection success rate of $connectionSuccessRate% indicates the reliability of the reconnection mechanism.

## Conclusion

"@

# Add conclusion based on results
if ($messagesPerSecond -gt 50 -and $connectionSuccessRate -gt 95) {
    $report += "The WebSocket server performed **excellently** under load, handling multiple concurrent clients with high message throughput and reliable reconnections."
} elseif ($messagesPerSecond -gt 20 -and $connectionSuccessRate -gt 85) {
    $report += "The WebSocket server performed **well** under load, with good message throughput and mostly reliable reconnections."
} else {
    $report += "The WebSocket server performance **needs improvement**, particularly in handling message throughput and connection reliability."
}

$report += @"

## Raw Test Output

See the complete test output in the file: $rawOutputPath

"@

if ($GenerateChart) {
    # Add placeholder for chart
    $report += @"

## Performance Charts

### Messages Per Component
```mermaid
pie title Messages Per Component
"@

    foreach ($component in $stats.MessagesPerComponent.Keys | Sort-Object) {
        $count = $stats.MessagesPerComponent[$component]
        $report += "`n    `"$component`" : $count"
    }

    $report += @"
```

### Client Message Distribution
```mermaid
bar
    title Client Message Distribution
    x-axis [Clients]
    y-axis [Message Count]
"@

    foreach ($client in $stats.ClientStats.Keys | Sort-Object) {
        $messages = $stats.ClientStats[$client].MessagesReceived
        $report += "`n    `"$client`" : $messages"
    }

    $report += @"
```
"@
}

# Write report to file
$report | Out-File -FilePath $ReportPath

Write-Host "=================================="
Write-Host "Test completed!"
Write-Host "Report saved to: $ReportPath"
Write-Host "Raw output saved to: $rawOutputPath"
Write-Host "=================================="

# Open the report if requested
if ($VerboseOutput) {
    Write-Host "Opening report..."
    Invoke-Item $ReportPath
} 