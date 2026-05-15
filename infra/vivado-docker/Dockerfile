# Vivado 2025.2 ML Standard image for t27 FPGA CI.
# Strips down install to Artix-7 family only (~35 GB on disk) to fit
# GitHub-hosted ubuntu-latest runner after easimon/maximize-build-space.
#
# Build context expects: Vivado_2025.2_Lin64.bin and install_config.txt next to this Dockerfile.
# Built once and pushed to ghcr.io/ghashtag/t27-vivado:2025.2 by
# .github/workflows/build-vivado-image.yml (manual dispatch).

FROM ubuntu:22.04 AS installer

ENV DEBIAN_FRONTEND=noninteractive TZ=UTC

# Runtime + installer dependencies for Vivado (libtinfo5 / libncurses5 are the classic gotchas).
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl wget xz-utils \
        libc6 libstdc++6 libtinfo5 libncurses5 libx11-6 libxext6 libxrender1 libxtst6 libxi6 \
        libfontconfig1 libfreetype6 libgtk2.0-0 libcanberra-gtk-module libgomp1 \
        libusb-1.0-0 libssl-dev libtinfo-dev \
        locales \
    && locale-gen en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

ENV LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8

COPY Vivado_2025.2_Lin64.bin /tmp/installer.bin
COPY install_config.txt /tmp/install_config.txt

RUN chmod +x /tmp/installer.bin \
    && /tmp/installer.bin \
        --agree XilinxEULA,3rdPartyEULA \
        --batch Install \
        --config /tmp/install_config.txt \
    && rm -f /tmp/installer.bin /tmp/install_config.txt \
    && find /opt/Xilinx -name "*.tar.gz" -delete 2>/dev/null || true \
    && find /opt/Xilinx -name "*.zip" -delete 2>/dev/null || true \
    && rm -rf /opt/Xilinx/Vivado/2025.2/data/parts/xilinx/{kintex,virtex,zynq,versal,spartan,ultrascale}* 2>/dev/null || true

# Final stage: copy only the pruned install to keep image lean.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive TZ=UTC LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libc6 libstdc++6 libtinfo5 libncurses5 libx11-6 libxext6 libxrender1 libxtst6 libxi6 \
        libfontconfig1 libfreetype6 libgtk2.0-0 libcanberra-gtk-module libgomp1 \
        libusb-1.0-0 libssl-dev \
        locales tini \
    && locale-gen en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=installer /opt/Xilinx /opt/Xilinx

ENV PATH="/opt/Xilinx/Vivado/2025.2/bin:${PATH}" \
    XILINXD_LICENSE_FILE=""

WORKDIR /work
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["bash"]
