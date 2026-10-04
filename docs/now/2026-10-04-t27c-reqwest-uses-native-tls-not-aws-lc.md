# NOW -- t27c reqwest uses native-tls, not aws-lc (2026-10-04)

## t27c reqwest uses native-tls, not aws-lc (Closes #5919)

- reqwest 0.13 defaults to rustls on aws-lc-rs. aws-lc-sys is a C and assembly library built by its own cmake script, and in a clean `--features net` build it was the longest single unit (134 s wall, on a machine at load average ~250), ahead of t27c itself. t27c makes HTTPS calls from one place, `audio`.
- reqwest now uses `default-features = false` with `native-tls` in place of `default-tls`. `charset`, `http2`, `system-proxy`, `json`, `blocking` and `stream` stay. `aws-lc-sys` is gone from every feature set, and the `--features net` binary shrinks from 15.6 MB to 12.6 MB.
- An HTTPS GET through reqwest with the same features returns 200 over HTTP/2, so ALPN still works. The Dockerfile already installs `libssl-dev`, and its runtime `ca-certificates` pulls `libssl3`, so it needs no change.
