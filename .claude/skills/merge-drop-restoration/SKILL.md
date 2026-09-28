---
name: merge-drop-restoration
description: How to restore a phase a batch merge silently dropped from cli/tri — the fixture-driven method that closed the August 824/3 trio (validate_lean_standalone phase, synthetic cclk_sweep parameter, W450 sweep JSON shape). Load when a tri test fails on a report shape or a flag that is never set, when git log -S "finds nothing", or when a restoration must match a snapshot fixture byte-for-byte.
---

# Merge-drop restoration

One 2026-08-06 bulk merge left at least four independently-hidden drops in
`cli/tri/src/fpga.rs`. Each hid the next because the tests that could see it
failed earlier on the one before. This is the method that unwound all four in
one pass — verified 824/3 → 827/0.

## The signature of the defect family

- A flag is initialized `false`, read by a `passed` conjunction, never set —
  because the code that sets it no longer exists.
- A test fails at `passed == true` long before its deeper asserts, so shape
  drift underneath stays invisible for months.
- `git log -S '<the dropped code>'` finds only the ADDER, never the remover —
  merge commits don't show in `-S` without `-m`. Absence of a removing commit
  is NOT evidence the code was never there.

## The method (fixture-driven, in order)

1. **Find the pin.** The snapshot fixture (`tests/fixtures/...`) is the
   contract. Read it before reading the code: `report_json`, `source:
   "synthetic"`, `variant_count` — the fixture names exactly what the drop
   took.
2. **Find the fixture-writing commit.** `git log -- <fixture path>` → the
   commit that added it (here: W450, 8eb0caf5c). That commit's version of the
   code is the shape the fixture pinned.
3. **Extract, don't remember.** `git show <commit>:cli/tri/src/fpga.rs >
   /tmp/ref.rs` and diff against the live file. Restore verbatim, then adapt
   only where the surrounding signature legitimately moved (the 21-arg
   `measured_to_lean` call was unchanged since W448 — exact match).
4. **Check for readers before deleting reshaped variants.** d51db4ac1 had
   "restored" the flag with a reshaped JSON entry; grep showed its keys had
   zero readers. The fixture wins. But if a later shape HAS readers, keep a
   superset — the snapshot check is strict-superset, extras are allowed.
5. **Follow the parameter chain to the end.** The merge dropped
   `synthetic_operating_point` from `cclk_sweep`'s SIGNATURE (18→17 params),
   from the CLI flag, AND from the PVT resolution. A partial restoration
   (d51db4ac1) compiled fine and still lied (`"not_read"` hardcoded). When a
   value must reach a log entry, trace the writer, not just the reader.
6. **Comment the restoration at the site** — name the merge and what it
   dropped ("survived the merge that dropped the call site" style, see
   fpga.rs ~6280). The next person diffing needs to know this is a
   restoration, not new behavior.

## When the "bug" is a false lemma

The same pass found two W472 Lean statements that were FALSE, not unproven:
`RawNsPredicate base` never implied `RawNsPredicate (base + 2)` (base=1000 →
1002 > 1000). Strengthen the hypothesis or fix the bound, with a comment
naming the counterexample — never add an axiom or `sorry` to make a false
statement compile.

## Hazards specific to this family

- **Pseudo-Lean from bulk merges**: `struct` (not `structure`), `arr.length`
  (not `.size`), `arr.update` (not `.set` with proof param), `[...]` for
  Array (not `#[...]`), `Array.forall` (does not exist; it's `Array.all`).
  If a proof file "from a wave" uses these, it never compiled — check
  whether any gate ever ran it (master lean CI has NO push trigger,
  deliberately; #5082 pending).
- **`lake build --wfail` in CI**: dead tactic tails (`ring_nf`/`simp`/
  `aesop` after a closing `simp`) are warnings and will fail the gate even
  when the build succeeds locally without `--wfail`.
- **W846**: `cmd | head; echo rc=$?` reports head's status. Unpiped, or
  `PIPESTATUS`, before believing a tool's exit code.
