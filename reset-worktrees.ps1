# Reset all worktrees to origin/main
Write-Host "Resetting all worktrees to origin/main..."

$worktrees = @(
    "D:\Development\squirrel",
    "D:\Development\galaxy",
    "D:\Development\squirrel-app",
    "D:\Development\squirrel-cli",
    "D:\Development\squirrel-plugins",
    "D:\Development\squirrel-worktrees\commands",
    "D:\Development\squirrel-worktrees\context",
    "D:\Development\squirrel-worktrees\mcp",
    "D:\Development\squirrel-worktrees\monitoring",
    "D:\Development\squirrel-worktrees\reviewing",
    "D:\Development\squirrel-worktrees\web"
)

foreach ($worktree in $worktrees) {
    Write-Host "Resetting $worktree..."
    Push-Location $worktree
    git reset --hard origin/main
    Pop-Location
}

Write-Host "All worktrees have been reset to origin/main" 