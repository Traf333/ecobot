# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies only - this layer will be cached unless Cargo.toml/Cargo.lock changes
RUN cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY src ./src

# Build the application in release mode
# This will only rebuild your code, not dependencies
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/ecobot /app/ecobot

# Create log directory for persistent logs
RUN mkdir -p /var/log/ecobot

# Create a non-root user and give ownership of app and log dirs
RUN useradd -m -u 1000 botuser && \
    chown -R botuser:botuser /app /var/log/ecobot

USER botuser

# Run the binary
CMD ["/app/ecobot"]
