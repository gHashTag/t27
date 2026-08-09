# FORMAL_FOUNDATIONS.md — propositions, measurements, and what they do not prove

> **Standing rule for this file:** every numbered proposition states its
> **evidence class** — `PROVED` (machine-checked), `MEASURED` (reproducible
> observation over a stated domain), or `CONJECTURE`. A measurement over a
> corpus is not a theorem over all inputs, and this document keeps the two
> apart even when the stronger claim would read better.

**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Propositions established in the 2026-08-09 audit campaign

### Proposition 1 — the seal path function is injective on this corpus, and not in general

Let `Σ` be the map from spec path to seal path implemented by `seal_file_path`:

```
Σ(p) = ".trinity/seals/" ++ replace(strip_suffix(strip_prefix(p, "specs/"), ".t27"), '/', '_') ++ ".json"
```

**1a. `Σ` is injective on the current corpus.** `MEASURED`.
Enumerating all 496 specs under `specs/` yields **496 distinct images**.
Reproduce:

```bash
for f in $(find specs -name '*.t27'); do ./target/release/t27c seal-path "$f"; done | sort -u | wc -l   # 496
```

**1b. `Σ` is not injective on all path sets.** `PROVED` (counterexample).
Because `_` is a legal character inside a path component, flattening `/` to `_`
cannot be injective. Witness:

```
Σ("specs/a_b/c.t27") = .trinity/seals/a_b_c.json
Σ("specs/a/b_c.t27") = .trinity/seals/a_b_c.json
```

Pinned by `seal_path_tests::flattening_is_not_injective_in_general`, which
asserts the collision **holds** — so if the encoding is ever changed, that test
fails and forces this document to be revisited.

**1c. The residual risk is contained at write time, not by the encoding.**
`seal --save` refuses to write a seal whose recorded `spec_path` differs from
the spec being sealed. A future collision therefore surfaces as a loud error
rather than silent data loss. This is the substantive lesson: **a partial
invariant plus a guard at the mutation site is stronger than a total invariant
nobody re-checks.** The predecessor scheme `<parent-dir>_<module-name>` had no
guard, and its collision (`feed_forward.t27` and `feed_forward_network.t27`,
both declaring `module FeedForward;`) silently destroyed one seal and left that
spec unverifiable for months.

**Corollary 1d.** `Σ` requires no parse and no compile — it is a pure function
of the path string. This is why the pre-commit hook can resolve a seal location
without building the compiler, and why two independent derivations of the same
path (a bash `basename` guess and the compiler's rule) collapsed into one.

---

### Proposition 2 — the open-source Yosys frontend cannot consume concurrent SVA

`MEASURED` on Yosys 0.63 (`70a11c6`, macOS arm64).

The `--with-sva` bundle advertises formal-friendliness. Measured support:

| Construct | Example | Yosys `read_verilog -sv -formal` |
|---|---|---|
| Named property block | `property p; @(posedge clk) a \|-> b; endproperty` | **rejected** — `syntax error, unexpected TOK_PROPERTY` |
| Inline concurrent assertion | `assert property (@(posedge clk) a \|-> b);` | **rejected** — `syntax error, unexpected '@'` |
| Immediate assertion in `always` | `always @(posedge clk) assert (!a \|\| b);` | **accepted** |

**Consequence 2a.** Since SymbiYosys uses Yosys as its frontend, the emitted
SVA could never have been checked by the open-source formal flow — with or
without the file-scope bug fixed in this campaign. A `.sby` harness over these
files would have failed at parse. Consuming this SVA would require a Verific-enabled Yosys; **`sv2v` does not
work for this — see Prop 5, it deletes assertions.** The constructive route is
Prop 6: emit the subset Yosys accepts.

**Consequence 2b.** The file-scope defect was real and independent: SystemVerilog
forbids `property` outside a module/interface/checker, and the emitter wrote it
bare. That is now fixed (properties are wrapped in a `bind`-able
`module behavior_sva_v2` whose ports are the referenced signals), which is
necessary for any conformant tool — but **not sufficient for Yosys**, per 2a.

**Consequence 2c.** The bundle contains exactly **one** assertion in synthesised
RTL (`multilayer_sequencer.sv`); the rest live in the separate property file.
"Formal-friendly" describes the emitter's intent, not a checked property of the
design.

---

### Proposition 3 — a verified formal-proof pipeline using only Yosys

`MEASURED`. No SymbiYosys required. For immediate assertions (Prop 2), this
sequence both **proves** true properties and **refutes** false ones:

```
read_verilog -sv -formal <file>
prep -top <top>
async2sync                 # $check cells are edge-triggered; lowering needs this first
chformal -lower            # modern Yosys emits $check; legacy `sat` cannot model it
sat -verify -prove-asserts -seq <N> -tempinduct
```

Validated in both directions, which is the part that matters — a pipeline that
only ever reports success is indistinguishable from one that checks nothing:

- true property → exit **0**
- false property → exit **1**, `Called with -verify and proof did fail!`

Omitting `async2sync` gives `Cannot lower edge triggered $check cell`; omitting
`chformal -lower` gives `No SAT model available for cell $check`. Both were
encountered and are recorded here so the next attempt does not rediscover them.

---

### Proposition 4 — conformance payload classification

`MEASURED`. Of 101 files in `conformance/`: **88** carry vectors, **5** are
measured reports, **8** are schema definitions, **0** are empty. The prior
validator reported "43 valid, 58 empty" because it resolved payloads with
`.as_array()` only, while the corpus stores vectors both as arrays and as
objects. **A count is a claim about a predicate, and the predicate was wrong.**

---

### Proposition 5 — `sv2v` cannot rescue the SVA: it deletes it

`MEASURED` on sv2v 0.0.13.

The obvious repair for Prop 2 is to preprocess SystemVerilog into Verilog-2005
with [sv2v](https://github.com/zachjs/sv2v) before handing it to Yosys. It does
not work, and the failure mode is the dangerous kind.

sv2v's own README states: *"Assertions are also supported, but are simply
dropped during conversion."* Confirmed directly — input a module containing a
`property` block and an `assert property`, and the output contains **zero**
assertions:

```
$ sv2v sva_in.sv > sva_out.v ; echo $?
0
$ grep -c assert sva_out.v
0
```

**5a. The exit code is 0 and there is no warning.** A pipeline
`sv2v → yosys → sby` would therefore run to completion, report success, and
prove **nothing** — there would be no properties left to violate. This is
strictly worse than the current state, where the flow fails loudly at parse.

**5b.** sv2v also does not support the `bind` keyword, which is the mechanism
the module-wrapped SVA of Prop 2b relies on.

**Conclusion.** sv2v is not a path to a checkable SVA flow here. A green formal
run over an empty property set is the CI-theater failure of Conclusion 1
wearing a real tool's name.

---

### Proposition 6 — the property set *is* checkable, in the immediate-assertion subset

`MEASURED`. Rather than translate the emitted form, emit the form Yosys accepts.
`t27c gen-behavior-sva-yosys` produces immediate assertions:

| Behavior form | Immediate translation | Status |
|---|---|---|
| `a \|-> b` | `assert (!(a) \|\| (b))` | translated |
| `a \|-> ##N b` | `assert (!($past(a, N)) \|\| (b))` | translated |
| `a \|-> s_eventually b` | — | **not expressible**; reported, not dropped |

**6a. Yosys reads it and the assertions survive into the netlist.**
`read_verilog -sv -formal` exits 0 (the `property` form does not), and
`stat` reports **2 `$check` cells** for a two-assertion module — the contrast
with Prop 5 that matters: the properties are still there to be violated.

**6b. The prover actively refutes.** Run over free inputs, the pipeline of
Prop 3 returns `Called with -verify and proof did fail!`, because
`running && !full` is reachable. An engine that reports success on an
unconstrained module would be evidence of nothing.

**6c. The liveness gap is reported, not hidden.** `s_eventually` has no
immediate form — an immediate assertion evaluates in a single cycle. Those
behaviors are listed on stderr and in a `NOT TRANSLATED` comment inside the
generated file, so the artefact states its own coverage. Emitting a silently
smaller property set would repeat Conclusion 1: a gate going green over a
domain nobody was told had shrunk.

**6d. Guard correctness.** The delayed form guards on `rst_n && $past(rst_n)`,
not `rst_n` alone. Guarding on the current cycle only lets the assertion fire
one cycle after reset, when the antecedent's history predates the reset. The
prover produced that counterexample during development and was right.

---

### Proposition 7 — a lost-interrupt race in `interrupt_controller`, found and fixed by proof

`PROVED` (machine-checked, Yosys 0.63). **This is the first real hardware defect
this campaign's formal work has found, and it was found by the prover, not by
reading.**

The emitted RTL latched three interrupt sources and cleared on read as four
independent non-blocking assignments:

```verilog
if (inference_done) irq_status[0] <= 1'b1;
if (dma_done)       irq_status[1] <= 1'b1;
if (error)          irq_status[2] <= 1'b1;
if (status_read)    irq_status     <= 3'b000;  // Clear on read
```

Non-blocking assignments in one `always` block resolve last-write-wins, so a
`status_read` concurrent with an event **discards that event**.

**7a. The refutation is discriminating.** Two properties differing only by the
guard `!$past(status_read)`:

| Property | Before fix |
|---|---|
| `$past(inference_done) && !$past(status_read) \|-> irq_status[0]` | **PROVED** |
| `$past(inference_done) \|-> irq_status[0]` | **REFUTED** |

The difference isolates the cause to the concurrent read exactly.

**7b. The mechanism was then confirmed positively**, which is stronger than a
counterexample. This holds on every reachable state:

```
$past(inference_done) && $past(status_read) |-> irq_status[0] == 0     PROVED
```

Not "the event can be lost" — the event **is always** lost. A host servicing an
interrupt would silently drop any event arriving in the same cycle as its status
read, the classic read-clear race, in a completion-signalling path.

**7c. The fix**: apply the clear to the *previous* value and OR this cycle's
sources on top, so clear-on-read survives without being able to discard a
simultaneous event.

```verilog
irq_status <= (status_read ? 3'b000 : irq_status)
            | {error, dma_done, inference_done};
```

**7d. All six properties now prove**, including clear-on-read
(`a_read_clears`) — the fix does not trade one behaviour for another. The
harness is checked in at `formal/interrupt_controller_props.sv` and validated
in both directions: it proves against the fixed RTL and **refutes against the
old RTL**, so it is a regression witness rather than a decoration.

**7e. Two unit tests had pinned the bug in place.** `each_source_latches_its_bit`
and `status_read_clears_latch` asserted the *literal text* of the buggy chain.
They passed for exactly as long as the race existed and would have failed the
moment it was fixed. **A test that asserts the shape of an implementation
cannot notice that the implementation is wrong.** Both now assert reachable
behaviour, with the formal harness carrying the real proof.

---

### Proposition 8 — `axi_lite_slave` accepted more transactions than it could answer

`PROVED` (machine-checked). Second real defect found by pointing the prover at
generated RTL, and the second one a passing unit test had been holding in place.

`s_axi_awready`, `s_axi_wready` and `s_axi_arready` were asserted at reset and
**never deasserted**. The module has a single `bvalid`/`bresp` register and a
single `rvalid`/`rdata` register, so it can owe at most one response per
channel. Accepting a second transaction while the first response is
unacknowledged merges two transactions into one response beat, and an AXI
master waits forever for the beat that never comes.

**8a. Formalised as a transaction balance**, which is stronger than a
handshake-shape check:

```verilog
outstanding_w <= outstanding_w + (awvalid && wvalid && awready && wready)
                                - (bvalid && bready);
assert (outstanding_w <= 1);
```

| Property | Old RTL | Fixed RTL |
|---|---|---|
| `a_one_outstanding_write` | **REFUTED** | PROVED |
| `a_one_outstanding_read` | **REFUTED** | PROVED |
| `a_no_write_accept_while_pending` | **REFUTED** | PROVED |
| `a_bvalid_stable` / `a_rvalid_stable` | PROVED | PROVED |
| `a_sanity` (tautology) | PROVED | PROVED |

Both channels had the identical defect. AXI VALID-stability was never violated —
the bug is not that responses are malformed, it is that there are **too few of
them**.

**8b. The fix** releases `ready` only on the response handshake and drops it on
accept, bounding each channel to one outstanding transaction. It costs one cycle
of throughput per transaction and is what the single-register design implies.

**8c. One refutation was an artifact, and separating it mattered.**
`bresp == 2'b00` came back REFUTED under `-tempinduct`, even though `bresp` is
only ever assigned `2'b00`. Temporal induction may begin in an **unreachable**
state where `bresp` holds garbage. Re-run from a reachable start
(`-set-init-zero`) it **PROVES**.

```
tempinduct (unconstrained init) -> REFUTED     # artifact
BMC from zero-init state        -> PROVED      # truth
```

The two genuine defects were refuted under **both** settings. **A refutation is
only evidence of a bug if the counterexample state is reachable** — otherwise it
is evidence about the proof method. Cross-checking every refutation against a
reachable start is the cheap discriminator, and it is what kept a false bug
report out of this document.

**8d. `a_sanity` is a deliberate tautology** carried in the harness. A property
that cannot fail, failing, means the run is not evaluating what it appears to —
the `-flatten` trap from Prop 7 surfaces exactly that way.

---

### Proposition 9 — two AXI4 master defects in `dma_controller`

`PROVED` (machine-checked). Third module checked, two more real defects, and
the fourth and fifth passing unit tests found holding a bug in place.

**9a. Burst abandonment.** `m_axi_arlen` and `m_axi_awlen` were hardwired to
`8'hFF` — 256 beats — for *every* transfer, while the FSM left `READ_DATA`
once `bytes_remaining` fell to one beat. A short transfer therefore requested
256 beats and then dropped `rready` mid-burst. **An AXI4 master may not abandon
a burst it requested.**

```
a_read_burst_not_abandoned      old: REFUTED      fixed: PROVED
```

Fixed by deriving the burst length from the bytes still owed
(`ceil(bytes/8)`, capped at 256, encoded as beats-1) and leaving `READ_DATA`
only on `rlast`, chaining another burst from an advanced address when bytes
remain. The write path had the mirror defect: `wlast` was raised when the
*transfer* ended rather than when the announced *burst* did.

**9b. Ready without valid.** `READ_ADDR` advanced on `if (m_axi_arready)`
alone. A `ready` asserted while `arvalid` was still low moved the FSM into
`READ_DATA` **having issued no address** — the master then sat ready for a
burst nobody owed it. `WRITE_ADDR` had the same shape.

```
a_rready_implies_burst          old: REFUTED      fixed: PROVED
```

Note that AXI VALID-stability (`a_arvalid_stable` and friends) **proved on the
broken design**. The defect is not a malformed handshake; it is a *missing*
one. Those properties are kept in the harness to bound what the bug was not.

**9c. Two candidate findings were rejected**, and rejecting them mattered as
much as the fixes:

| Candidate | Verdict |
|---|---|
| `zero_length_moves_nothing` | **Not a bug.** Proved on the pre-fix RTL from a reachable state. The guard added alongside the real fixes is hardening, not a repair, and is recorded as such. |
| `beats_taken <= ceil(length/8)` | **Inconclusive, not claimed.** With `rvalid` a free input, a misbehaving slave is indistinguishable from a master defect. It refuted even after the fixes, and a faithful enough slave model to settle it was not built. Recorded as an open question rather than a finding. |

**9d. Environment assumptions are part of the claim.** The `a_rready_implies_burst`
property is meaningful only with a minimal slave model (`assume (!rvalid ||
burst_active)`). Stating a master-side property without constraining the slave
proves nothing about the master. Every `assume` in a harness narrows what the
`assert` means, and the narrowing belongs in the write-up.

---

### Proposition 10 — a reusable AXI4 slave model, and one anomaly left open

`MEASURED` for 10a. **10b is deliberately left unresolved**, and saying so is
the point of this entry.

**10a. The model, and why its precondition is asserted not assumed.**
`formal/axi4_read_slave_model.sv` constrains a read slave to what AXI4 actually
requires: no unsolicited beats, `rlast` exactly on the (`arlen`+1)-th beat, and
slave-side VALID stability. It does **not** constrain `arready`, which a slave
may stall freely.

The model tracks one burst at a time, which is faithful only if the master
issues one at a time. That is checked by `a_model_precondition_single_burst`
rather than assumed. **Assuming it would let the model hide exactly the class
of defect it exists to expose**, and the distinction was not academic: the
precondition initially **refuted**. Port-only properties
(`!(arvalid && rready)`, no back-to-back AR handshakes) both **proved** on the
same RTL, which located the fault in the *model* — it cleared `burst_active`
from its own beat counter instead of from the master-visible `rlast`, so a
single disagreement latched it high forever. Keyed off `rlast`, the
precondition proves.

The sequence is the reusable part: **when a model's precondition fails, check
the same claim with properties that use only the ports of the unit under test.
If those hold, the model is wrong.**

**10b. Open anomaly — `arlen` at the address handshake.** With `length`
constrained to 8 (a single-beat transfer, so `arlen` must be 0):

```
assert (!(arvalid && arready) || arlen == 8'd0)     REFUTED
```

Hand-tracing the RTL says this should hold: `m_axi_arlen <= burst_len` and
`m_axi_arvalid <= 1'b1` are assigned on the same cycle in `READ_ADDR`, from a
`bytes_remaining` that the IDLE branch commits in the same non-blocking group
as the state change.

**The refutation and the hand-trace disagree, and this entry does not claim
which is right.** It is recorded as an anomaly, not a defect and not an
artifact. Consequently the over-read property
(`beats_taken <= ceil(length/8)`) also remains **open** — it refutes, but a
harness that produces one unexplained result cannot be trusted to settle a
second.

The alternative was to pick the reading that made a tidier story. Two waves ago
an unreachable-state refutation was nearly filed as a bug (Prop. 8c); the cost
of a false finding is that someone acts on it. **An anomaly with a name and a
reproduction is more useful than a confident answer that might be wrong.**

Reproduce:

```bash
yosys -p "read_verilog -sv -formal <bundle>/dma_controller.sv \
          formal/axi4_read_slave_model.sv <harness>; \
          prep -top chk -flatten; async2sync; chformal -lower; \
          sat -verify -prove-asserts -seq 16 -set-init-zero"
```

---

### Proposition 11 — the anomaly was the harness, and the cause was an opt-in flag

`PROVED`. Prop. 10 closed with `arlen == 0` refuting at the address handshake
while a hand-trace said it must hold, recorded as an unexplained anomaly.
It is now explained, and the explanation generalises past this repository.

**11a. Yosys's `sat` ignores `$assume` cells unless `-set-assumes` is passed.**
It is opt-in and silent. A harness without the flag still runs, still prints
`PROVED` or `REFUTED`, and every `assume` in it is inert — so a property meant
to hold *given a compliant environment* is being checked against an arbitrary
one. Demonstrated with a two-line module: `assume (1'b0)` alongside
`assert (a == !a)`.

```
without -set-assumes -> REFUTED   (the false assertion is reachable)
with    -set-assumes -> PROVED    (vacuously, as an unsatisfiable assumption requires)
```

**11b. That fully accounts for the anomaly.** With the flag, a readable
counterexample became available (single module, no `-flatten`, so signal names
survive). The trace shows the environment driving `m_axi_rvalid` **without ever
asserting `rlast`**: `bytes_remaining` walks 8 → 0 → −8 → −24, `beats_owed`
becomes enormous, and `burst_len` saturates to `8'hFF`. The 256-beat request
was real, and it required a **non-compliant slave**.

**11c. Under a compliant slave the property holds.** Re-run with the AXI4
contract active — no unsolicited beats, `rlast` exactly on the last beat of the
burst:

```
a_arlen_zero      PROVED
a_no_underflow    PROVED
```

So: **not a defect.** The design is correct against the protocol contract, and
the earlier refutation was a harness that had never applied its own constraints.

**11d. Audit of everything that came before.** All three checked-in harnesses
were re-run with and without the flag. All prove **both ways**, so the four RTL
defects of Props. 7–9 are unaffected — those properties never depended on an
assumption. Only the inconclusive Prop. 10 investigation was affected, and it
is now resolved.

**11e. A defensive clamp was written, then reverted.** Forcing
`beats_burst >= 1` would stop the `beats_owed == 0` wrap. But 11c *proves*
that state unreachable under contract, and the non-compliant case underflows to
a **large** `bytes_remaining`, where `arlen = 255` is arithmetically correct
rather than a wrap — so the clamp fixed nothing reachable while adding a branch.
Full immunity to a lawless slave is not available anyway: the only way to stop
consuming early is to abandon the burst, which is itself the violation fixed in
Prop. 9a. **Proving code unreachable is a reason to delete it, not to add it.**

**11f. The flow now verifies itself.** `formal/assume_liveness_check.sv` is
proved first in CI; it passes only when assumptions are live. This is the
tautology instrument of Prop. 7 turned on the tool instead of the design — the
recurring shape being that a checker which cannot fail, and a checker whose
constraints do nothing, are the same defect wearing different clothes.

---

### Proposition 12 — the 21 properties are non-vacuous, and the check is now permanent

`MEASURED`. Prop. 11 found constraints that did nothing. Vacuity is its mirror:
a property that **passes because the interesting case never happens**. Neither
appears as a failure; both make a green run worthless.

Two levels were checked.

**12a. Guard reachability.** For each `G |-> P`, the assertion body was replaced
with `assert (1'b0)` under the same guard. That run **proves iff `G` is
unreachable** — a precise vacuity oracle needing no `cover` support. All other
assertions in the file were neutralised to `assert (1'b1)` so each result speaks
about one guard only.

```
19 properties checked      reachable 19      vacuous 0
```

*(19 rather than 21: two `a_sanity` tautologies are deliberately unconditional
and have no guard to reach.)*

**12b. Interesting-case reachability.** Guard reachability is necessary, not
sufficient: `assert (!A || B)` is trivially satisfied whenever `A` is false, so
a property can be evaluated constantly and still test nothing. Six cases that
the properties exist to cover were probed by asserting their negation — a
**refutation is the witness** that the case occurs:

| Case | Property it gives teeth to | Result |
|---|---|---|
| `irq_enable == 0 && irq_status != 0` | `a_mask_suppresses` | REACHABLE |
| `inference_done && status_read` | `a_event_never_lost` | REACHABLE |
| `outstanding_w == 1` | `a_one_outstanding_write` | REACHABLE |
| `bvalid && !bready` | `a_no_write_accept_while_pending` | REACHABLE |
| `rvalid && rready && !rlast` (multi-beat burst) | `a_read_burst_not_abandoned` | REACHABLE |
| a burst active at all | `a_rready_implies_burst` | REACHABLE |

The fifth matters most: `a_read_burst_not_abandoned` is the regression witness
for Prop. 9a, and on single-beat bursts alone it would be vacuous. It is not.

**12c. Made permanent.** `formal/witnesses.sv` carries these as standalone
harnesses; CI runs each **expecting refutation**. A witness that starts proving
means the case became unreachable and the property guarding it is now free.

The pair of gates now reads: `$check`-cell counts prove the properties **exist**
(Prop. 6), witnesses prove they **bite**. Together with Prop. 11's
assumption-liveness check, the flow now verifies three distinct ways of passing
while testing nothing — **which is the same defect the whole campaign began with,
found once in a shell gate, once in a CI `echo`, and now twice inside the prover
itself.**

---

### Proposition 13 — two zero-count non-terminations, in a family where two siblings guard

`PROVED`. Fourth and fifth modules checked, fifth and sixth real defects. Both
are the same shape, and the shape is now a pattern worth naming.

**13a. `layer_sequencer` with `num_neurons == 0`.** The terminator is
`neuron_id == num_neurons - 1`, which compares against `16'hFFFF` when the count
is zero. It never matches, so the sequencer emits `valid` for neuron indices
0, 1, 2, … indefinitely.

**13b. `weight_prefetch_ctrl` with `num_words == 0`.** `words_remaining`
underflows to `16'hFFFF` on the first beat and `words_remaining == 1` never
matches, so the controller writes BRAM indefinitely — past the 4096-entry
buffer and past anything the caller asked for.

**13c. Stated as bounds, not as liveness.** Non-termination is a liveness
property and an immediate assertion cannot express it (Prop. 6). Both were
instead written as safety bounds that the runaway violates:

```
valid   |-> neuron_id < num_neurons        REFUTED -> PROVED
writes  <= num_words   (while active)      REFUTED -> PROVED
```

**A runaway loop usually has a safety shadow** — some counter or index that
leaves its legitimate range — and the shadow is checkable where the liveness
property is not.

**13d. The discriminating evidence was already in the module.**
`a_chunk_in_range` **proved on the same RTL that refuted `a_neuron_in_range`**:
`layer_sequencer` already contained `if (num_chunks == 0) state <= DONE_ST` and
had simply not done the same for neurons. `multilayer_sequencer` guards
`num_layers > 0`. `dma_controller` gained its `length != 0` guard in Prop. 9.
**Two siblings in the family guard the zero case and two did not** — which
settles the reading as oversight rather than a deliberate contract, without
needing to ask anyone.

**13e. Isolation.** Assuming the count non-zero, both properties prove. The
refutations are precisely the zero case and nothing else.

**13f. A ninth text-pinning test.** `prefetch_fsm_states_present` asserted
`IDLE: if (start_prefetch) begin` — the unguarded form. Every RTL defect found
in this campaign, now six of six, had a passing unit test holding it in place.

---

### Proposition 14 — the first multi-module proof, and a property that did not bite

`PROVED`. Every property in Props. 7–13 is **module-scoped**, because until the
datapath was wired there was no system behaviour to state a property about
(`BITNET_V2_POSITION.md` §2.3). `bitnet_engine_top` now instantiates the MAC and
both BRAMs, so integration properties became possible — and writing them
surfaced a failure mode the earlier waves had not met.

**14a. The hazard.** `weight_bram` reads with **one cycle of latency**
(`rd_data <= mem[rd_addr]`). Feeding the MAC straight from `layer_sequencer`
would pair chunk *N*'s control with chunk *N−1*'s weights. Every module-level
property would still pass — the sequencer is correct, the BRAM is correct, the
MAC is correct. Only the composition is wrong. The top now delays
`valid`/`first`/`last` by one cycle to meet the weight word.

**14b. A true property that constrained nothing.** The first attempt asserted

```verilog
a_mac_control_aligned: assert (mac_valid_q == $past(layer_valid));
```

That is true of the skew *registers* regardless of what the MAC is connected
to. Rewiring `valid_in` straight to `layer_valid` — reintroducing the exact
hazard — left it **PROVING**.

**A property about a signal is not a property about the wire it feeds.** The
repair states it on the MAC's own output, which pins down which control it
actually consumed:

```verilog
a_mac_consumed_skewed_control:
    assert (mac_valid_out == ($past(mac_valid_q) && $past(mac_last_q)));
```

| Build | Old property | New property |
|---|---|---|
| skew present (correct) | PROVED | **PROVED** |
| skew removed (the hazard) | **PROVED** ← useless | **REFUTED** |

This was caught only by the standing rule from Prop. 7 — *validate a regression
harness against the broken version*. Without that step the wave would have
shipped eight green integration properties, one of which certified nothing.

**14c. Two mechanical notes.** `sat` cannot model `$mem_v2`, so the proof runs
with `chparam -set DEPTH 4 weight_bram` and `memory_map`; the properties do not
read memory contents, so shrinking the array is sound. The properties live
inside the module under `` `ifdef FORMAL `` because the alignment they check is
internal and `sat` requires one flattened module, which mangles the names a
wrapper would need.

**14d. `threshold` is now connected.** It was declared and never referenced
(`BITNET_V2_POSITION.md` §2.3); it now gates `neuron_out`, and
`a_threshold_gates` holds it to that.

---

### Proposition 15 — the layer boundary exists, and `2'b11` is now unreachable

`PROVED`. Step 2 of `BITNET_V2_POSITION.md` §4. The bundle had **no module at
the layer boundary at all**: `pipeline_stage2_compute` emitted `signed [15:0]`,
the next layer consumed `[53:0]` packed trits, and nothing converted between
them. `t27c gen-activation-requant` fills that gap.

**15a. The reserved code.** The trit stdlib defines `2'b00 = -1`, `2'b01 = 0`,
`2'b10 = +1`, and **`2'b11` as reserved/invalid** — a mux fall-through, with no
error path anywhere. A requantizer that could emit it would corrupt every
downstream `trit27_*` primitive silently. `a_trit_never_invalid` forbids it, and
the emitter test forbids *assigning* it in the generated text.

**15b. A negative threshold makes the branches overlap.** `acc >= threshold` and
`acc <= -threshold` are both true when `threshold < 0`. Written as parallel
comparisons that is a don't-care; written as a **priority chain** the positive
branch wins and the output stays legal for every input, without trusting the
host to program a sane value. **Prefer a total function over a documented
precondition when the cost is one ternary operator.**

**15c. Validated against two deliberate breaks**, per the standing rule:

| Variant | Result |
|---|---|
| correct | PROVED |
| dead-zone emits `2'b11` | **REFUTED** (`a_trit_never_invalid`) |
| priority order reversed | **REFUTED** (`a_positive_branch`) |

**15d. The design fork now has an address.** `BITNET_V2_POSITION.md` recorded
that this line quantises activations to ternary — more aggressive than any
published BitNet variant — and that the choice was *implicit in the absence of a
requantizer*. It is now explicit in one output port. A 4-bit variant changes
`trit [1:0]` to `act [3:0]` and swaps the dead-zone for a scale-and-round;
nothing else in the datapath moves. **An unmade decision with no interface is
untrackable; the same decision with an interface is a diff.**

**15e. Two tests of mine were too broad and failed on their own subject.**
`never_emits_the_reserved_code` asserted `!contains("2'b11")` over the whole
emitted text — which fails on the comment explaining why `2'b11` is forbidden,
and on the assertion that forbids it. Narrowed to assignment contexts. This is
the third instance of the same slip in this campaign (`8'hFF` in Prop. 9,
`FORMAT-SPEC` in Prop. 4): **a substring ban catches the documentation that
justifies the ban.**

**15f. Count-named tests were renamed to invariant-named ones.** Adding one file
broke `bundle_order_has_twelve_entries`, `build_sv_entries_returns_eleven_files`
and two positional lookups (`entries[9]`, `entries[10]`). They now assert
`BUNDLE_ORDER.len() == BUNDLE_FILE_COUNT` and look up by filename. **A test
whose name contains a number has to be renamed every time the system grows,
which is a strong hint it is asserting the wrong thing.**

---

### Proposition 16 — the loop closes, and a controller whose decision nobody read

`PROVED`. Step 3: the requantizer's packed word now feeds back as the next
layer's activations, so the engine can actually iterate.

**16a. `use_buffer_a` was dead.** `double_buffer_ctrl` computes the ping-pong
decision and `bitnet_engine_top` connected it to a wire — and **never consumed
it**. The single activation BRAM had `wr_en` tied to `1'b0`. So the controller
was correct, its output was wired, and nothing acted on it: the engine could
run one layer and had no path from a layer's output to the next layer's input.

Grep count of `use_buffer_a` in the top before this wave: **2** — the
declaration and the port connection. **A signal that appears exactly twice is
connected but unused, and that is invisible to any per-module check.**

**16b. The invariant.** Reading and writing the same buffer in one layer lets a
neuron consume activations that same layer just produced. Both BRAMs are
correct, the controller is correct, and the composition would be wrong:

```verilog
a_no_read_write_same:
    assert (!(use_buffer_a && wr_en_a) && !(!use_buffer_a && wr_en_b));
```

Validated by inverting the ping-pong — writing the buffer being read:

| Variant | Result |
|---|---|
| correct | PROVED |
| ping-pong inverted | **REFUTED** |

**16c. The write address is a word counter, not a neuron counter.** The
requantizer emits one packed word per 27 neurons, so `buf_write_addr` — which
counts neurons — is the wrong address by a factor of 27. A dedicated
`act_wr_word` counter, reset at `layer_start`, is the right one. **A signal
named for what it addresses is not necessarily the address you need; check the
rate.**

**16d. This is the third integration defect class in three waves**, none of
which any module-level property could reach: a latency skew (Prop. 14), an
absent stage (Prop. 15), and a dead control signal (here). All three were found
by wiring things together and asserting across the seam — which is only
possible once there is a seam.

---

### Proposition 17 — the host path is wired, and one property is left open on purpose

`MEASURED`. `weight_prefetch_ctrl` and `interrupt_controller` are now
instantiated: **10 instances, 8 of 10 modules**, and the tie-offs
`mem_addr = 32'd0`, `mem_rd_en = 1'b0`, `prefetch_done = 1'b1` are gone.

**17a. Weights were never loaded.** `wmem`'s write port was also tied off
(`wr_en(1'b0)`) — so alongside the dead `use_buffer_a` of Prop. 16, **neither
memory in the datapath was ever written**. The prefetch controller now streams
from the external port into the weight BRAM, and the external port is driven by
its AXI read channel.

**17b. An open, reproduced anomaly — deliberately not asserted.** A single
weight BRAM is only safe if prefetch never writes an address the MAC is
reading, and `multilayer_sequencer` keeps `PREFETCH` and `LAYER_RUN` in
separate states, which should make that impossible. It does not hold:

| Property | Result |
|---|---|
| `!(pf_bram_we && mac_valid_q)` | **REFUTED** |
| `!(pf_bram_we && mac_valid_q && pf_bram_addr == chunk_addr)` | **REFUTED** |

Both still refute with a memory model constraining `mem_rd_valid` to follow
`mem_rd_en`, so this is **not** an unconstrained-environment artefact
(cf. Prop. 11).

It is recorded in the RTL as a comment and in this document, and **not
asserted**. Three options were available: ship the failing assertion, weaken it
until it passes, or record the gap. The first breaks CI for everyone; the
second is the vacuity failure of Prop. 12 committed on purpose. **A property
you cannot yet prove is a finding, not a defect in the property — and the
honest place for it is documentation, not a weakened assert.**

**17c. A tenth text-pinning test, and its name was the giveaway again.**
`external_memory_outputs_tied_off` asserted `assign mem_addr = 32'd0;` — the
tie-off *as the contract*, exactly like `dma_burst_length_is_max` in Prop. 9.
Renamed to `external_memory_port_is_driven_by_prefetch`. **Ten such tests across
this campaign, and every RTL defect found had one.**

---

### Proposition 18 — the open anomaly of Prop. 17, closed: a stale flag and a missing handshake

`PROVED`. Prop. 17 recorded a reproduced but uncharacterised refutation:
prefetch could write the weight BRAM while the MAC was reading it, despite
`multilayer_sequencer` keeping `PREFETCH` and `LAYER_RUN` in separate states.
**Two independent defects, in two different modules.**

**18a. Getting a legible trace was the whole problem.** Top-level signal names
survive `-flatten`, so `sat ... -show pf_bram_we -show mac_valid_q ...` prints a
readable cycle table where a VCD gave only mangled internals. The table made
both causes visible in one reading, after two waves of not being able to see
them.

**18b. Defect one — a stale completion flag.** `weight_prefetch_ctrl` sets
`prefetch_done` in `DONE_ST` and clears it **only** at reset or inside the
`start_prefetch && num_words != 0` guard. After a completed prefetch it stays
high, so the next requester sees the *previous* transaction's completion.

Fixed by clearing on **request** rather than on successful start, with a
zero-word request routed straight to `DONE_ST` so clearing the flag cannot
strand the requester:

```verilog
IDLE: if (start_prefetch) begin
    prefetch_done <= 1'b0;
    if (num_words != 16'd0) begin ... end else state <= DONE_ST;
end
```

**18c. Defect two — a missing request/acknowledge.** That alone did not fix it.
The second trace showed `layer_start` one cycle after `start_prefetch`:
`multilayer_sequencer` tests `prefetch_done` in the **first** cycle of
`PREFETCH`, before the controller has had a cycle to clear it. A level-triggered
handshake cannot distinguish "done already" from "done still".

Fixed with an explicit acknowledgement — the requester waits to observe the flag
*low* before accepting it high:

```verilog
if (!prefetch_done) pf_ack <= 1'b1;
if (pf_ack && prefetch_done) begin ... end
```

**18d. Why one fix was not enough, and why that matters.** After 18b the
property still refuted, and the temptation at that point is to conclude the
first diagnosis was wrong. It was not — it was **incomplete**. Two modules each
contributed a defect that the other's correctness would have masked. **A
refutation that survives a correct fix means another cause, not a wrong
diagnosis; re-read the trace rather than reverting.**

**18e. The recorded gap paid off.** Prop. 17 chose to document rather than
weaken. Had the property been softened to pass, both defects would have shipped
under a green check, and the trace that identified them would never have been
taken.

---

### Proposition 19 — the host aperture is wired; config is CSRs, not ports

`PROVED`. `axi_lite_slave` was the **last emitted module never instantiated**:
verified in isolation — its lost-write-response defect was found and fixed in
Prop. 8 — and unreachable from the top. It is now the engine's control
aperture: **9 of 10 modules, 11 instances**.

**19a. Config stopped being a port bundle.** `start`, `num_layers`,
`neurons_per_layer`, `chunks_per_neuron`, `threshold` and `weight_words` were
top-level inputs, which meant every instantiator had to synthesise a
configuration bus of its own. They are now CSRs a host writes over AXI-Lite.
`weight_words` is packed into `reg_chunks[31:16]` because the 16-word aperture
has no spare register — recorded in the emitted header rather than left for a
reader to discover.

**19b. Two properties guard against a decorative instantiation.**

```verilog
a_start_is_ctrl_bit0:     assert (start == reg_ctrl[0]);
a_status_reflects_engine: assert (reg_status[0] == busy && reg_status[1] == done);
```

Both would hold *vacuously* if the slave were instantiated and ignored — which
is exactly how `use_buffer_a` sat dead for four waves (Prop. 16). **Wiring a
module is not the same as using it, and the property has to name the
connection.**

**19c. What remains, stated precisely.** `dma_controller` is the one module
still standalone. Four of its defects were fixed in Props. 8–9 and none of that
is reachable from the top. The honest count is **9 of 10**, not 10 of 10.

**19d. Three tests named for the old interface.** `control_ports_present`,
`top_control_ports_present` and two `reg [31:0] cycles;` string matches broke on
a *correct* interface change. Renamed to
`host_aperture_replaces_config_ports`, and they now assert the *absence* of the
old ports as well as the presence of the new ones — a rename plus an inversion,
because the interesting claim moved.

---

### Proposition 20 — every emitted block is wired; one interlock is open

`MEASURED`. `dma_controller` was the last standalone module. With it
instantiated the count reaches **10 of 10, 12 instances** — every emitted block
is reachable from the top, closing the emitted-vs-integrated gap that
`BITNET_V2_POSITION.md` §3c opened.

**20a. It closed a functional gap, not just an integration one.** The activation
buffers were written *only* by the requantizer — that is, only from the
**previous** layer. Layer 0 therefore read uninitialised memory, and there was
no path for input data to enter the engine at all. The DMA fills the buffer the
first layer will read.

**20b. Adding a second writer invalidated an existing invariant's scope.**
`a_no_read_write_same` (Prop. 16) forbids writing the buffer being read. That
was correct when the requantizer was the only writer. The DMA's intent is the
**opposite** — it deliberately fills the buffer about to be read, because that
is where layer 0's input belongs. Left unscoped, the invariant made a correct
DMA look like a violation.

Now scoped to the requantizer path (`if (rst_n && !dma_local_we)`).
**An invariant written against one producer usually encodes an assumption about
how many producers there are, and adding a second is the moment to re-read it —
not to weaken it, but to state the domain it was always about.**

**20c. Open, recorded not asserted.**

```
assert (!(dma_local_we && mac_valid_q))     REFUTED
```

`reg_ctrl` is host-writable at any time, so a host writing `ctrl = 3` requests
an inference and a DMA in the same cycle, and the DMA loads into the buffer the
MAC is reading. An interlock was added — `.start(reg_ctrl[1] && !reg_ctrl[0] &&
!busy)` — and it **narrows without closing**: `busy` is
`(current_layer != 0) || layer_start`, false during the very first layer, so a
residual window remains.

Recorded rather than weakened, per Prop. 17. The interlock is kept because it is
a genuine improvement, and the property is kept out of CI because asserting it
would certify an interlock that does not hold. The likely fix is a real
`inference_active` signal held from start to done, rather than a decode of
`current_layer` — **`busy` is a derived proxy, and Prop. 12's lesson about
proxies applies to design signals as much as to gates.**

---

### Proposition 21 — `busy` becomes a state; the interlock is narrowed twice and still open

`MEASURED`. Two real fixes landed against the open property of Prop. 20, and
**neither was sufficient** — which is the finding.

**21a. `busy` was a decode, not a state.** It was
`(current_layer != 0) || layer_start` — **false throughout the entire first
layer**, so any interlock keyed off it had a hole exactly where the first
inference happens. It is now a register set at `start` and cleared at `done`.

This is the proxy failure of Prop. 12 arriving in RTL rather than in CI. **A
counter comparison that is usually equivalent to "running" is not the same
object as a flag someone maintains**, and the difference appears precisely at
the boundaries where interlocks matter. Before keying safety logic off an
existing signal, read its definition rather than its name.

**21b. The interlock guarded one direction of a mutual exclusion.**
`.start(reg_ctrl[1] && !reg_ctrl[0] && !busy)` blocked a DMA during inference,
but nothing blocked an inference during a DMA: a host writing `ctrl = 2` then
`ctrl = 3` had compute running against a buffer the DMA was still filling.
Now symmetric — `start = reg_ctrl[0] && !dma_busy` as well. **An interlock that
names only one of two mutually exclusive activities is half an interlock.**

**21c. A property of mine encoded the pre-interlock semantics.**
`a_start_is_ctrl_bit0` (Prop. 19) asserted `start == reg_ctrl[0]` — which the
interlock deliberately breaks. Rather than delete it, it was **split**: the
general form now says `start == (reg_ctrl[0] && !dma_busy)`, and the original
is kept under `if (!dma_busy)`, so the interlock remains the *only* thing that
may suppress a start. **When a change invalidates a property, ask whether it
becomes two properties — the new behaviour, and the guarantee that nothing else
changed.**

**21d. Still open, and isolated.** `!(dma_local_we && mac_valid_q)` refutes.
Neutralising it alone makes every other property in the module pass, so it is
the sole remaining failure. The residual window is a timing relationship
between `dma_busy` and `local_we` rather than a missing guard. Recorded, not
weakened, for the third time (Props. 17, 20) — and the two waves where that
discipline paid off (Prop. 18) are why.

**21e. An eleventh text-pinning test.**
`top_busy_from_current_layer_or_layer_start` — the *name* encoded the decode as
the contract, exactly like `dma_burst_length_is_max` and
`external_memory_outputs_tied_off`.

---

### Proposition 22 — why no top-level gate can close this, and where the fix belongs

`MEASURED`. Three successive narrowings landed against one property, each real
and each insufficient. The fourth attempt produced the diagnosis instead:

1. `busy` made a state register rather than a decode of `current_layer` (Prop. 21a)
2. the interlock made symmetric in both directions (Prop. 21b)
3. the gate extended across `layer_valid`, `mac_valid_q` and `act_trit_valid`

**22a. The trace.** With `-show` on the top-level signals:

```
t   reg_ctrl  start  inference_active  dma_busy  dma_local_we  layer_valid  mac_valid_q
14        47      1                 1         0             0            0            0
15         2      0                 0         0             0            0            0
16         2      0                 0         1             0            0            0
17         2      0                 0         1             0            1            0
18         2      0                 0         1             0            1            1
19         2      0                 0         1             1            0            1   <== OVERLAP
```

At t15 the host clears `reg_ctrl[0]`; `inference_active` falls and the DMA gate
opens. At **t17 `layer_valid` rises again** — the sequencer restarted work of
its own accord.

**22b. The diagnosis.** `multilayer_sequencer` runs its own state machine and
**does not stop when the host clears the start bit.** `inference_active` tracks
a host request, not the engine's state. So the DMA gate opens while the
sequencer is mid-traversal, and the sequencer then re-raises `layer_start`.

**Quiescence is a property of the sequencer, and `bitnet_engine_top` cannot
observe it.** Gating harder at the top can only narrow the window — which is
exactly what three attempts did, each by a little.

**22c. Where the fix belongs.** `multilayer_sequencer` needs an `idle` output
(`state == IDLE`), and the interlock should key off that. That is a module
interface change, and it is deliberately **not** made here as a fourth
narrowing. Three partial fixes in a row is the signal to stop patching the
observer and change what is observable.

**22d. The general shape.** A supervisor that can be *asked* to stop is not the
same as one that *has* stopped. Any interlock built on the request rather than
the acknowledgement inherits the gap — the same request/acknowledge distinction
that Prop. 18c found in the prefetch handshake, one level up. **When a gate
keeps almost-working, suspect that the signal it reads answers a different
question than the one being asked.**

---

### Proposition 23 — the interlock closes: export quiescence, then restore the term you dropped

`PROVED`. Prop. 22 diagnosed that no top-level gate could close
`!(dma_local_we && mac_valid_q)` because quiescence lived inside
`multilayer_sequencer` and was not observable from `bitnet_engine_top`. Acting
on that diagnosis closed it — in two steps, and the second is the interesting
one.

**23a. Export the observable.** `multilayer_sequencer` gains one output,
`assign idle = (state == IDLE)`. The module that knows whether it has stopped
now says so. This is what four accumulated top-level conditions had been
approximating.

**23b. Replacing a guard is where terms get dropped.** Substituting `seq_idle`
for the old conjunction removed `!reg_ctrl[0]` along with it — and the property
still refuted. The trace showed `reg_ctrl = 35`: a host setting the inference
bit and the DMA bit **in the same write**. At that instant the sequencer *is*
idle, so `seq_idle` permits the DMA, and the inference starts alongside it.

`seq_idle` answered a different question than **one** of the old terms, not all
of them. Restored:

```verilog
.start(reg_ctrl[1] && !reg_ctrl[0] && seq_idle
       && !layer_valid && !mac_valid_q && !act_trit_valid)
```

**When replacing a compound guard with a better condition, enumerate what each
old term was for.** A new condition that subsumes three of four leaves a hole
exactly where the fourth was, and the hole is invisible because the guard now
looks principled.

**23c. Four waves, and the shape of the ending.** Props. 20–23 spent three waves
narrowing at the wrong level and one wave diagnosing. The diagnosis was worth
more than any of the narrowings, and it named a change of five lines. **Time
spent understanding why a fix does not work is not time lost from fixing it.**

All **17 integration properties** on `bitnet_engine_top` now prove, alongside 42
module-level ones.

---

### Proposition 24 — the interlocks did not stall the engine

`MEASURED`. Props. 20–23 spent four waves *adding constraints* to the reachable
state space. That is exactly the condition under which safety properties start
passing for the wrong reason — **an over-tight guard makes every safety property
hold by making the engine do nothing** — so the new properties were audited
before anything was built on top of them.

**24a. Guard reachability: 19 of 19, none vacuous.** Each integration property's
body was replaced with `assert (1'b0)` under its own guard while the rest were
neutralised to `assert (1'b1)`; that run proves iff the guard is unreachable
(the oracle from Prop. 12a). Every guard is reachable.

**24b. Liveness witnesses: the engine still runs.** Guard reachability is
necessary and not sufficient. Six probes assert that an activity is
*impossible*, so a **refutation** is the evidence it still happens:

| Probe | Expected | Result |
|---|---|---|
| DMA can start | refutes | REACHABLE |
| DMA can write local memory | refutes | REACHABLE |
| weight prefetch can write | refutes | REACHABLE |
| MAC can be active | refutes | REACHABLE |
| neuron output can fire | refutes | REACHABLE |
| **DMA and MAC concurrently active** | **proves** | **unreachable** |

The last row is the inverse and the point of the exercise: the five activities
the engine needs are all still reachable, and the one combination four waves of
interlock work was aimed at is genuinely impossible. **A safety property and a
liveness witness together say something neither says alone** — "this cannot
happen" is only interesting once "this can happen" is established for the parts.

**24c. Checked before extending, not after.** The natural next step after
Prop. 23 was a cross-layer property. Auditing first was the right order: a
cross-layer result built on a stalled engine would have proved trivially, and
the audit cost one wave against a claim that would otherwise have compounded.
**After a run of changes that constrain behaviour, re-establish that the
behaviour still exists before building on the constraint.**

Both checks are now CI steps, so an over-tight guard added later fails the
build rather than quietly greening it.

---

### Prop. 25 — the first properties that span two layers: one proves, one refutes and stays open — `PROVED` / `REFUTED`

Every property up to Prop. 24 held **inside one module or inside one layer**.
The double-buffer scheme, though, only means anything across a layer boundary:
layer *N* writes one buffer while layer *N+1* reads the other. Two properties
were written for that seam, using two formal-only probe registers that record
whether either activation buffer has ever been written:

```verilog
reg fv_wrote_a, fv_wrote_b;
always @(posedge clk or negedge rst_n)
    if (!rst_n) begin fv_wrote_a <= 1'b0; fv_wrote_b <= 1'b0; end
    else begin
        if (wr_en_a) fv_wrote_a <= 1'b1;
        if (wr_en_b) fv_wrote_b <= 1'b1;
    end
```

**25a. The ping-pong genuinely alternates — `PROVED`.**

```verilog
always @(posedge clk) if (rst_n && $past(rst_n) && $past(layer_done_pulse))
    a_buffer_alternates: assert (use_buffer_a == !$past(use_buffer_a));
```

Proved at `-seq 40` over the twelve-instance engine. This is the first result in
the repository that constrains the relationship between **two different layers**
rather than the internals of one: the buffer layer *N* wrote is the buffer layer
*N+1* reads. It is now part of the default `-DFORMAL` set, taking it to **20
integration properties, all proving**.

**25b. Layer 0 can read a buffer nothing ever wrote — `REFUTED`, and open.**

```verilog
always @(posedge clk) if (rst_n && mac_valid_q)
    a_no_read_before_write: assert (use_buffer_a ? fv_wrote_a : fv_wrote_b);
```

This refutes. Nothing in the engine requires a DMA before inference, so the host
can set `reg_ctrl[0]` on a freshly reset device and the MAC will consume an
activation buffer that has never been written. **Every module-level and
single-layer property still passes** while this happens — reading uninitialised
memory violates no local contract. It is only visible across the DMA-to-layer-0
seam, which is exactly why twenty-four propositions did not see it.

**25c. Three interlocks were tried. All three were withdrawn.** The obvious fix
is to gate `start` on evidence that the input buffer was loaded. Each attempt
failed for a different and instructive reason:

| Attempt | Result |
|---|---|
| `input_loaded` set by `dma_done` | still REFUTED — a **zero-length DMA** reaches DONE without ever asserting `dma_local_we`, satisfying its completion contract while writing nothing |
| set by `dma_local_we`, declared early | still REFUTED — and it broke an unrelated proved property (below) |
| same, with a synchronous reset | still REFUTED — **baseline** broke regardless of reset style |

The first row is the interesting one on its own: it is the **third member of a
family** this campaign keeps finding — zero neurons (Prop. 9), zero words
(Prop. 10), zero bytes here. **A zero-sized job is vacuously complete, and
completion is therefore not evidence that work was done.**

Because no attempt closed 25b and every attempt cost a proved property
elsewhere, none shipped. 25b is recorded behind its own `` `ifdef FORMAL_OPEN ``
guard, and `formal-yosys.yml` gates that **it must still refute** — so the day
someone closes it, CI goes red and says to promote it. An open finding that
cannot rot into a forgotten one.

**25d. A probe harness must establish its own baseline first — `MEASURED`.**
The liveness table in Prop. 24 reads a nonzero `yosys` exit as "the probe
refuted". While the second interlock was in the tree, the *unprobed* design
stopped proving — an obligation `async2sync` generates itself began to fail —
and **every row of the liveness table silently flipped to "refutes"**, including
the one row whose expected answer is "proves". The probes were reporting on a
failure that had nothing to do with any probe. Diagnosis took four rounds
because the harness's own verdict was untrustworthy and nothing said so.

> **A verdict harness that cannot distinguish "your property failed" from
> "something else failed" is not measuring your property.** Run the design with
> no probe and no properties first; only then is a probe verdict evidence.

This is now the `Baseline - unprobed design must prove` CI step, ahead of every
probe.

**25e. A reference above its declaration silently forks the signal.** The second
interlock read `dma_local_we` at line 125 of the emitted top; the wire is
declared at line 262. Verilog's implicit-net rule conjures a one-bit wire at
first use, so the interlock read a **different, undriven signal** with the same
name — leaving the solver free to fabricate DMA writes and refute a property
that had nothing to do with the change. No warning, no error, and the emitted
file looks correct on inspection. **In a code generator, an insertion point is a
correctness property**, not a formatting choice: emit a declaration early and an
`assign` after the signals it reads.

Reproduce all of it:

```bash
t27c gen-bitnet-bundle --output-dir build/rtl
t27c gen-trit-stdlib > build/rtl/trit_stdlib.sv
# 25a, in the default set (proves); 25b behind the open guard (refutes)
yosys -p "read_verilog -sv -formal -DFORMAL -DFORMAL_OPEN build/rtl/*.sv; \
          chparam -set DEPTH 4 weight_bram; prep -top bitnet_engine_top -flatten; \
          memory_map; async2sync; chformal -lower; \
          sat -verify -prove-asserts -seq 40 -set-init-zero -set-assumes"
```

---

## 2. Related work — verified citations

Titles fetched from each source's own metadata on 2026-08-09; none is quoted
from memory.

| Work | Title (as published) | Relevance |
|---|---|---|
| [arXiv:2402.17764](https://arxiv.org/abs/2402.17764) | *The Era of 1-bit LLMs: All Large Language Models are in 1.58 Bits* | The ternary-weight result this line's direction rests on. Motivation only — it validates no claim about t27 silicon. |
| [arXiv:2310.11453](https://arxiv.org/abs/2310.11453) | *BitNet: Scaling 1-bit Transformers for Large Language Models* | The predecessor architecture the HLS pipeline is named for. |
| [arXiv:2504.18415](https://arxiv.org/abs/2504.18415) | *BitNet v2: Native 4-bit Activations with Hadamard Transformation for 1-bit LLMs* | Current direction of the field: activation width, not weight width, is now the binding constraint. Relevant to whether a ternary-weight-only datapath is still the right target. |
| [arXiv:1811.01721](https://arxiv.org/abs/1811.01721) | *Rethinking floating point for deep learning* | Prior art for replacing IEEE-754 in ML datapaths — the closest methodological precedent for GoldenFloat, and a fair standard for what evidence such a proposal is expected to carry. |
| [arXiv:2106.10860](https://arxiv.org/abs/2106.10860) | *Multiplying Matrices Without Multiplying* | Multiplication-free matmul via lookup. Directly adjacent to `OP_LUT_NPU`'s 81-entry table and to Microsoft's T-MAC. |
| [Vericert](https://github.com/ymherklotz/vericert) | "A formally verified high-level synthesis tool based on CompCert and written in Coq." | The standard against which this repo's compiler-correctness claim must be measured — and by which it is exceeded. See [`COMPETITORS.md`](../COMPETITORS.md) §2.1. |

---

## 3. Conclusions

1. **Every quality gate audited in this campaign enforced something weaker than
   its name**, and three enforced nothing at all (`echo` statements). The
   generalisable check is cheap: for each gate, write down the property its name
   claims, then read it and write down the property it tests. The gap is the
   defect. It requires no domain knowledge and found six real ones here.

2. **Presence is not integrity.** 730 seal files existed and 0 verified. Both
   the local gate (`[[ -f ]]`) and the CI job (an echo) measured presence. The
   distinction is not pedantic: it was the difference between a four-month-stale
   provenance chain and a sound one.

3. **A clean 0% or 100% is a harness fault until disproved.** Twice this
   campaign a uniform result was an artefact — `FAIL: 496` meant *binary not
   found*, `58 empty/skipped` meant *object-shaped*. Once it was real
   (`seal 0/496`). The discipline of checking costs a minute; not checking cost
   a wrong plan that would have fabricated test vectors for 45 files that
   already had them.

4. **A single outlier after a uniform operation is signal.** 495/496 after a
   mass re-seal was not noise — it was a non-injective path function destroying
   one seal.

5. **Regenerating a measurement is repair; rewriting a baseline is a decision.**
   The coverage file was regenerated without asking. The seal re-baseline waited
   for the maintainer, because it canonicalises whatever the current codegen
   emits and no independent oracle says that output is right.

6. **Evidence citing a command that does not exist is not weak evidence — it is
   not evidence.** `clara_spec_coverage.json` recorded a passing run of
   `bash scripts/clara/demo.sh`, a path absent from the repository, for four
   months. Run an evidence file's own reproduction command before reading its
   numbers.

## 4. Open questions

- **Is a ternary-weight datapath still the right target** given arXiv:2504.18415
  moving the constraint to activation width? This is a design question the
  repo has not addressed in writing.
- **Can the `--with-sva` output be checked at all in the open-source flow?**
  Prop 2 says not without `sv2v` or Verific. Evaluating `sv2v` is the smallest
  next experiment.
- **Per-file vector *sufficiency* is unmeasured.** Prop 4 counts files carrying
  vectors; it says nothing about whether those vectors exercise anything.
- **Compiler correctness is unproved and unclaimed.** `bootstrap/` is
  unverified Rust. Vericert is the mature alternative if that property is
  wanted.
- **Layer 0 can read an activation buffer nothing wrote** (Prop. 25b). Three
  interlocks were tried and withdrawn; the property is gated as an expected
  refutation so a fix cannot land silently. The likely shape of a real fix is a
  per-buffer written-flag in hardware rather than a single global one, since
  "some write happened" is not "the buffer being read was written".

---

**φ² + 1/φ² = 3 | TRINITY**
