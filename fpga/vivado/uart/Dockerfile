# Dockerfile — self-hosted GitHub Actions runner base image
#
# Does NOT install Vivado.  Vivado is expected to be bind-mounted from the
# host at /tools/Xilinx (or /opt/Xilinx) at container start, matching a
# typical self-hosted setup where the installer was run once on bare metal.
#
# Build:
#   docker build -t ghactions-vivado-runner .
#
# Run example (Vivado on host at /opt/Xilinx):
#   docker run --rm \
#     -v /opt/Xilinx:/opt/Xilinx:ro \
#     -e XILINX_VIVADO=/opt/Xilinx/Vivado/2023.2 \
#     -e RUNNER_TOKEN=<token> \
#     ghactions-vivado-runner
#
# The RUNNER_TOKEN must be obtained fresh from:
#   https://github.com/<owner>/<repo>/settings/actions/runners/new

FROM ubuntu:22.04

LABEL org.opencontainers.image.description="GitHub Actions self-hosted runner base for Vivado (Vivado volume-mounted from host)"

ENV DEBIAN_FRONTEND=noninteractive

# Runtime dependencies required by Vivado on Ubuntu 22.04.
# libtinfo5 is not in 22.04 main repos; libncurses5 pulls it transitively
# via the compat package.  Install both to be safe.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        jq \
        libc6 \
        libglib2.0-0 \
        libgtk2.0-0 \
        libncurses5 \
        libstdc++6 \
        libtinfo5 \
        libxext6 \
        libxrender1 \
        libxtst6 \
        locales \
        sudo \
        unzip \
    && rm -rf /var/lib/apt/lists/*

# Generate en_US.UTF-8 locale (Vivado log parser expects it)
RUN locale-gen en_US.UTF-8
ENV LANG=en_US.UTF-8

# Create a non-root runner user
RUN useradd -m -s /bin/bash runner && \
    echo "runner ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER runner
WORKDIR /home/runner

# The entrypoint should configure and start the Actions runner.
# Configuration is left to the operator (token rotation, repo URL, labels).
# See SETUP_RUNNER.md for details.
CMD ["/bin/bash"]
