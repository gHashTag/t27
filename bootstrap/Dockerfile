# Multi-stage Dockerfile for t27c compiler
# NOTE: Docker build context is the REPO ROOT (Railway sets dockerfilePath = "bootstrap/Dockerfile")
FROM rust:1.82-slim as builder

WORKDIR /app

# Copy manifest and lock for reproducible builds
COPY bootstrap/Cargo.toml bootstrap/Cargo.lock ./

# Copy source code from bootstrap directory
COPY bootstrap/src ./src

# Build with server feature enabled
RUN cargo build --release --features server

# Runtime stage - minimal Debian
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /app/target/release/t27c /t27c

# Copy specs and conformance data for the API to serve
COPY specs/ /specs/
COPY conformance/ /conformance/

# Expose port for HTTP server
EXPOSE 8080

# Set environment variable for Railway
ENV RUST_LOG=info

# Railway will inject PORT environment variable
CMD ["sh", "-c", "/t27c serve --port $PORT"]
