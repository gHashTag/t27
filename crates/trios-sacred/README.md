# trios-sacred

Rust FFI bindings for [zig-sacred-geometry](https://github.com/gHashTag/zig-sacred-geometry) — φ-attention, Fibonacci spirals, Beal conjecture search.

## Setup

```bash
cd crates/trios-sacred
git submodule update --init vendor/zig-sacred-geometry
```

## Usage

```rust
use trios_sacred::{golden_sequence, phi_bottleneck, beal_search};

let seq = golden_sequence(10);
let bn = phi_bottleneck(512);
let candidates = beal_search(2, 100, 10, 50);
```
