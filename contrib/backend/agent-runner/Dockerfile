# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1.82-bookworm AS builder

WORKDIR /app

# Cache dependency compilation before copying source.
# The empty-main trick means a fresh dep layer is only rebuilt when
# Cargo.toml / Cargo.lock changes.
# Note: Cargo.lock is generated on first build; if not present, Docker
# will generate it during this layer (acceptable for initial builds).
COPY Cargo.toml ./
# Copy Cargo.lock if it exists (optional for reproducibility)
COPY Cargo.loc[k] ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build the real binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

# ── System deps ───────────────────────────────────────────────────────────────
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

# ── Git global config ─────────────────────────────────────────────────────────
RUN git config --global user.name "T27 Agent" && \
    git config --global user.email "t27-agent@users.noreply.github.com" && \
    git config --global init.defaultBranch main && \
    git config --global advice.detachedHead false && \
    mkdir -p /root/.ssh && chmod 700 /root/.ssh && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts 2>/dev/null || true

# ── OpenCode web UI (primary, best-effort) ────────────────────────────────────
RUN curl -fsSL https://opencode.ai/install | bash || echo "[warning] opencode install skipped"

# ── code-server (fallback web IDE) ───────────────────────────────────────────
RUN curl -fsSL https://code-server.dev/install.sh | sh -s -- --method=standalone || \
    echo "[warning] code-server install skipped"

# ── PATH ──────────────────────────────────────────────────────────────────────
ENV PATH="/root/.opencode/bin:/root/.local/bin:${PATH}"

# ── Python fallback agent runner ─────────────────────────────────────────────
# Kept as a fallback script inside the image.
# The Rust binary is the real entrypoint but this provides a Python safety net.
COPY agent-runner.py /usr/local/bin/agent-runner.py
RUN chmod +x /usr/local/bin/agent-runner.py

# ── Rust binary ───────────────────────────────────────────────────────────────
COPY --from=builder /app/target/release/t27-agent-runner /usr/local/bin/t27-agent-runner
RUN chmod +x /usr/local/bin/t27-agent-runner

# ── Workspace ─────────────────────────────────────────────────────────────────
RUN mkdir -p /home/sandbox/workspace /tmp
WORKDIR /home/sandbox/workspace

# ── Health check ──────────────────────────────────────────────────────────────
# The Rust binary handles all health-check logic internally.
# Docker can still probe the HTTP endpoint directly.
HEALTHCHECK --interval=10s --timeout=3s --start-period=60s \
    CMD curl -sf "http://localhost:${PORT:-8080}/global/health" || exit 1

# ── Expose & entrypoint ───────────────────────────────────────────────────────
EXPOSE 8080

# Single binary is the entrypoint. Replaces init.sh + agent-runner.py + healthcheck.sh.
ENTRYPOINT ["t27-agent-runner"]
