# Mutation census of specs/analog/sar_switch_q.t27, 2026-10-25

Issue #12086, part of epic #10486 (weak spots of the analog v0.1 release). Every number below was read from a run of
`tri mutate spec` on 2026-10-25 against master `b9d83f9ef2` with the master-built `t27c`; the tool never edits the spec,
and the checkout stayed clean during the runs.

## What was run

Before (the census, exit 0):

```
tri mutate spec --file specs/analog/sar_switch_q.t27 --t27c <master-built t27c> --jobs 4 --timeout 30
```

After (the gate, 03:12-03:19 +07, exit 0, "Gate passed."):

```
tri mutate spec --file specs/analog/sar_switch_q.t27 --t27c <master-built t27c> --jobs 3 --timeout 30 \
  --accepted docs/analog/mutation-accepted-sar_switch_q.txt
```

## Census

| Run | Mutants | Killed | Survived | Hung | Unviable |
| --- | --- | --- | --- | --- | --- |
| Before (master) | 169 | 102 | 48 | 14 | 5 |
| After (this change) | 169 | 131 | 19 | 14 | 5 |

By kind (same in both runs): drop-guard 8, flip-cmp 42, swap-logic 2, drop-step 12, swap-arith 93, ret-default 12.

The gate line: 33 not killed (19 survived, 14 hung), 33 accepted by the file, 0 accepted lines now killed.

## The 48 survivors before, and what became of each

| Region (lines) | Survivors | Verdict | Outcome |
| --- | --- | --- | --- |
| `qs_dev`, the gate drive, body bias and Vds of the NMOS and PMOS (150, 151, 153, 154, 157) | 13 | no original test read the rounding and the sums at a word that tells the mutant from the original; the equality branches (`vg == vlo`, `vlo == vb`, `vhi == vg`) leave the same zero, and the PMOS body branch (154) never runs in this netlist (every PMOS bulk is its high end or the supply) | 6 killed (150 swap-arith x2, 151 swap-arith `- 0.5`, 153 swap-arith x2, 157); 7 accepted (150, 151 x2, 153, 154 x3) |
| `qs_dev`, the drain/source swap (140) | 1 | equivalent: at `vd == vs` there is no channel current and both ends carry the same charge | 1 accepted |
| `qs_nodes`, the no-charge mode (198) | 2 | test gap: in the no-charge mode the clock node (node 6, no lumped capacitor) carries a unit charge per volt, and no original test ran that mode through `qs_nodes` | 2 killed |
| `qs_iter` (253) | 2 | `>` to `>=` is equivalent (the scale is 1.0 at the bound); `/` to `*` was a test gap | 1 killed, 1 accepted |
| `qs_step` (270, 272, 273, 275) | 5 | the bound of the sixty moves, the counter and the 61 for "did not settle" were never read; 273 (`<` to `<=` on the 1 uV tolerance) differs only when the largest move equals the tolerance to the last bit | 4 killed (270, 272, 275 x2), 1 accepted (273) |
| `qs_rest` (280) | 2 | test gap: the rest level of vout (0.1 V under the input) was not pinned | 2 killed |
| `qs_clk_rise`, `qs_clk_fall` (292, 298) | 3 | the ramp end at 2 ns: the dropped guard was a gap; `>=` to `>` is equivalent (the ramp gives the rail exactly at 2000 ps) | 1 killed (292 drop-guard), 2 accepted (292, 298) |
| `qs_track` (327, 328, 330, 335, 337, 338, 340) | 12 | the step schedule (the rise window, the growth to 500 ps, the clip at the end) was never read; the `worst` updates at 330 and 340 and the clips at 337 and 338 write the value already held at the tie | 8 killed (327 x5, 328, 335, 338 swap-arith), 4 accepted (330, 337, 338, 340) |
| `qs_hold` (361, 362, 364, 367) | 7 | the clock at the end of a fall step and the reading window at 4 ns were never read; 361 takes one extra fall step that moves the state but no field of `QsHold`; 364 writes the value held | 4 killed (362, 367 x3), 3 accepted (361 x2, 364) |
| `qs_droop_v` (378) | 1 | test gap: `cl_ff - 4 fF` instead of `+ 4 fF` | 1 killed |

Totals: 48 survivors, of which 29 killed and 19 accepted as equivalent or dead (the rows add up: 13 + 1 + 2 + 2 + 5 + 2
+ 3 + 12 + 7 + 1 = 48; killed 6 + 2 + 1 + 4 + 2 + 1 + 8 + 4 + 1 = 29; accepted 7 + 1 + 1 + 1 + 2 + 4 + 3 = 19). The
per-mutant attribution to the test that killed it was not measured, only the totals before and after.

The 14 hung mutants are drop-step mutants (ten) and swap-arith mutants (four) that make a loop endless, so no test can go
red and only the clock ends the mutant. They are named in the accepted file with that reason. The 5 unviable mutants
empty a void function (`ret-default`), which `zig` rejects ("not yet implemented") before any check runs.

## The 14 tests added (appended after the last original test, so the accepted line numbers did not move)

- the netlist, per node and per device, as the deck gives it (10 asserts);
- the NMOS switch and the PMOS pull-up reading gate drive, Vds and body bias in whole words, with the tables' charges
  and currents at those words (3080 and 3074 runtime asserts: a scan of word grids);
- below its source the NMOS gate, and above its high end the PMOS gate, is `xn` words past it, with the table's charge at
  zero plus the linear tail (12289 and 8193 asserts);
- the nodes with the lumped capacitors, the boost capacitor and the clock node (33 asserts);
- the rest state and the clock ramp over 2 ns in both directions (18);
- one Newton iteration moves no node more than a quarter volt, and a step reports 61 after exactly sixty (3);
- a step equals a hand loop of Newton moves, count and state node for node (45);
- a far clock does not settle in sixty moves and a near one settles in a few (2);
- the track phase is the step schedule its comment gives, in four cases (36);
- the hold run reads vout when 4 ns of the fall have passed and ends 6 ns after it began, whatever the step (10);
- the droop is the leak over 413 ns into the load plus the cell's 4 fF, and scales with the multiplier (4);
- the hold run ends in the state of one hand step when a fall step is 6 ns or half a ps short of it (6);
- one more fall step after 6 ns moves vout and the gate node but not the leak of the off switch (6): this one pins why
  the two mutants at line 361 are accepted rather than killed.

`tri test specs/analog/sar_switch_q.t27`: 22 tests pass, 0 fail, 0 vacuous passes. The seal is re-saved with the
master-built `t27c` (`tri seals changed`: seal-save, seal-recheck, importers and importer-seal pass).
`tri coverage 'specs/analog/*.t27' --strict`: 26 specs, `gaps 0 coverage 130/130`.

## Reading the accepted file

`docs/analog/mutation-accepted-sar_switch_q.txt` follows `specs/tri/mutate/survivors.t27`: one `path:line [kind]` per
mutant with a free-text note after it; one entry covers all mutants of that kind on the line (line 154 `swap-arith`
covers two). With `--accepted` the tool exits 2 on an unaccepted survivor or hung mutant, and also on an accepted line
whose mutant is now killed, so the file cannot go stale in either direction. A moved line leaves its mutant unaccepted:
edit the functions above line 380 and the file must be regenerated. New tests go at the end of the spec for that reason.

## Not claimed

- The 18 accepted survivor keys (19 mutants; the line 154 `swap-arith` key covers two) are classified by reading, plus
  one measured pin (line 361); they are not proved equivalent by a tool.
- The equivalence at line 361 is measured at one point (0.3 V, 2 pF, mode 0): the leak is bit for bit the same after the
  extra step. At another point (a different input, load or mode) the leak could move; the mutants are then not
  equivalent there and no test of this change would kill them.
- Line 273 is accepted because no test builds a largest move equal to `QS_TOL` to the last bit of an f64; that is not a
  proof the value is unreachable, so the accepted file says "not constructed", not "equivalent". The two mutants at
  line 361 are marked "unobservable in the returned QsHold": the state differs, no field of the record shows it.
- Only `sar_switch_q.t27` was gated here. The other analog specs have a census with survivors not yet pinned: #12096
  (`sar_adc`), #12097 (`sar_kick`), #12098 (`sar_cdac`), #12099 (`sar_comparator_psp`).
