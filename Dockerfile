# Multi-stage Dockerfile for Unified T27 (Frontend + Backend)
# Build Context: REPO ROOT

# Force rebuild on frontend changes - v9
ARG BUILD_TIMESTAMP=9

# --- Frontend Build Stage ---
FROM oven/bun:latest AS frontend-builder
# Force rebuild on frontend changes - v9
ARG BUILD_TIMESTAMP=9
RUN echo "Frontend build timestamp: $BUILD_TIMESTAMP"

# Install Python and build essentials for node-gyp
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y python3 make g++ && ln -s /usr/bin/python3 /usr/bin/python
WORKDIR /app

# Copy trigger file to force frontend rebuild when changed
COPY FRONTEND_REBUILD_TRIGGER.txt /tmp/trigger.txt
RUN cat /tmp/trigger.txt

# Copy entire web package (preserves directory structure)
COPY external/opencode/packages/web/ ./

# Install dependencies
RUN bun install --frozen-lockfile

# Build the app (use same-origin API calls: /auth/login instead of http://localhost:3000)
ENV VITE_API_URL=
RUN bun run build

# --- Backend Build Stage ---
FROM rust:1-slim AS backend-builder
# Install build essentials for OpenSSL and other dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Copy bootstrap files
COPY bootstrap/Cargo.toml bootstrap/Cargo.lock ./
# Need to create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --features server
# Now copy real source
COPY bootstrap/src ./src
# Force cargo to rebuild by updating the mtime of main.rs
RUN touch src/main.rs
RUN cargo build --release --features server

# --- Final Runtime Stage ---
FROM rust:1-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/t27c /usr/local/bin/t27c
RUN chmod +x /usr/local/bin/t27c

# Copy frontend assets to /app/public (served by t27c)
# Force rebuild: v8 - explicit sandbox URL
RUN echo "Frontend build v8" && ls -la /app/dist
COPY --from=frontend-builder /app/dist /app/public

# Copy additional specs and conformance data
COPY specs/ /app/specs/
COPY conformance/ /app/conformance/

EXPOSE 8080
ENV RUST_LOG=info
CMD ["t27c", "serve", "--port", "8080"]
