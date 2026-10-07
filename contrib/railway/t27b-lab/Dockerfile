# t27b lab: runs `t27b corpus` off the owner's Mac (#6071, epic #6063).
#
# Railway builders and runtimes are x86_64 Linux. t27b is an AArch64 backend,
# so it is cross-built for aarch64-unknown-linux-gnu and run under qemu-user;
# the reference path (t27c gen + zig test) runs natively. lab.py does the work
# at run time so that it can follow T27_REF; this image only holds toolchains
# and a warm clone. No secrets: the repository is public and cloned
# anonymously, and the service never writes to GitHub.
FROM rust:1-bookworm

# Same Zig as the repository CI (.github/workflows: ZIG_VERSION 0.16.0).
ARG ZIG_VERSION=0.16.0

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      gcc-aarch64-linux-gnu libc6-dev-arm64-cross qemu-user \
      ca-certificates curl xz-utils git python3 tini \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-x86_64-linux-${ZIG_VERSION}.tar.xz" -o /tmp/zig.tar.xz \
 && mkdir -p /opt/zig \
 && tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 \
 && rm /tmp/zig.tar.xz \
 && /opt/zig/zig version

RUN rustup target add aarch64-unknown-linux-gnu

ENV PATH=/opt/zig:$PATH \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    T27_REPO=https://github.com/gHashTag/t27 \
    T27_REF=master \
    T27_SRV=/srv \
    T27_WORK=/work

# A warm, blobless clone; lab.py fetches T27_REF before every run.
RUN git clone --filter=blob:none --no-checkout "$T27_REPO" /work/t27

# lab.py publishes the git blob sha of both files and the build time as
# `image` in status.json and every run (#6443), so `tri t27b doctor` can tell
# a stale deploy (LAB-IMAGE-STALE). image-built is written after both COPYs:
# a changed file always gets a new time.
COPY lab.py Dockerfile /opt/t27b-lab/
RUN date -u +%Y-%m-%dT%H:%M:%SZ > /opt/t27b-lab/image-built

WORKDIR /work
# tini is PID 1 and reaps every orphan reparented to it; lab.py as PID 1 never
# did, and the lab ran out of its 1000 pids on zombies (#7090).
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["python3", "-u", "/opt/t27b-lab/lab.py"]
