# Example: Small Step - Adding New Trit Operation

This example demonstrates PHI LOOP for adding a new trit operation to the base types.

## Task

Add `tritRotate(a: Trit) -> Trit` operation to rotate trit values.

## 1. Small Step

Target: `specs/base/ops.t27`

```diff
+ pub fn tritRotate(a: Trit) Trit {
+     return switch (a) {
+         .neg => .pos,
+         .zero => .neg,
+         .pos => .zero,
+     };
+ }
```

## 2. Hash Seal

```bash
# Compute spec_hash_before
sha256sum specs/base/ops.t27  # a7f3c9d...

# Edit spec (Small Step)
# Compute spec_hash_after
sha256sum specs/base/ops.t27  # b2e8a1f...

# Generate backend
tri gen --module base/ops

# Compute gen_hash_after
sha256sum backend/zig/base/ops.zig  # c3d9b2e...
```

Hash set recorded:
- spec_hash_before: `a7f3c9d5e2f1b8a4c6d7e9f0a1b2c3d4e5f6a7b8c9d0e1f2`
- spec_hash_after: `b2e8a1f3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c`
- gen_hash_after: `c3d9b2e1a0f9e8d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0`
- test_vector_hash: `d4e0c9b2a1f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1f0e`

## 3. Verify

```bash
tri gen --module base/ops
tri test --module base/ops
tri verdict --toxic
```

Output:
```
✓ Generated: backend/zig/base/ops.zig
✓ Test: 1/1 passed
✓ Verdict: CLEAN
```

## 4. Fixate (Clean Verdict)

```bash
tri experience save
# Records: diff, hashes, verdict, bench_delta
tri skill commit
tri git commit -m "feat(ops): add tritRotate operation

Adds rotation operation:
- neg -> pos
- zero -> neg
- pos -> zero

Hash seal: b2e8a1f...

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

## Complete Skill Registration

```json
{
  "skill_id": "ops_tritRotate_v1",
  "parent_skill": null,
  "task_id": "base-ops-extension",
  "spec_path": "specs/base/ops.t27",
  "spec_hash_before": "a7f3c9d...",
  "spec_hash_after": "b2e8a1f...",
  "gen_hash_after": "c3d9b2e...",
  "test_status": "pass",
  "verdict": "clean",
  "bench_delta": "+0.2ns",
  "sealed_at": "2026-04-04T12:34:56Z"
}
```

## Key Takeaways

- One node, one mutation: Only modified `specs/base/ops.t27`
- Hash seal recorded before and after generation
- Verdict clean → experience saved and committed
- Skill registration creates immutable record
