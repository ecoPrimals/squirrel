function Invoke-WslCommand {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    
    $commandText = $Commands -join " && "
    Write-Host "Running in WSL: $commandText" -ForegroundColor Cyan
    
    # Execute in WSL
    wsl bash -c "$commandText"
    return $LASTEXITCODE
}

function Invoke-GitBashCommand {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    
    $commandText = $Commands -join " && "
    Write-Host "Running in Git Bash: $commandText" -ForegroundColor Cyan
    
    # Check if Git Bash exists in common locations
    $gitBashPath = "C:\Program Files\Git\bin\bash.exe"
    if (-not (Test-Path $gitBashPath)) {
        $gitBashPath = "C:\Program Files (x86)\Git\bin\bash.exe"
    }
    
    if (Test-Path $gitBashPath) {
        & $gitBashPath -c "$commandText"
        return $LASTEXITCODE
    } else {
        Write-Host "Git Bash not found. Please install Git for Windows or specify the correct path." -ForegroundColor Red
        return 1
    }
}

# Create a WSL-aware git command
function Git-Wsl {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    
    $gitCommands = $Commands | ForEach-Object { "git $_" }
    Invoke-WslCommand $gitCommands
}

# Add convenient aliases
Set-Alias -Name wsl-run -Value Invoke-WslCommand
Set-Alias -Name bash-run -Value Invoke-GitBashCommand
Set-Alias -Name wgit -Value Git-Wsl

# Examples:
# wsl-run "ls -la" "echo 'Hello from WSL'"
# bash-run "git status" "git add ." "git commit -m 'Update files'"
# wgit "add ." "commit -m 'Update from WSL'" "push origin main"

Write-Host "WSL Bridge loaded!" -ForegroundColor Green
Write-Host "Available commands: Invoke-WslCommand, Invoke-GitBashCommand, Git-Wsl" -ForegroundColor Green
Write-Host "Available aliases: wsl-run, bash-run, wgit" -ForegroundColor Green 