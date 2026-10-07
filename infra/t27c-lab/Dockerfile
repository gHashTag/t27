# t27c lab: the compiler's CI gates, run on Railway. See lab.py's docstring.
FROM rust:1-bookworm

RUN apt-get update \
 && apt-get install -y --no-install-recommends python3 git ca-certificates xz-utils tini \
 && rm -rf /var/lib/apt/lists/*

# zig for re-seals by hand only (`PATH=/opt/zig:$PATH t27c seal ... --save`),
# so a seal's `tests` field records a real compile. Not on PATH: the gates run
# without zig, as CI's do. 0.16.0 is the zig the seals were made with.
ARG ZIG_SHA256=70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
RUN curl -fsSL -o /tmp/zig.tar.xz https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz \
 && echo "$ZIG_SHA256  /tmp/zig.tar.xz" | sha256sum -c - \
 && mkdir /opt/zig && tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 \
 && rm /tmp/zig.tar.xz && /opt/zig/zig version

ENV CARGO_HOME=/data/cargo \
    LAB_DATA=/data \
    PYTHONUNBUFFERED=1

WORKDIR /app
COPY lab.py /app/lab.py
RUN python3 /app/lab.py --self-check

# tini is PID 1 and reaps every orphan reparented to it; lab.py as PID 1 never
# did, and the lab ran out of its 1000 pids on zombies (#7090).
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["python3", "/app/lab.py"]
