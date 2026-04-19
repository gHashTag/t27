# trios-crypto

Rust FFI bindings for [zig-crypto-mining](https://github.com/gHashTag/zig-crypto-mining) — Bitcoin mining, SHA-256d, DePIN proof-of-work.

## Setup

```bash
cd crates/trios-crypto
git submodule update --init vendor/zig-crypto-mining
```

## Usage

```rust
use trios_crypto::{sha256, double_sha256};

let hash = sha256(b"hello world").unwrap();
let double = double_sha256(b"hello world").unwrap();
```
