# Sacred Geometry Decision — A1-Relaxed

**Date**: 2026-04-19  
**Decision**: A1-relaxed (local vendor as temporary measure)  
**Status**: Accepted

## Context

The `zig-sacred-geometry` repository (referenced as an upstream Zig package) returned HTTP 404 during П1 integration. This blocked the `trios-sacred` crate from compiling with FFI enabled.

## Decision

Created a **local vendor** at `crates/trios-sacred/vendor/zig-sacred-geometry/` with:

- `build.zig` — Static library target (`libsacred_geometry.a`)
- `build.zig.zon` — Package manifest with local fingerprint
- `src/c_abi.zig` — 6 C-ABI exports matching `trios-sacred` FFI declarations:
  - `sacred_phi_attention` — φ-weighted attention matrix with softmax normalization
  - `sacred_fibonacci_spiral` — Fibonacci spiral coordinates (x = cos(t)·φ^t, y = sin(t)·φ^t)
  - `sacred_golden_sequence` — Golden ratio low-discrepancy sequence
  - `sacred_beal_search` — Beal conjecture search (A^m + B^n = C^r with gcd > 1)
  - `sacred_phi_bottleneck` — Nearest Fibonacci number ≤ model_dim
  - `sacred_head_spacing` — φ^n mod 1 for attention head spacing

## Rationale

- A1 rule: "All vendors must be upstream git submodules"
- A1-relaxed: Local vendor is acceptable as TECH_DEBT when upstream is unavailable
- The local vendor provides the correct C-ABI interface and compiles to a static library
- When the upstream repo becomes available, the local vendor can be replaced with a submodule

## Consequences

- **Positive**: `trios-sacred` compiles and links with FFI enabled
- **Negative**: Local vendor must be manually maintained until upstream is available
- **TECH_DEBT**: Track in TECH_DEBT.md for П2 resolution

## Resolution Path (П2)

1. Check if `zig-sacred-geometry` upstream repo becomes available
2. If yes: replace local vendor with git submodule
3. If no: consider publishing the local vendor as the upstream
