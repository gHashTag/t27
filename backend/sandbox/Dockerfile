FROM node:22-bookworm-slim

ARG OPENCODE_VERSION="latest"

# System deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash ca-certificates curl g++ git gh make openssh-client python3 \
    && rm -rf /var/lib/apt/lists/*

# Git config
RUN git config --global user.name "T27 SWE Agent" \
  && git config --global user.email "t27-agent@users.noreply.github.com"

# SSH for GitHub
RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh \
  && ssh-keyscan github.com >> /root/.ssh/known_hosts

# Install OpenCode
RUN curl -fsSL https://opencode.ai/install | bash

# Install code-server as fallback
RUN curl -fsSL https://code-server.dev/install.sh | sh -s -- --method=standalone

WORKDIR /home/sandbox/workspace

# Copy agent scripts
COPY init.sh /usr/local/bin/sandbox-init.sh
COPY agent-runner.py /usr/local/bin/agent-runner.py
COPY healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/sandbox-init.sh /usr/local/bin/healthcheck.sh

ENV PATH="/root/.opencode/bin:/root/.local/bin:${PATH}"

HEALTHCHECK --interval=10s --timeout=3s --start-period=60s \
  CMD /usr/local/bin/healthcheck.sh

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/sandbox-init.sh"]
