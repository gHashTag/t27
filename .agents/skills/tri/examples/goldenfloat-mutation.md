# Example: GoldenFloat Mutation - GF8 Precision Fix

This example demonstrates PHI LOOP for fixing a precision issue in GF8 format.

## Task

Fix GF8 exponent bias causing overflow in edge cases.

## 1. Small Step

Target: `specs/numeric/gf8.t27`

```diff
- pub const EXPONENT_BIAS: u2 = 0b01;  // Incorrect bias
+ pub const EXPONENT_BIAS: u2 = 0b10;  // Correct bias for TF3 range
```

## 2. Hash Seal

```bash
# Before
sha256sum specs/numeric/gf8.t27  # f1a2b3c4...

# Edit spec
sha256sum specs/numeric/gf8.t27  # e5d6f7a8...

# Generate
tri gen --format gf8
sha256sum backend/zig/numeric/gf8.zig  # b9c0d1e2...

# Test vectors
sha256sum conformance/gf8-conformance.json  # a3b4c5d6...
```

Hash seal:
- spec_hash_before: `f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d`
- spec_hash_after: `e5d6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5`
- gen_hash_after: `b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0a1`
- test_vector_hash: `a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d`

## 3. Verify

```bash
tri gen --format gf8
tri test --format gf8
tri verdict --sacred  # Check sacred physics compliance
tri bench --format gf8
```

Output:
```
✓ Generated: backend/zig/numeric/gf8.zig
✓ Test: 127/127 passed
✓ Verdict: CLEAN
✓ Sacred physics: G (1e-12), γ (1e-15) OK
✓ Bench: +0.8% faster
```

## 4. Fixate

```bash
tri experience save --format gf8
tri skill commit
tri git commit -m "fix(gf8): correct exponent bias for TF3 range

Fixes overflow in edge cases:
- Corrects EXPONENT_BIAS from 0b01 to 0b10
- Ensures TF3 compatibility
- Passes sacred physics conformance

Hash seal: e5d6f7a8...

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

## Complete Skill Registration

```json
{
  "skill_id": "gf8_exponent_bias_fix_v1",
  "parent_skill": null,
  "task_id": "numeric-precision-fix",
  "spec_path": "specs/numeric/gf8.t27",
  "spec_hash_before": "f1a2b3c4...",
  "spec_hash_after": "e5d6f7a8...",
  "gen_hash_after": "b9c0d1e2...",
  "test_status": "pass",
  "verdict": "clean",
  "bench_delta": "+0.8%",
  "sealed_at": "2026-04-04T14:22:18Z",
  "sacred_compliance": {
    "G": "pass",
    "γ": "pass",
    "ΩΛ": "n/a"
  }
}
```

## Key Takeaways

- Sacred physics checked with `tri verdict --sacred`
- Bench delta recorded for performance impact
- Sacred compliance tracked in skill registration
