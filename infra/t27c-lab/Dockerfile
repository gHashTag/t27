# t27c lab: the compiler's CI gates, run on Railway. See lab.py's docstring.
FROM rust:1-bookworm

RUN apt-get update \
 && apt-get install -y --no-install-recommends python3 git ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/data/cargo \
    LAB_DATA=/data \
    PYTHONUNBUFFERED=1

WORKDIR /app
COPY lab.py /app/lab.py
RUN python3 /app/lab.py --self-check

CMD ["python3", "/app/lab.py"]
