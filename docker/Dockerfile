# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency manifests
COPY code/Cargo.toml code/Cargo.lock ./
COPY code/crates ./crates

# Build dependencies (this layer will be cached)
RUN cargo build --release --workspace

# Build the applications
RUN cargo build --release --bin web_server --bin squirrel

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app squirrel

# Set working directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/web_server /usr/local/bin/
COPY --from=builder /app/target/release/squirrel /usr/local/bin/

# Copy configuration files
COPY config/ ./config/
COPY plugins/ ./plugins/

# Create necessary directories
RUN mkdir -p /app/logs /app/data && \
    chown -R squirrel:squirrel /app

# Switch to app user
USER squirrel

# Expose port
EXPOSE 8080 8081

# Set environment variables
ENV RUST_LOG=info
ENV SQUIRREL_CONFIG_PATH=/app/config/production.toml

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["web_server"] 