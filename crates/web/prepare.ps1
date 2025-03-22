# Create in-memory database for offline prepare
$env:DATABASE_URL = "sqlite::memory:"

# Prepare SQLx queries
cargo sqlx prepare

Write-Host "SQLx queries prepared successfully" 