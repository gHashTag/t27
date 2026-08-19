# Wave Loop 603 — the check was wrong, not the catalog

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_602_REPORT.md`](WAVE_LOOP_602_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W602 reported 5 findings.  FOUR WERE NOT DEFECTS.
The catalog's `s` field is signedness, not layout -- and the catalog
already had a documented "not applicable" sentinel W602 never looked for.

Then the real work: the EMITTED artifacts, which history says is the
failure that actually happened.
```

---

## 1. Refuting this chain's own most recent finding

W602's recommended variant was *"settle the five `no-spurious-layout` records"*.
The falsification check asked what the catalog's own convention is, and it
answers immediately:

| `s` | records | signed in reality? |
|---:|---|---|
| **1** | `q_format`, `minifloat`, `unum_i`, `tapered_fp` | **yes** — Qm.n, minifloat, Unum I and Morris tapered FP all carry a sign bit |
| **0** | `bcd`, `block_fp`, `shared_exp`, `stochastic_rounding`, `unum_ii` | **no** — none is a signed scalar format; `unum_ii` is SORN projective |

**The split is exactly the signed / not-a-signed-scalar-format line.** `bits=0`
means *the width is parameterised*; `s=1` means *the family has a sign bit*.
Independent facts, not a contradiction.

And the catalog has a documented sentinel for "not applicable":
**`phi_distance=-1.0`, used by 46 records.** W602 called the data wrong without
checking whether a convention existed.

**One finding survives** — the only case that cannot be a convention under any
reading, a *concrete* width exceeded by its own fields:

```
gfternary   bits=2 is concrete, but s+e+m = 1+0+2 = 3 exceeds it   status=Verified
```

`gfternary` is the 3-value set {−φ, 0, +φ}; three values need 2 bits
(`storage=u2`), so `bits=2` is right and `s=1 m=2` appears to record *the
alphabet* in fields that mean *field widths*. **A specification decision** —
reported, not changed. Recorded as **P18**, with **P17 annotated at its head.**

> **Tenth instance in this chain of the instrument being wrong rather than the
> code — and the second published finding refuted by its own data.** W588
> counted module references with a regex that matched path prefixes. The failure
> mode is identical: **asserting what data means before asking what it means
> here.**

## 2. The emitted artifacts — the failure that actually happened

The catalog exists to feed generated targets. Git says how that has gone:

```
aa01dd4f1  fix(gen): untrack stale gen/numeric catalog artifacts (drift 77 vs SSOT 83)
```

**The emitted files fell six formats behind the source, and the remedy was to
delete them.** That removes the symptom and prevents nothing — because nothing
compared them.

### Three defects, found by trying to run it

1. **The generator fails from the repo root.** Its defaults were
   `formats_catalog.t27` and `gen_catalog/` *relative to the current directory*,
   so `python3 tools/gen_formats_catalog.py` — the only invocation anyone would
   type — died with `FileNotFoundError`. Now defaults to the repo-relative paths
   its own header documents.
2. **Its header documented six output languages. It emits sixteen** — md, json,
   py, rs, h, ts, zig, go, swift, java, kt, hpp, vh, hs, ml, jl.
3. **My own check under-measured 4× and reported success.** `emitted-agrees`
   looked up `s`/`e`/`m`; the emitter renames them `s_bits`/`e_bits`/`m_bits`.
   It found nothing, silently compared only `bits`, and printed *"83 fields
   compared"* as though thorough. The real number is **332**.

   *This is the exact failure mode the whole chain exists to catch, written by
   the hand that has been cataloguing it for thirty-five waves.*

### Verified by breaking it

```
corrupt binary16.e_bits -> 99, remove gf1024:
  [emitted-agrees] SSOT has 83 records, emitted JSON has 82 -- exactly the drift
                   that untracked the artifacts in aa01dd4f1 (77 vs 83)
  [emitted-agrees] binary16: SSOT e=5 but emitted e_bits=99
  [emitted-agrees] gf1024: in the SSOT but not in the emitted JSON
restore:  FINDINGS 1
```

**An absent artifact is reported as absent, not as a mismatch** — `gen/numeric/`
is gitignored by design, and merging "not generated" with "generated wrong" is
the collapse this chain has now unwound four times.

## 3. Verification

| Gate | Result |
|---|---|
| `catalog-gate` | 83 records · 83 getters · **8 check kinds** · 332 emitted fields · **1 finding** |
| `cordic.t27` | 330 / 336 |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| `cc-gate` | 101 |
| `catalog_gate` unit tests | 7 |

---

## 4. Three cooperation variants for W604

### Variant A (recommended) — Make `catalog-gate` part of the suite

Eight instruments now exist — `lex-conform`, `parse-conform`, `check-calls`,
`cc-gate`, `impl-status`, `parse-complete`, `test-report`, `catalog-gate` — and
each must be invoked by name. Nothing runs them together, so a regression in any
one is invisible until somebody remembers to check.

**This wave found the argument for it.** The 5 pre-existing Verilog unit-test
failures W602 reported have been failing since W459 and no gate reports them;
the emitted artifacts drifted by six formats and no gate reported that either.
**A gate nobody runs is a gate that does not exist.**

### Variant B — `gfternary`, and what an alphabet records

One record, one decision: should a 3-symbol alphabet state `s`/`e`/`m` at all,
or a symbol count? It is the last open finding in the catalog and the smallest
decision left anywhere in this chain's backlog.

### Variant C — Flash the board

Unchanged, and now backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete;
Phase 1 begins with `dlc10 idcode` and must not skip it.

---

## Recommendation

**Variant A.** This wave and the last both found defects that existed for many
waves because no single command would surface them. Eight instruments that must
be remembered individually are, in practice, eight instruments that are not run.

---

*φ² + φ⁻² = 3 | TRINITY*
