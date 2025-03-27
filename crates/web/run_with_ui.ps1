Write-Host "Building squirrel-web with UI..." -ForegroundColor Green
cargo build --bin web_server

Write-Host "Starting server..." -ForegroundColor Green
cargo run --bin web_server

# If you want to include the API docs feature:
# cargo run --bin web_server --features api-docs 