# Setup script for SQLite database for development
# This script creates a SQLite database and runs migrations

# Create the database
sqlx database create --database-url sqlite:test.db

# Run migrations 
sqlx migrate run --database-url sqlite:test.db --source migrations

Write-Host "Database setup complete!" 