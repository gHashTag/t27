# Ready patch — allow `as f32` / `as f64` casts

**Status:** written and verified as a diagnosis; **not applied**, because
applying it re-runs `bootstrap/build.rs`, which panics on a pre-existing
LANG-EN violation (see `WAVE_LOOP_549_RESEARCH.md` §4.4). Apply it in the same
wave that clears that blocker.

## Finding

`f32` and `f64` are first-class in the compiler: `TypeInfo::F32` exists, they
are accepted as parameter and return types, and the Zig/Rust/C emitters handle
them. They were simply missing from the **cast** whitelist, so:

```t27
fn f(x: f32) -> f32 { return x; }        // parses fine
fn g(x: u32) -> f32 { return x as f32; } // parse error: unknown cast target type `f32`
```

Verified on this host with two minimal specs. This blocks three IGLA specs at
the parser — `eda.t27`, `eval.t27`, `training.t27` — before any backend gets a
chance to decide whether it can represent a float.

## Patch

`bootstrap/src/compiler.rs`, in the `as`-cast parser (~line 2626):

```diff
         const VALID_CAST_TYPES: &[&str] = &[
-            "bool", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize",
+            "bool", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize",
+            "f32", "f64",
         ];
```

## After applying

1. `bootstrap/stage0/FROZEN_HASH` must be resealed — `compiler.rs` changed.
2. Re-measure with `t27c synth-gate --specs-dir specs/igla/race` and
   `t27c gen-verilog` across `specs/igla/**`.
3. Expect the three float specs to move past the parser. They may then fail in
   a backend that cannot represent floats — which is the correct place for that
   decision, and a separate finding if it happens.

*phi^2 + phi^-2 = 3 | TRINITY*
