# Correction: the "4-MiBit Icarus/Yosys cliff" is a CI time budget, not a capability limit

**Date:** 2026-08-12 · **Status:** correction to a threshold quoted across the
`FPGA_LOOP_CLOSEOUT_*` series · **Method:** `~/skills/claim-audit-lab/scripts/reference_column_sweep.py`

The dated closeout reports are historical records and are **not** edited by this note. This
file is the correction; the reports keep their original text.

---

## What is quoted

Across the FPGA closeout series a threshold appears as a capability boundary of the tools:

| Where | Text |
|---|---|
| `FPGA_LOOP_CLOSEOUT_W653_2026-07-07.md:96` | *"0.244 MiBit is far below **the 4-MiBit Icarus/Yosys cliff**"* |
| `FPGA_LOOP_CLOSEOUT_W826_2026-08-01.md:68` | *"intentionally **crosses the 4-MiBit cliff**; use only if explicitly desired"* |
| `FPGA_LOOP_CLOSEOUT_W856_2026-08-05.md:57` | *"The established **4-MiBit soft cliff** remains the next meaningful watch-point"* |
| `docs/NOW.md:1494` | *"4-MiBit cliff"* |

The word *cliff*, and the attribution to *Icarus/Yosys*, together assert that the simulators
fail at or near 4 MiBit.

## What was measured

`FPGA_LOOP_CLOSEOUT_W584_2026-07-07.md` is the only run in the corpus **at** 4 MiBit
(4,194,304 bits), and it **passed both paths**:

| Check | Result |
|---|---|
| Direct `t27c icarus-simulate` W584 | **PASS** (~22.5 min wall-clock) |
| Direct `t27c icarus-cocotb` W584 | **PASS** (~23.7 min wall-clock) |

Its own "Weak spot addressed" section says so explicitly:

> *"confirmed that **Icarus 12.0 can simulate a 4-MiBit packed vector** when the
> assertion-side literal is bound to a local variable"*

`FPGA_LOOP_CLOSEOUT_W585_2026-07-07.md:9` records the same: *"W584 already reached 4 MiBit"*.

## What the real constraint is, in W584's own words

The same report's **"Risks accepted"** names it:

> *"Direct simulation wall-clock is now ~22.5 min; **18-D would likely exceed 40 min and
> approach CI timeout limits**."*

**That is a time budget, and it is a true and defensible statement.** It became
*"the 4-MiBit Icarus/Yosys cliff"* in later reports — a *capability* limit attributed to the
*tools*. The two are different claims with different consequences:

| Claim | Status | If a reviewer checks it |
|---|---|---|
| "Icarus/Yosys cliff at 4 MiBit" | **refuted by W584** | the one run at that size passed; the claim fails immediately |
| "our CI budget stops around 22 min wall-clock, which 4 MiBit reaches" | **supported by W584** | holds, and explains the engineering decision |

There is also a second, separate constraint W584 records and the "cliff" phrasing hides: at
rank 17 indexed probes cover only the lower half of the address space, because of the signed
`i16` field range (`e ≤ 16383`) in the chosen `Pt { x: i16, y: i16 }` struct. That is a
representation limit, not a simulator limit either.

## What to write instead

> The AoS rank-scaling series is bounded by CI wall-clock, not by simulator capacity. W584
> simulates a 4-MiBit packed vector successfully in ~22.5 min (`icarus-simulate`) and ~23.7 min
> (`icarus-cocotb`); 18-D is projected past 40 min and would approach the CI timeout. Treat
> 4 MiBit as the current budget ceiling, and note that no capability failure has been observed
> at that size.

**Do not describe a threshold as a limit of someone else's tool when your own measurement at
that threshold passed.** A vendor-attributed limit is the easiest kind of claim for a reviewer
to test, and this one is contradicted by a report in the same directory.

## Provenance

Found by `reference_column_sweep.py`, which flags a bare numeric threshold in an
expected-style column on a row carrying a failure mark with no stated source. 24 rows in this
family matched; all were CONFIRMED against the same missing support. The generalisation is
recorded as entry 17 in `~/skills/claim-audit-lab/SKILL.md`: *a computable or measurable
reference, quoted instead of checked.*
