# NOW -- sadd orders before it subtracts, and underflow is zero (2026-10-03)

Refs #5506, Refs #5577.

## emit-bitexact: `gft_sadd.sadd: Zig backend failed to build/run`

The build is fine. The Zig binary is built with `-OReleaseSafe` by
`tools/verify_multitarget.py`, and it traps on two defects in `magsub` and
`sadd`. C and Rust get the same values wrong without a trap. #5505 was the first
change that made the Zig arm run at all.

- **Defect 1: a shift by a negative count.** `sadd` called `magsub(ma, mb)`
  before ordering the operands. `magsub(hi, lo)` shifts by `hi_off - lo_off`, so
  a smaller first operand gives a negative count and Zig traps at
  `ls >> @intCast(d)`.
  - In C this is undefined behaviour, and Rust `-O` leaves it unchecked.
  - Both happened to throw the result away, because the ordered call followed.
- **Defect 2: underflow produces something that is not an encoding.**
  Normalisation stops at its floor (`cap = min(12, off - 1)`). Below the
  smallest binade `q` therefore stays under 512, and `mant` goes negative.
  - `(off << 9) | mant` is then negative, so `sadd(256, 66047)` returned
    `0xFFFFFEFF`, which is not a GF-T16 value.
  - C and Rust wrap on `as u32`; Zig's `@intCast` traps.
- **Fix 1:** `sadd` orders the operands before subtracting. The `bsign` choice
  moved into the same branch.
- **Fix 2:** `magsub` returns 0 when `mant < 0`. GF-T16 has no subnormals, and
  `enc()` already makes every value below 2^-40 zero.
- **Only invalid results change.** A result changes only where it was a shift
  that C left undefined or a negative non-encoding. The gate's comparison is
  untouched.
- **Where the fix went.** The same `magsub`/`sadd` body is copied into 31 specs:
  the 30 `specs/ternary/gft_*.t27` that carry it, plus
  `specs/port/tools/gft_deep_demo.t27`.
  - All 31 are fixed, because `tools/duplicate_bodies_baseline.txt` fails if
    the group splits.
  - All 30 sealed ones are resealed with a t27c built from this base. Every
    spec's own tests pass, and each seal records `tests.failed = 0`.
  - `gft_generalize_demo.t27` and `gft_train_demo.t27` have their own different
    bodies and are unchanged.
- **The Python model** `tools/gft_backprop_microcode.py` gets the same
  flush-to-zero rule.
  - It adds a self-test for `sadd(256, 66047) == 0`.
  - It plants a mutant that removes the flush, and the self-test must catch it.
- **Not fixed: the floor sits at `off = 1`, not `off = 0`.** A difference that
  is still representable in the smallest binade can therefore flush to zero.
  That is a precision question, not a validity one, and this change does not
  address it.

## spec-guards on master: 30 stale seals

`tools/check_seal_currency.py` failed on 30 seals: 15 specs, each with its
twin seal. These are the specs that #5577 lists as failing their own tests:

- fpga: `timing_tb`, `build_verify`
- ml: `residual_connection`, `huber_loss`, `kl_divergence`, `mse_loss`,
  `rmsprop`, `multi_head_attention`
- tri: `html`, `xml`, `bellman_ford`, `pattern`, `merge_sort`, `template`,
  `terminal`

#5580 put these seals back to their pre-#5578 hashes on purpose, and
`tools/seal_baseline.txt` records all 30 as kind `stale`.
`check_seal_currency.py` never read that ledger, so it was red by design, and a
new stale seal would have been hidden among the 30.

- **A reseal is not safe.** It would certify the contracts that #5577 says are
  broken. Since #5605, `t27c seal --save` refuses a spec whose tests fail;
  `--force` would record the failures, but that still signs a broken contract.
  The repair is to fix each spec, then `seal --save`.
- **The change:** `check_seal_currency.py` now reads the ledger through
  `check_seal_coverage.baseline()`, so there is one ledger and one reader. It
  forgives a stale seal only when both of these hold:
  - the ledger records it with kind `stale`;
  - its `spec_hash` still disagrees with the spec.

  A stale generated-code hash whose spec has NOT moved is compiler drift. The
  ledger does not describe that, so it still fails.
- **Self-check.** `--self-check` removes each of the three conditions in turn,
  and in each case the seal must still fail.
  - It also checks that the ledger's kind column is read: a `phantom` entry
    does not forgive.
- **When a ledgered seal holds again,** the scan prints a NOTE telling you to
  shrink the ledger.
- **Path filter.** `spec-guards.yml` now also triggers on
  `tools/check_seal_coverage.py` and `tools/seal_baseline.txt`, because the
  check depends on both.

## Measured

- **Seal scan, current-tree t27c:** 30 seals known-stale, 0 new.
  - A t27c binary older than the base reports 4 more (`railway_deploy`,
    `mod_structure`) whose `spec_hash` has not moved. These are compiler drift
    and are still reported, as designed. Rebuilt from this base, those 4 match.
- **Other checks:**
  - `check_seal_currency.py --self-check`: PASS.
  - `tools/gft_backprop_microcode.py`: OK, including the planted underflow
    mutant.
  - Duplicate-body scan: OK. The magsub group (31) and the sadd group (30)
    stay whole.
- **Not run on this Mac:** the gate's Zig compile, under the owner's rule of
  2026-10-03. The diagnosis comes from the CI log and the emitted Zig, and CI
  runs the Zig arm on this PR.
