# Railway Vivado Self-Hosted Runner
# Base: Ubuntu 22.04 x86_64 (Vivado 2025.2 requirement)
# Final image size: ~150 GB (Vivado install ~110 GB + Ubuntu + runner)
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# System deps for Vivado + GH Actions runner
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl wget jq git xz-utils \
    libc6 libstdc++6 libtinfo5 libncurses5 libx11-6 libxext6 libxrender1 libxtst6 libxi6 \
    libfontconfig1 libfreetype6 libgtk2.0-0 libcanberra-gtk-module libgomp1 \
    libusb-1.0-0 libssl-dev \
    build-essential pkg-config \
    sudo locales \
    && locale-gen en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

ENV LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8

# Install Rust (for t27c build inside runner)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && ln -s /root/.cargo/bin/* /usr/local/bin/

# Vivado installer is mounted from a Railway volume at /opt/installer
# Run scripts/install_vivado.sh after first container start (or in build stage with the installer COPY'd)
RUN mkdir -p /opt/Xilinx /opt/installer /actions-runner

# GH Actions runner v2.319.1 (x64 Linux)
WORKDIR /actions-runner
RUN curl -sL https://github.com/actions/runner/releases/download/v2.319.1/actions-runner-linux-x64-2.319.1.tar.gz \
    | tar xz \
    && ./bin/installdependencies.sh

# Build Rust entrypoint (project policy: no .sh / .py — Rust only)
COPY Cargo.toml /entrypoint-src/Cargo.toml
COPY src /entrypoint-src/src
RUN cd /entrypoint-src \
    && cargo build --release --bin entrypoint \
    && cp target/release/entrypoint /usr/local/bin/runner-entrypoint \
    && rm -rf /entrypoint-src /root/.cargo/registry /root/.cargo/git

# Vivado settings sourcing
ENV PATH="/opt/Xilinx/Vivado/2025.2/bin:${PATH}"

ENTRYPOINT ["/usr/local/bin/runner-entrypoint"]
