# Example: Toxic Verdict Rollback

This example demonstrates rollback procedure when `tri verdict --toxic` returns toxic.

## Task

Update TRINITY constant in sacred physics.

## 1. Small Step (FLAWED)

Target: `specs/math/sacred_physics.t27`

```diff
- pub const TRINITY: PackedTrit = @as(PackedTrit, 0b01_00_01_00);  // Wrong value
+ pub const TRINITY: PackedTrit = @as(PackedTrit, 0b10_10_10_10);  // Wrong value!
```

## 2. Hash Seal

```bash
sha256sum specs/math/sacred_physics.t27  # before: a1b2c3d4...
sha256sum specs/math/sacred_physics.t27  # after:  e5f6a7b8...
tri gen --module sacred_physics
sha256sum backend/zig/math/sacred_physics.zig  # c9d0e1f2...
sha256sum conformance/sacred-physics.json  # a3b4c5d6...
```

## 3. Verify → TOXIC

```bash
tri gen --module sacred_physics
tri test --module sacred_physics
tri verdict --sacred
```

Output:
```
✗ Generated: backend/zig/math/sacred_physics.zig
✗ Test: 0/1 passed
✗ Verdict: TOXIC
✗ Reason: TRINITY exact tolerance violation
✗ Expected: 0b01_00_01_00
✗ Actual:   0b10_10_10_10
✗ Rollback required
```

## 4. Fixate (Toxic Verdict) — ROLLBACK

**NEVER** roll back the generated `.zig` file. Always restore the spec.

```bash
# Record mistake (for experience learning)
tri experience record --mistake "TRINITY value incorrect"
--spec specs/math/sacred_physics.t27 \
--hash a1b2c3d4... \
--verdict toxic \
--reason "Exact tolerance violation for sacred constant"

# Rollback SPEC, not generated code
git restore specs/math/sacred_physics.t27

# Verify spec is restored
sha256sum specs/math/sacred_physics.t27  # Should match: a1b2c3d4...

# No skill commit — toxic verdict = immutable failure
# No git commit — toxic changes never committed
```

## 5. Retry with Correct Small Step

Target: `specs/math/sacred_physics.t27`

```diff
- pub const TRINITY: PackedTrit = @as(PackedTrit, 0b01_00_01_00);  // Original
+ // TRINITY remains unchanged - no mutation needed
```

Or if change was actually required:

```t27
pub const TRINITY: PackedTrit = @as(PackedTrit, 0b01_00_01_00);  // Verified correct
pub const TRINITY_EXTENDED: PackedTrit = @as(PackedTrit, 0b10_01_00_01);  // New constant
```

## 6. New PHI LOOP

```bash
tri skill begin "sacred-physics-extension"
tri spec edit math/sacred_physics
tri skill seal --hash
tri gen
tri test
tri verdict --sacred  # Now returns CLEAN
tri experience save
tri skill commit
tri git commit -m "feat(sacred): add TRINITY_EXTENDED constant

Hash seal: [computed]

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

## Key Takeaways

- **Toxic verdict** = immediate rollback
- **Rollback spec**, never generated code
- **Record mistake** for experience learning
- **No skill commit** on toxic verdict
- **No git commit** on toxic verdict
- **Verify spec** before retrying

## Toxic Verdict Triggers

| Trigger | Example | Consequence |
|----------|-----------|--------------|
| Exact tolerance violation | TRINITY wrong value | Immediate rollback |
| Absolute tolerance violation | G outside 10^-12 | Immediate rollback |
| Relative tolerance violation | γ outside 10^-15 | Immediate rollback |
| Circular dependency | Graph cycle | Fix graph before proceeding |
| Forward tier dep | Tier 1 → Tier 2 | Fix tier assignment |
