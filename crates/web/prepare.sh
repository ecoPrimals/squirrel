#!/bin/bash

# Create in-memory database for offline prepare
export DATABASE_URL="sqlite::memory:"

# Prepare SQLx queries
cargo sqlx prepare

echo "SQLx queries prepared successfully" 