# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1.82-bookworm AS builder

WORKDIR /app

# Cache dependencies first (empty main trick for layer caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build the real binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    gh \
    python3 \
    python3-pip \
    openssh-client \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Configure git identity for commits
RUN git config --global user.name "T27 Agent" && \
    git config --global user.email "t27-agent@users.noreply.github.com" && \
    git config --global init.defaultBranch main && \
    git config --global advice.detachedHead false

# Copy compiled binary
COPY --from=builder /app/target/release/t27-agent-runner /usr/local/bin/t27-agent-runner
RUN chmod +x /usr/local/bin/t27-agent-runner

# Install OpenCode for web UI observation (best-effort, non-fatal)
RUN curl -fsSL https://opencode.ai/install | bash || echo "opencode install skipped"

# Add opencode to PATH if installed
ENV PATH="/root/.opencode/bin:/root/.local/bin:${PATH}"

# Create workspace directory
RUN mkdir -p /workspace /tmp

# Set working directory
WORKDIR /workspace

# Expose port for OpenCode web UI
EXPOSE 8080

ENTRYPOINT ["t27-agent-runner"]
