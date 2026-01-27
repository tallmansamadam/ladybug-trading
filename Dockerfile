# Dockerfile for LadyBug Trading Engine Backend

FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY rust-engine/Cargo.toml rust-engine/Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY rust-engine/src ./src

# Build the application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install SSL certificates and ca-certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/ladybug-engine /app/ladybug-engine

# Expose the API port
EXPOSE 8080

# Set environment variables (will be overridden by docker-compose)
ENV RUST_LOG=info

# Run the application
CMD ["/app/ladybug-engine"]
