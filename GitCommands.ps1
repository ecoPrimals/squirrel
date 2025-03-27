function Git-Chain {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Commands
    )

    $result = $true
    foreach ($cmd in $Commands) {
        Write-Host "Executing: $cmd" -ForegroundColor Cyan
        Invoke-Expression $cmd
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Command failed: $cmd" -ForegroundColor Red
            return $false
        }
    }
    return $true
}

function Git-Commit {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,
        
        [Parameter(Mandatory=$false)]
        [string]$Scope = "",
        
        [Parameter(Mandatory=$false)]
        [string]$Type = "feat",
        
        [Parameter(Mandatory=$false)]
        [string[]]$Files = @(".")
    )

    # Add files
    $filesArg = $Files -join " "
    git add $filesArg

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to add files" -ForegroundColor Red
        return $false
    }

    # Format commit message
    $formattedMessage = if ($Scope) {
        "$Type($Scope): $Message"
    } else {
        "$Type: $Message"
    }

    # Commit
    git commit -m "$formattedMessage"
    return ($LASTEXITCODE -eq 0)
}

function Git-Flow {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,
        
        [Parameter(Mandatory=$false)]
        [string]$Scope = "",
        
        [Parameter(Mandatory=$false)]
        [string]$Type = "feat",
        
        [Parameter(Mandatory=$false)]
        [string[]]$Files = @("."),
        
        [Parameter(Mandatory=$false)]
        [string]$Branch = ""
    )

    # Add and commit
    if (-not (Git-Commit -Message $Message -Scope $Scope -Type $Type -Files $Files)) {
        return $false
    }

    # Push if branch is specified
    if ($Branch) {
        Write-Host "Pushing to $Branch..." -ForegroundColor Cyan
        git push origin $Branch
        return ($LASTEXITCODE -eq 0)
    }

    return $true
}

# Examples of usage:
# Git-Chain "git status", "git diff"
# Git-Commit -Message "Add new feature" -Scope "ui" -Type "feat"
# Git-Flow -Message "Implement dialog component" -Scope "ui" -Branch "feature/new-dialog"

Write-Host "Git helper functions loaded!" -ForegroundColor Green
Write-Host "Available commands: Git-Chain, Git-Commit, Git-Flow" -ForegroundColor Green 