# Multi-stage Dockerfile for t27c compiler
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy source code (using standard src/ structure)
COPY Cargo.toml .
COPY src ./src

# Build with server feature enabled
RUN cargo build --release --features server

# Runtime stage - minimal Debian
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /app/target/release/t27c /t27c

# Expose port for HTTP server
EXPOSE 8080

# Set environment variable for Railway
ENV RUST_LOG=info

# Railway will inject PORT environment variable
CMD ["sh", "-c", "/t27c serve --port $PORT"]
