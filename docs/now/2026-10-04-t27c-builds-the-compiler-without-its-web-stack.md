# NOW -- t27c builds the compiler without its web stack (2026-10-04)

## t27c builds the compiler without its web stack (Closes #5912)

- `cargo build -p t27c` compiled an HTTP client, TLS, tokio, axum, hyper and a JWT library for every build, though only `serve`, `bench-endpoints`, `bridge` and `audio` use them. The default build now resolves 49 crates instead of 212.
- New feature `net` (reqwest) serves `bridge` and `audio`. `server` implies `net` and keeps axum, tokio, hyper, tower-http, jsonwebtoken and the rest. `uuid`, `tower` and `ignore` were not imported anywhere and are removed.
- Without `net`, `t27c bridge status` and `t27c audio` exit 1 and print the `--features net` rebuild command. `bridge handoff` stays offline. The HTTP half of bridge moved to `bootstrap/src/bridge/client.rs`.
- The `jwt` and `proxy` unit tests still run under a plain `cargo test`: the crates they need are dev-dependencies (#2301). The `Dockerfile` already builds `--features server` and is unchanged.
