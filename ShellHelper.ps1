function Invoke-CommandChain {
    param(
        [Parameter(Mandatory=$true)]
        [string[]]$Commands,
        
        [Parameter(Mandatory=$false)]
        [switch]$StopOnError = $true,
        
        [Parameter(Mandatory=$false)]
        [switch]$Verbose = $false
    )
    
    $result = $true
    foreach ($cmd in $Commands) {
        if ($Verbose) {
            Write-Host "Executing: $cmd" -ForegroundColor Cyan
        }
        
        # Execute the command
        Invoke-Expression $cmd
        
        # Check if command succeeded
        if ($LASTEXITCODE -ne 0) {
            if ($Verbose) {
                Write-Host "Command failed: $cmd" -ForegroundColor Red
                Write-Host "Exit code: $LASTEXITCODE" -ForegroundColor Red
            }
            
            $result = $false
            
            if ($StopOnError) {
                return $false
            }
        }
    }
    
    return $result
}

function Run-Commands {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    
    Invoke-CommandChain -Commands $Commands -Verbose
}

# This function mimics the Bash && operator
function And-Commands {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    
    Invoke-CommandChain -Commands $Commands -StopOnError -Verbose
}

# This function mimics the Bash ; operator (run all regardless of errors)
function Seq-Commands {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )
    
    Invoke-CommandChain -Commands $Commands -StopOnError:$false -Verbose
}

# Add aliases to make it more intuitive
Set-Alias -Name run -Value Run-Commands
Set-Alias -Name andrun -Value And-Commands
Set-Alias -Name seqrun -Value Seq-Commands

# Example usage:
# And-Commands "git add .", "git commit -m 'Update files'", "git push origin main"
# run "git status" "ls -la"
# andrun "mkdir test" "cd test" "touch file.txt"
# seqrun "echo 'Running first'" "invalid-command" "echo 'Still runs despite error'"

Write-Host "Shell helper functions loaded!" -ForegroundColor Green
Write-Host "Available commands: Run-Commands, And-Commands, Seq-Commands" -ForegroundColor Green
Write-Host "Available aliases: run, andrun, seqrun" -ForegroundColor Green 