# trios-hdc

Rust FFI bindings for [zig-hdc](https://github.com/gHashTag/zig-hdc) — Hyperdimensional Computing / Vector Symbolic Architecture.

## Setup

```bash
cd crates/trios-hdc
git submodule update --init vendor/zig-hdc
```

## Usage

```rust
use trios_hdc::HdcSpace;

let space = HdcSpace::new(10000);
let a = space.random_vector();
let b = space.random_vector();
let bound = space.bind(&a, &b);
let sim = space.similarity(&bound, &a);
```
