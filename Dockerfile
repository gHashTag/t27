FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release -p igla-trainer

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/igla-trainer /usr/local/bin/
ENTRYPOINT ["igla-trainer"]
