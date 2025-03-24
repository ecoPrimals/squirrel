# Script to update all worktrees by pulling from main

Write-Host "Updating all worktrees with latest changes from main..." -ForegroundColor Cyan

# First update main branch itself
Write-Host "`nUpdating main branch..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel"
git pull origin main

# Update galaxy worktree
Write-Host "`nUpdating galaxy worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\galaxy"
git checkout galaxy
git pull origin main

# Update app worktree
Write-Host "`nUpdating app worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-app"
git checkout app
git pull origin main

# Update cli worktree
Write-Host "`nUpdating cli worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-cli"
git checkout cli
git pull origin main

# Update plugins worktree
Write-Host "`nUpdating plugins worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-plugins"
git checkout plugins
git pull origin main

# Update commands worktree
Write-Host "`nUpdating commands worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-worktrees\commands"
git checkout commands
git pull origin main

# Update context worktree
Write-Host "`nUpdating context worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-worktrees\context"
git checkout context
git pull origin main

# Update mcp worktree
Write-Host "`nUpdating mcp worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-worktrees\mcp"
git checkout mcp
git pull origin main

# Update monitoring worktree
Write-Host "`nUpdating monitoring worktree..." -ForegroundColor Green
Set-Location -Path "D:\Development\squirrel-worktrees\monitoring"
git checkout monitoring
git pull origin main

# Return to the main repo
Set-Location -Path "D:\Development\squirrel"
Write-Host "`nAll worktrees have been updated successfully!" -ForegroundColor Cyan 