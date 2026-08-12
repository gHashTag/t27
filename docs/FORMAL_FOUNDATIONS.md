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

**Gate:** `seal-coverage.yml` → `t27c seal-audit --repo-root . --strict`

Let `Σ` be the map from spec path to seal path implemented by `seal_file_path`:

```text
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

```text
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

**Gate:** `formal-yosys.yml` → *Assert the property set is non-empty*

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

**Gate:** `formal-yosys.yml` — 13 `sat -verify -prove-asserts` invocations

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

**Gate:** `schema-validation.yml` → `t27c validate-conformance` **(count only — per-file sufficiency remains unmeasured)**

`MEASURED`. Of 101 files in `conformance/`: **88** carry vectors, **5** are
measured reports, **8** are schema definitions, **0** are empty. The prior
validator reported "43 valid, 58 empty" because it resolved payloads with
`.as_array()` only, while the corpus stores vectors both as arrays and as
objects. **A count is a claim about a predicate, and the predicate was wrong.**

---

### Proposition 5 — `sv2v` cannot rescue the SVA: it deletes it

**Gate:** **none.** A one-time measurement of an external tool (`sv2v`) that CI does not install. Cited as history, not as a standing property.

`MEASURED` on sv2v 0.0.13.

The obvious repair for Prop 2 is to preprocess SystemVerilog into Verilog-2005
with [sv2v](https://github.com/zachjs/sv2v) before handing it to Yosys. It does
not work, and the failure mode is the dangerous kind.

sv2v's own README states: *"Assertions are also supported, but are simply
dropped during conversion."* Confirmed directly — input a module containing a
`property` block and an `assert property`, and the output contains **zero**
assertions:

```text
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

**Gate:** `formal-yosys.yml` → *Behavior-DSL subset still emits and parses*

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

**Gate:** `formal-yosys.yml` → *Prove interrupt_controller properties*

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

```text
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

**Gate:** `formal-yosys.yml` → *Prove axi_lite_slave properties*

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

```text
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

**Gate:** `formal-yosys.yml` → *Prove dma_controller properties*

`PROVED` (machine-checked). Third module checked, two more real defects, and
the fourth and fifth passing unit tests found holding a bug in place.

**9a. Burst abandonment.** `m_axi_arlen` and `m_axi_awlen` were hardwired to
`8'hFF` — 256 beats — for *every* transfer, while the FSM left `READ_DATA`
once `bytes_remaining` fell to one beat. A short transfer therefore requested
256 beats and then dropped `rready` mid-burst. **An AXI4 master may not abandon
a burst it requested.**

```text
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

```text
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

**Gate:** `formal-yosys.yml` → *Prove dma_controller properties* (via `formal/axi4_read_slave_model.sv`)

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

```text
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

**Gate:** `formal-yosys.yml` → *Assumptions are active in the proof flow*

`PROVED`. Prop. 10 closed with `arlen == 0` refuting at the address handshake
while a hand-trace said it must hold, recorded as an unexplained anomaly.
It is now explained, and the explanation generalises past this repository.

**11a. Yosys's `sat` ignores `$assume` cells unless `-set-assumes` is passed.**
It is opt-in and silent. A harness without the flag still runs, still prints
`PROVED` or `REFUTED`, and every `assume` in it is inert — so a property meant
to hold *given a compliant environment* is being checked against an arbitrary
one. Demonstrated with a two-line module: `assume (1'b0)` alongside
`assert (a == !a)`.

```text
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

```text
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

**Gate:** `formal-yosys.yml` → *Properties are non-vacuous (witnesses must refute)*

`MEASURED`. Prop. 11 found constraints that did nothing. Vacuity is its mirror:
a property that **passes because the interesting case never happens**. Neither
appears as a failure; both make a green run worthless.

Two levels were checked.

**12a. Guard reachability.** For each `G |-> P`, the assertion body was replaced
with `assert (1'b0)` under the same guard. That run **proves iff `G` is
unreachable** — a precise vacuity oracle needing no `cover` support. All other
assertions in the file were neutralised to `assert (1'b1)` so each result speaks
about one guard only.

```text
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

**Gate:** `formal-yosys.yml` → *Prove layer_sequencer properties* and *Prove zero-size properties*

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

```text
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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

```text
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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

`MEASURED`. Three successive narrowings landed against one property, each real
and each insufficient. The fourth attempt produced the diagnosis instead:

1. `busy` made a state register rather than a decode of `current_layer` (Prop. 21a)
2. the interlock made symmetric in both directions (Prop. 21b)
3. the gate extended across `layer_valid`, `mac_valid_q` and `act_trit_valid`

**22a. The trace.** With `-show` on the top-level signals:

```text
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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

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

**Gate:** `formal-yosys.yml` → *Engine is still alive under its interlocks*

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

### Prop. 25 — the first properties that span two layers — `PROVED` (25b **closed** in Wave 582, see Prop. 33)

**Gate:** `formal-yosys.yml` → *No property is gated as an expected refutation* and *Baseline - unprobed design must prove*

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
| `input_loaded` set by `dma_done` | still REFUTED — ~~a zero-length DMA reaches DONE without writing~~ **RETRACTED, see Prop. 26a.** The real cause was the same implicit-net fault as row 2 |
| set by `dma_local_we`, declared early | still REFUTED — and it broke an unrelated proved property (below) |
| same, with a synchronous reset | still REFUTED — **baseline** broke regardless of reset style |

> **Correction (Prop. 26a).** The mechanism given for row 1 was wrong. A
> zero-length DMA did **not** reach DONE — it was silently dropped and never
> completed at all. The claim was written from the emitted file's own comment
> rather than its behaviour, and the comment was wrong too. Rows 1 and 2 failed
> for the *same* reason: `dma_done` was also read above its declaration, so the
> interlock was wired to an undriven twin and had no effect whatsoever.

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
./target/release/t27c gen-bitnet-bundle --output-dir build/rtl
./target/release/t27c gen-trit-stdlib > build/rtl/trit_stdlib.sv
# Expect a NONZERO exit: 25a proves, 25b refutes by design (see the CI gate).
yosys -p "read_verilog -sv -formal -DFORMAL -DFORMAL_OPEN build/rtl/*.sv; \
          chparam -set DEPTH 4 weight_bram; prep -top bitnet_engine_top -flatten; \
          memory_map; async2sync; chformal -lower; \
          sat -verify -prove-asserts -seq 40 -set-init-zero -set-assumes"
```

---

### Prop. 26 — the zero-sized-request sweep: a 2–2 policy split, and a retraction — `MEASURED` / `PROVED`

**Gate:** `formal-yosys.yml` → *Prove zero-size properties*

Three waves found one defect shape one module at a time, reactively: zero
neurons (Prop. 9), zero words (Prop. 10), and a claimed zero bytes (Prop. 25c).
Finding the same shape three times is a signal about where the rest of them are.
This proposition stops finding them by accident.

**26a. The retraction first.** Prop. 25c stated that a zero-length DMA "reaches
DONE without ever asserting `dma_local_we`, satisfying its completion contract
while writing nothing". **That is false.** The emitted RTL reads:

```verilog
IDLE: if (start && (length != 32'd0)) begin   // the old guard
```

`done` is asserted only in `DONE_ST`, and a zero-length request never leaves
`IDLE`. So it did not complete vacuously — it was **silently dropped**, which is
strictly worse. The claim came from the comment sitting directly above that
line, which read *"A zero-length request moves no data and completes
immediately"* — true of the intent, false of the code, for several waves.

> **A generated file's comments are not evidence about the generated file.**
> This campaign's standing rule is *verify the artifact, not the source*; a
> comment inside the artifact is still source. The behaviour is the artifact.

Rows 1 and 2 of the Prop. 25c table failed for the **same** reason, not two
different ones: `dma_done` is declared at line 262 and was read at line 125, so
that interlock was also wired to an undriven twin (Prop. 25e) and had no effect
at all. One fault, reported as two.

**26b. Every module that takes a count, measured.** Each wrapper in
[`formal/zero_size_props.sv`](../formal/zero_size_props.sv) holds its count at
zero and asserts the module *never completes*. A proof means the request is
dropped; a refutation means it completes.

| module | count | verdict | policy |
|---|---|---|---|
| `layer_sequencer` | `num_neurons` | refutes | **completes** |
| `weight_prefetch_ctrl` | `num_words` | refutes | **completes** |
| `multilayer_sequencer` | `num_layers` | **PROVES** | **dropped** — host hangs |
| `dma_controller` | `length` | **PROVES** | **dropped** — host hangs |

A **2–2 split**. Neither policy is wrong in isolation; four modules disagreeing
is the defect, because a host driving this engine cannot know which to expect.

**26c. The dropping half is the dangerous half.** A dropped request produces no
work, no completion, and no error. It is the one outcome a host **cannot
observe**: the CSR write is accepted, nothing happens, and the completion
interrupt never arrives. A vacuous completion is at least visible. Both
droppers were changed to complete:

```verilog
IDLE: if (start) begin
    done <= 1'b0;
    if (length != 32'd0) begin ... end
    else state <= DONE_ST;        // moves no data, and says so
end
```

**26d. Completing must not mean pretending.** A completion policy is only safe
paired with a proof that the zero job did nothing. Four such properties were
added and **all four prove**: no `valid`, no `layer_start`/`start_prefetch`, no
`bram_we`/`axi_arvalid`, no `local_we`/`m_axi_arvalid`/`m_axi_awvalid`.

After the fix all eight properties hold with **inverted polarity** — every
`*_never_completes` refutes, every no-work property proves — and that is the CI
gate. Both halves are needed: the first alone permits a module that lies, the
second alone permits a module that hangs.

**26e. Proactive sweeps find what reactive ones cannot.** Props. 9 and 10 were
each found because something else broke and the zero case was noticed on the
way. Prop. 25c was found by *guessing* at a mechanism, and the guess was wrong.
The sweep found both real instances in one pass, and produced a **policy
question** — which behaviour is correct? — that no single-module investigation
had raised. **When the same defect shape appears twice, enumerate the whole
class before it appears a third time.**

Reproduce:

```bash
./target/release/t27c gen-bitnet-bundle --output-dir build/rtl
yosys -p "read_verilog -sv -formal build/rtl/dma_controller.sv formal/zero_size_props.sv; \
          prep -top zs_dma -flatten; async2sync; chformal -lower; \
          sat -verify -prove-asserts -seq 24 -set-init-zero -set-assumes"
# Expect a NONZERO exit: a_zero_length_never_completes must REFUTE.
# a_zero_length_moves_no_data must PROVE -- isolate it to see that half.
```

---

### Prop. 27 — the document recording these proofs was itself unchecked evidence — `MEASURED`

**Gate:** `formal-yosys.yml` → *Every proposition carries the gate that keeps it true*

Twenty-six propositions rest on the implicit claim that their reproduction blocks
work. Prop. 26a found one claim written from a comment rather than a run, which
made that implicit claim the least-tested thing in the repository — and the one
everything else stands on. This audits it.

**27a. Fourteen of nineteen shell blocks were transcripts.** Every fenced block
in this document was classified by whether it contains an executable command:

| class | count | what it is |
|---|---|---|
| runnable | 3 | a command a reader can run |
| template | 2 | contains `<placeholders>` — not meant to run |
| **transcript** | **14** | **a result, formatted identically to a command** |

A ```` ```bash ```` fence reads as *"run this"*. Fourteen of them were showing
output. This is the same failure shape the campaign keeps finding — **a form
that reads as stronger evidence than it is** — and this time the form was the
campaign's own documentation. All fourteen are now ```` ```text ````.

**27b. Both blocks a reader could actually run were broken.** The three runnable
blocks were executed. Prop. 1's works. The two added in Waves 574 and 575 —
*by this campaign, in the document that carries the rule* — both begin:

```text
t27c gen-bitnet-bundle --output-dir build/rtl        # `t27c` is not on PATH
```

`which t27c` returns nothing; the binary is at `./target/release/t27c`. Prop. 3's
own lesson 6 states that **evidence citing a command that does not exist is not
weak evidence — it is not evidence**. Both blocks were written after that lesson
was recorded, and neither was ever run. Fixed.

> Writing a rule down does not apply it. The two blocks violating Prop. 3 were
> added by the same author who wrote Prop. 3, in the same file, in the following
> two waves. **A rule with no gate is a preference.**

**27c. Every proposition now names the gate that re-checks it.** Each claim was
traced to a CI step by matching the identifiers it cites — property names, module
names, commands — against the workflows and `formal/*.sv`. Six propositions
matched nothing and were checked individually rather than declared ungated,
which is how the heuristic's four false negatives were caught (Props. 1, 3, 6
and 24 are gated; the extractor simply could not see prose probe labels like
`'DMA can start'`).

**One proposition has no gate, and says so:** Prop. 5 measured that `sv2v` drops
assertions. CI does not install `sv2v` — it appears in this repository only in
comments. That is correct and now explicit: a one-time historical measurement,
not a standing property.

**27d. The convention is now enforced.** A CI step fails the build if a
proposition lacks a `**Gate:**` line, if a ```` ```bash ```` block calls bare
`t27c`, or if a ```` ```bash ```` block contains no command at all. The three
defects found here cannot recur silently.

**27e. What this does not establish.** The gate map says each claim *has* a
check, not that the check is *sufficient* for the claim. Prop. 4's gate counts
conformance files without measuring vector sufficiency, and says so. Reviewing
gate adequacy claim-by-claim is a separate, larger audit.

Reproduce:

```bash
python3 -c "
import re
lines = open('docs/FORMAL_FOUNDATIONS.md').read().split(chr(10))
props = [l for l in lines if re.match(r'^### (Proposition|Prop\.) [0-9]+', l)]
gates = [l for l in lines if l.startswith('**Gate:**')]
print(len(props), 'propositions,', len(gates), 'gate lines')
"
```

---

### Prop. 28 — the gates bite: 13 of 13 detected a mutation aimed at the claim they guard — `PROVED`

**Gate:** `formal-mutation.yml` → *Baseline, control, and mutation* (weekly)

Prop. 27 established that every claim **has** a check, and said explicitly that it
did not establish any check was **sufficient**. This is the missing half. It is
the vacuity oracle of Prop. 12a redirected at the gate map: a gate that cannot
fail is not a gate, exactly as a property whose guard is unreachable is not a
property.

**28a. Method.** For each gate, apply one mutation that should violate the claim
it guards, then run that gate alone. The gate must go **red**. Mutations are
applied to the *generated* RTL rather than to the emitters, so a run costs one
`yosys` invocation instead of a rebuild, and each mutation string is asserted to
occur exactly once before use.

**28b. The two phases that make the third mean anything.**

| phase | requirement | why |
|---|---|---|
| **baseline** | unmutated build, every gate passes | without it, "the gate went red" is not evidence the *mutation* did it — Prop. 25d, applied to this harness |
| **control** | a dead wire added to every module, every gate still passes | catches a gate that fires on any edit at all, which would score 8/8 while detecting nothing |
| **mutation** | each gate goes red for its own mutation | the actual claim |

Both control phases came back clean, which is what licenses reading the third.

**28c. Result — 13 of 13.**

| gate | mutation | verdict |
|---|---|---|
| Prop. 7 `interrupt_controller` | revert clear-then-set → set-then-clear | red |
| Prop. 8 `axi_lite_slave` | ready stays high while a response is pending | red |
| Prop. 9 `dma_controller` | advance the burst without a handshake | red |
| Prop. 13 `layer_sequencer` | drop the zero-neuron guard | red |
| Prop. 25 integration | double buffer stops alternating | red |
| Prop. 24 liveness | tie inference `start` off, stalling the engine | red |
| Prop. 26 DMA | zero-length request dropped again | red |
| Prop. 26 multilayer | zero-layer inference dropped again | red |
| Prop. 11 assumptions | drop `-set-assumes` from the flow | red |
| Prop. 27 doc gate | remove one `**Gate:**` line | red |
| Prop. 27 doc gate | make a block cite bare `t27c` | red |
| Prop. 27 doc gate | leave a ```` ```bash ```` block with no command | red |
| Prop. 1 seals | edit a spec, leave its seal stale | red |

The Prop. 24 mutation is worth separating out. Stalling the engine leaves every
*safety* property true — an engine that does nothing violates nothing — and the
liveness witnesses are the only reason it goes red. That gate exists precisely
for a mutation no safety property can see, and it caught it.

**28d. A clean sweep is a reason to check the harness, not to celebrate.** 8/8
on the first RTL batch was the point at which the previous three waves would
have found a harness defect. So the baseline and control phases were added
before the result was written down, not after — and the workflow runs all three
phases in that order every time, so the result cannot silently degrade into
"everything passes because nothing runs".

**28e. What this still does not establish.** Each gate detects *the* mutation
chosen for it. That is one point per claim, not a proof of adequacy over all
possible violations — mutation testing bounds from below and never from above.
Prop. 4's gate remains a counted-files check with no vector-sufficiency
measurement, and no mutation here changes that.

Reproduce:

```bash
python3 -c "
import yaml
w = yaml.safe_load(open('.github/workflows/formal-mutation.yml'))
s = [x for x in w['jobs']['gate-adequacy']['steps'] if 'Baseline' in x.get('name','')][0]
b = s['run'].split(chr(10))
i = [n for n,l in enumerate(b) if chr(60)*2+chr(39)+'PY'+chr(39) in l][0]
j = [n for n,l in enumerate(b) if l.strip()=='PY' and n>i][0]
open('/tmp/h.py','w').write(chr(10).join(b[i+1:j]))
print('harness extracted to /tmp/h.py -- run it from the repo root')
"
```

---

### Prop. 29 — the other end of every count: two defects the bound could not see — `PROVED` / open

**Gate:** `formal-yosys.yml` → *Oversized requests do not wrap the local address*

Wave 575 swept the zero end of every count and found two real defects. This is
the maximum end, which had never been examined. The shape looked for: **a count
wider than the thing it indexes.**

| module | count | address it drives | ratio |
|---|---|---|---|
| `weight_prefetch_ctrl` | `num_words` 16 bits | `bram_addr` 12 bits | 16× |
| `dma_controller` | `length` 32 bits (8 bytes/word) | `local_addr` 12 bits | 128× |

**29a. The first verdict was a bound artifact, and looked like good news.** The
monotonicity property proved at `-seq 24` on both modules. It is a true
statement and a worthless one: reaching address 4096 takes 4096 writes, so the
counterexample is **unreachable by construction** at any tractable bound. The
proof establishes "no wrap within 24 cycles", which nobody doubted.

> A bounded proof says nothing about a property whose counterexample lies beyond
> the bound. **Before believing a bounded proof, ask how many cycles a violation
> would need.** If the answer exceeds the bound, the verdict is structural, not
> empirical.

The fix is the technique this repository already uses for memories
(`chparam -set DEPTH 4 weight_bram`): scale the model until the counterexample
fits. Narrowing the address to 3 bits (8 entries) brought the wrap inside the
bound, and **both modules refuted immediately.**

**29b. Defect one — the address wraps and overwrites.** Past 4096 entries the
counter wraps to zero and the transfer keeps writing over data it already
fetched, then reports success. Silent corruption, invisible to every existing
property. Both modules now **clamp** to the address space and raise a new
`overflow` output.

**29c. The error IRQ existed and was tied off.** `bitnet_engine_top` instantiated
the interrupt controller with `.error(1'b0)` — a sticky, maskable,
read-to-clear status bit that nothing could ever set. An oversized request is
exactly what it is for, so both `overflow` outputs now drive it. Following
Prop. 26c: the request completes, nothing is corrupted, **and the host is told.**

**29d. Defect two — every word was written one slot too high.** Found only
because 29b's fix did not make the property pass. In both engines the data, the
write-enable and the address increment are non-blocking assignments in the same
cycle:

```verilog
bram_data <= axi_rdata[53:0];
bram_we   <= 1'b1;
bram_addr <= bram_addr + 12'd1;   // the BRAM sees the POST-increment address
```

All three reach the memory from the same stage, so the word fetched at index N
is written at address **N+1**: address 0 is never written, and the final word
wraps over it. Both now carry a separate `word_index` and write at the word's
own index.

This defect had nothing to do with sizing. It was found because a property that
should have passed after a correct fix did not, and the gap was investigated
rather than papered over.

**29e. `weight_prefetch_ctrl` is proved; `dma_controller` is open.** After both
fixes the scaled prefetch model **proves**, and removing the clamp makes it
refute again — discriminating in both directions. The DMA, with the same two
fixes applied by identical construction, **still refutes** under the same scaled
model and a single-beat AXI environment, and the cause is not identified. Two
patches were tried; neither closed it. Per the standing discipline, that is a
finding rather than a third guess: it is gated as an **expected refutation**, so
closing it turns the build red and asks for promotion.

**29f. Two environment faults were diagnosed on the way, both mine.** The
property first refuted because it compared a write address from a *new* transfer
against the last address of the *previous* one, and again because an
unconstrained `m_axi_rlast` let the solver play a slave that never ends a burst.
Neither was a design defect. **An unconstrained input is an adversary**, and a
refutation is a claim about the environment until the environment is pinned
down.

Reproduce:

```bash
./target/release/t27c gen-bitnet-bundle --output-dir build/rtl
python3 /tmp/ms.py 2>/dev/null || echo "extract the harness from formal-yosys.yml: 'Oversized requests do not wrap'"
```

---

### Prop. 30 — the write-pairing shape, enumerated across every port — `PROVED` / open

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)* and *Oversized requests do not wrap the local address*

Prop. 29d found a data/enable/address trio registered together with the address
advanced, so word *N* landed at address *N+1* and slot 0 was never written. It
was found by accident, in two modules, while investigating something else. The
zero-sweep's lesson (Prop. 26e) says that after the second sighting of a shape
you enumerate the class rather than wait for the third.

**30a. The syntactic scan found nothing, which was the wrong question.** A
regex over every clocked block looking for a self-incremented output co-assigned
with an enable returned **zero** candidates — because both instances had already
been fixed. A scan for the *broken form* of a shape can only find instances
nobody has repaired. The useful question is semantic: **does every write port
present address, data and enable from the same stage?**

**30b. Three write ports, enumerated.**

| port | address source | enable | verdict |
|---|---|---|---|
| `wmem` weight BRAM | `pf_bram_addr` ← `word_index` | `pf_bram_we` | **PROVED** contiguous |
| `amem_a`/`amem_b` activation | `act_wr_word` (registered index) | `act_word_valid` | **PROVED** contiguous |
| DMA local | `local_addr` ← `word_index` | `local_we` | **open** — refutes |

The activation port is the one that had never been checked at all. It pairs a
*registered* index with a *combinational* valid, which is the correct shape —
and it is now proved rather than asserted by inspection.

**30c. Contiguity is the right property; monotonicity was not enough.**
Prop. 29's property required the write address to increase. That permits
skipping slot 0 — which is exactly what the defect did. The property that would
have caught 29d directly is stronger:

```verilog
reg [11:0] fv_next;
always @(posedge clk)
    if (!rst_n || !active) fv_next <= 12'd0;
    else if (we)           fv_next <= fv_next + 12'd1;

always @(posedge clk) if (rst_n && active && we)
    a_writes_contiguous: assert (addr == fv_next);
```

**No gap and no repeat, starting at zero.** It now guards all three ports.
**A property that a known defect would have passed is the wrong property**, and
the cheapest time to notice is right after fixing that defect.

**30d. Non-vacuous.** The activation property's guard was checked with the
Prop. 12a oracle — body replaced by `assert (1'b0)` under the same guard — and
refutes, so the guard is reachable. Integration properties: **21, all proving.**

**30e. The DMA port stays open, and was not re-diagnosed.** Its two properties
refute. The wrapper's baseline was re-checked with every property neutralised
and **proves**, so the harness is sound and the refutation is real. An attempt
to read the counterexample was inconclusive: the trace showed `local_we` low
throughout, which cannot violate a property guarded on `local_we`, so the
extraction — not the verdict — is untrustworthy. Recorded as-is rather than
re-diagnosed with a tool that just gave a contradictory answer.

---

### Prop. 31 — the instrument was broken, and fixing it found the defect — `MEASURED` / open

**Gate:** `formal-yosys.yml` → *Trace reader reads a known counterexample*

Two waves stalled on one open finding, and the blocker had stopped being the
design. `sat -show`'s text table was parsed with an ad-hoc regex that dropped
rows, producing a trace in which the guard signal was low throughout — which
cannot violate a property guarded on that signal. Prop. 30e recorded that and
correctly refused to reason from it.

**31a. `yosys sat -dump_json` emits invalid JSON.** The structured alternative
looked like the fix, and its output does not parse:

```text
{ "name": "$auto$async2sync.cc:107:execute$243", "wave": "0.1..." }
                                       ^ \e is not a JSON escape
```

RTLIL names are written verbatim, backslashes and all, so any name containing
`\e`, `\d` or similar breaks the document. [`formal/trace_reader.py`](../formal/trace_reader.py)
escapes stray backslashes before parsing. It also expands WaveJSON properly:
`.` repeats the previous value, `=` consumes the next `data` entry. A reader
that ignores `.` loses most of the trace — the same failure, one layer down.

**31b. Validated before use.** The reader is pointed at a property whose
counterexample is **known** — the prefetch with its clamp removed, which must
show a write at a wrapped address. It parses 91 signals and finds the wrap at
t=18. That check is a CI step, so the instrument cannot silently rot into the
thing it replaced.

> **Verify the instrument on a case whose answer you already know, before
> trusting it on one you don't.** The reader that produced two waves of
> confusion would have failed this check in one second.

**31c. With the instrument working, the defect was legible immediately.** The
first query — not eyeballing, but asking *at which timestep does the guard hold
and the assertion fail* — returned `t=28: local_addr=1, expected 0`. The first
write of a transfer was landing at address 1.

Two mechanisms, both now fixed:

1. **`local_addr` served two roles.** Write pointer when data comes *from* the
   bus, read pointer when it goes *to* it. Prop. 29d gave only the first role
   its own index, so the two fought: a transfer could enter `READ_DATA` with
   `local_addr` already advanced by the write path. Both paths now share one
   sequential index.
2. **The pointer reset sat inside the `length != 0` branch.** A zero-length
   request takes the DONE path (Prop. 26c) and left the pointers where the
   previous transfer had put them. Reset now happens on **every** start.

**31d. Still open, and now open for a stated reason.** After both fixes the
property still refutes. That is the third patch on this item; the rule from
Prop. 29 — *after two failed attempts, read the counterexample rather than patch
again* — was followed, produced two real defects, and did not exhaust the cause.
It remains gated as an expected refutation. What changed is that the next
investigation starts with a working instrument instead of a broken one.

**31e. Both fixes were kept.** Neither closed the target property, which by
Prop. 25's standard is grounds for withdrawal. They were kept because each is
independently correct — one index per transfer, reset on every start — and
because module suites, the engine baseline, 21 integration properties and the
full zero-size sweep all still pass. **A fix that does not close its target is
withdrawn when it costs something; kept when it is right on its own terms.**

Reproduce:

```bash
python3 formal/trace_reader.py build/known.json bram_we bram_addr fv_next
```

---

### Prop. 32 — the DMA closes: a write strobe was a level, not a pulse — `PROVED`

**Gate:** `formal-yosys.yml` → *Oversized requests do not wrap the local address* (now expected to **prove**)

Four waves carried one open property. It is closed. Three distinct defects sat
behind it, and each was only visible once the previous one was fixed.

| wave | defect | mechanism |
|---|---|---|
| 578 (29d) | word *N* written at address *N+1* | data, enable and address registered together with the address advanced |
| 580 (31c) | first write of a transfer at a stale address | `local_addr` served two roles, and its reset sat inside the `length != 0` branch |
| **581 (this)** | **write strobe held across states** | `local_we` cleared only inside `READ_DATA`'s `else` |

**32a. The defect.** `local_we` was assigned in exactly one place outside reset —
`READ_DATA: if (rvalid) ... else local_we <= 1'b0;`. That `else` only runs while
the FSM *is in* `READ_DATA`. In `READ_ADDR`, between bursts, `local_we` is not
assigned at all, so it **holds**, and the DMA keeps writing at whatever address
`local_addr` last held. The counterexample is unambiguous:

```text
cycles with local_we = 1   : 24
cycles with m_axi_rvalid=1 : 18
local_we high with no beat behind it: 8 of 24
```

**A write strobe must be a pulse, not a level.** `local_we` now defaults low
before the `case`, so every state that does not write leaves it deasserted.

**32b. The instrument earned its wave.** Prop. 31 built and validated a
counterexample reader. Every step here was a *query* against it — "at which
timestep is the assertion enabled", then "how many enable cycles have no beat
behind them" — and the second query produced the defect outright. Four waves of
inspection had not found it; two queries did.

**32c. A scaled model must scale the harness too.** Most of this wave went to a
false lead. The scaled DUT narrows `local_addr` to 3 bits, but the wrapper still
declared `wire [11:0] local_addr`, leaving nine undriven bits. Every comparison
against them is `x`, and `x != fv_next` refutes — **which reads exactly like a
design defect**. The trace showed the address as `-`; that is `x`, not zero, and
reading it as "unparsed" cost hours.

> When you scale a model, scale **everything that touches the scaled signal**.
> A width mismatch at a harness boundary produces `x`, and `x` fails every
> comparison, so it manifests as a confident refutation of an innocent design.

**32d. What is proved, and at what scale.** `a_local_addr_never_wraps` proves on
the scaled model and refutes when the clamp is removed — discriminating in both
directions. `a_local_writes_contiguous` proves, but its clamp-removed variant
*also* proves at this bound, so **that property is not discriminating here** and
carries no weight on its own. Recorded rather than quietly counted as a second
result.

**32e. Score for the sweep that started this.** The maximum-sized-request sweep
(Prop. 29) has now produced **five** distinct RTL defects across two modules:
address wrap, off-by-one write pairing, dual-role pointer, misplaced reset, and
a held strobe. Four of the five had nothing to do with request size. **A sweep's
value is not only what it was aimed at.**

Reproduce:

```bash
python3 formal/trace_reader.py build/d4.json local_we local_addr fv_next
```

---

### Prop. 33 — the last open defect closes: the interlock was the right idea in the wrong shape — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*, *No property is gated as an expected refutation*

Prop. 25b stood open for eight waves: with nothing requiring a DMA first, the
MAC could consume an activation buffer nothing had written. Wave 574 tried three
interlocks and withdrew all three. **Every one of them was the right idea in the
wrong shape**, and the shape was only visible once the counterexample could be
read.

**33a. Wave 574's blocker had already dissolved.** All three attempts broke the
*baseline* — the design stopped proving with no property of ours involved. That
was never explained. Re-applying the same interlock to today's design: the
baseline **proves**. Nothing was done to fix it directly; it went away with the
three DMA defects closed in Waves 578–581.

> A blocker recorded rather than forced can dissolve on its own. Prop. 25's
> discipline — *record, do not weaken* — cost three withdrawn patches and bought
> a clean re-attempt eight waves later.

**33b. The interlock was necessary and insufficient.** With `input_loaded`
gating `start`, the baseline proves and all 22 properties hold — and 25b still
refutes. The counterexample, one query against the Prop. 31 reader:

```text
 t  input_loaded  use_buffer_a  fv_wrote_a  fv_wrote_b  act_word_valid  mac_valid_q
31            1             1           1           0               0            0   <- layer 0 done
32            1             0           1           0               0            0   <- ping-pong flips
37            1             0           1           0               0            1   <- MAC reads B
```

**Layer 0 completed having emitted no activation words at all.** That is legal:
a zero-neuron layer completes immediately by design (Prop. 26). The ping-pong
flips, and layer 1 reads a buffer nothing ever wrote.

**33c. A global flag cannot answer a per-buffer question.** `input_loaded` asks
*did anything get written*; the property asks *was the buffer this layer reads
written*. No amount of tuning a single bit answers the second question. The fix
is two real registers, `wrote_a` / `wrote_b`, set by the actual write enables —
exactly the shape predicted in Wave 574's open-questions note and not attempted
until the counterexample made it obvious.

**33d. Error, not stall.** Refusing to start a layer whose buffer is unwritten
would hang the engine on a legitimately empty layer, and **a stalled engine
satisfies every safety property** (Prop. 24). So the layer is not started *and*
`buffer_unwritten` drives the error IRQ that Prop. 29c gave a driver. All
liveness witnesses still refute: the engine works.

**33e. The gate did its job.** Prop. 25b was gated as an *expected refutation*
so that closing it would turn the build red and demand promotion. It did exactly
that. The property now lives in the default set (**23 integration properties,
all proving**), passes the vacuity oracle, and the gate has been replaced by one
asserting that **no expected-refutation guard remains** — nothing in the engine
is knowingly broken.

Reproduce:

```bash
./target/release/t27c gen-bitnet-bundle --output-dir build/rtl
grep -c "ifdef FORMAL_OPEN" build/rtl/bitnet_engine_top.sv    # must be 0
```

---

### Prop. 34 — "proved" is a claim about the pair (design, scale) — `MEASURED`

**Gate:** `formal-mutation.yml` → *Scale ceiling* (weekly)

Every engine property proves at `-seq 40` with `chparam DEPTH 4`. Prop. 29a
showed what that can be worth: two modules "proved" an address never wraps while
both contained a wrap, because reaching it took 4096 writes and the bound was
24. A bounded proof establishes nothing about a counterexample beyond the bound,
so **the number must travel with the claim.** This measures where the proof
stops.

**34a. The engine.** All 23 integration properties, whole set, one run each:

| `-seq` | `DEPTH` | verdict | wall time |
|---:|---:|---|---:|
| 40 | 4 | **PROVED** | 40.7 s |
| 60 | 4 | **PROVED** | 246.1 s |

> **Superseded by Prop. 53.** Every row below was measured before ten defects
> were fixed and six properties added. Re-measured in Wave 603, three of these
> configurations no longer complete: the ceiling is now `-seq 40`.
| 80 | 4 | **PROVED** | 396.1 s |
| 120 | 4 | *undecided* (>1800 s) | — |
| 40 | 8 | **PROVED** | 70.5 s |
| 60 | 8 | **PROVED** | 219.7 s |
| 40 | 16 | **PROVED** | 77.0 s |

> **Corrected in Wave 587.** The `seq 80` row originally read *undecided
> (>300 s)*. It proves in **396.1 s** — the 300 s budget was simply too small,
> and the ceiling recorded here was a property of that budget. This is the exact
> error Prop. 37 names, committed one wave before naming it. The engine holds at
> **2× the bound CI uses**, not 1.5×, and the real ceiling lies between 80 and
> 120.

Three things fall out. The properties hold at **2× the bound CI uses**, so the
documented claim is not sitting on the edge of its own tractability. They hold
with **both dimensions raised together** — `seq 60` *and* `DEPTH 8` — which a
single-axis sweep would not have established. And the cost is sharply
asymmetric: **1.5× the unrolling costs 6× the time**, while **doubling the
memory costs 1.7×**. Memory depth is cheap; unroll depth is not.

**34b. The modules, at 1×, 2× and 4× their CI bounds.**

| module | CI `-seq` | 2× | 4× |
|---|---:|---|---|
| `interrupt_controller` | 12 → PROVED | PROVED | PROVED |
| `axi_lite_slave` | 20 → PROVED | PROVED | PROVED |

> **Corrected by Prop. 36a.** The first two rows are measured with plain BMC,
> but CI proves both by `-tempinduct` — their real proofs are **unbounded**, and
> a scale table understates rather than describes them.
| `dma_controller` | 20 → PROVED | PROVED | PROVED |
| `layer_sequencer` | 12 → PROVED | PROVED | PROVED |
| `weight_prefetch_ctrl` | 20 → PROVED | **undecided** (>240 s) | **undecided** |

> **Superseded by Prop. 35.** Every row above is a *batch* measurement — one
> `sat` invocation proving all of a module's assertions together. Splitting
> `weight_prefetch_ctrl` one property per invocation proves all three at
> `-seq 40`, so its "undecided at 2×" was a property of the batching, not of the
> module.

Four of five extend to 4×. **`weight_prefetch_ctrl` does not extend at all** —
it becomes intractable at twice its bound. Its proof is real at `-seq 20` and
nothing is known beyond it. That is not a defect and not a pass; it is a third
answer, and it is only visible because the question was asked.

**34c. Undecided is not proved, and not refuted.** A timeout says the solver ran
out of time, not that the property fails. Reporting these as failures would be
alarmist and as passes would be false. They are recorded as **undecided at that
scale**, which is the only honest reading and the reason the table has three
verdicts rather than two.

**34d. No property refuted at any larger scale that completed.** The eight RTL
defects found in Waves 573–582 were all reachable within the bounds in use. That
is evidence the bounds were adequate *for the defects that existed*, and is not
evidence that no deeper defect exists — the `weight_prefetch_ctrl` row is
precisely where such a defect could hide unseen.

**34e. The claim now carries its ceiling.** `README.md` and this document say
*proved at `-seq 40`, `DEPTH 4`*, and the weekly gate re-establishes the three
scales the claim rests on, failing if any of them starts refuting or stops
completing. **A ceiling that is not checked drifts silently as the design
grows.**

Reproduce:

```bash
python3 formal/scale_probe.py 60 4          # aggregate verdict and timing
python3 formal/scale_probe.py 40 4 --each   # attribute a failure to one property
```

---

### Prop. 35 — a batch verdict is the minimum of its parts — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove weight_prefetch_ctrl properties* (now one invocation per property, at `-seq 40`)

Prop. 34b named `weight_prefetch_ctrl` as the one module whose proof does not
extend — intractable at twice its bound, and therefore the one place a deeper
defect could sit unseen. That turned out to be a fact about **how it was asked**,
not about the module.

**35a. Individually decidable, jointly intractable.** `-prove-asserts` solves
every assertion of a module in a single SAT instance. At `-seq 40`:

| property | verdict | time |
|---|---|---:|
| `a_sanity` | PROVED | 0.2 s |
| `a_no_overwrite` | PROVED | **87.2 s** |
| `a_rready_implies_active` | PROVED | 0.4 s |
| **all three together** | **undecided** | **>240 s** |

The parts sum to under 90 seconds; the whole exceeds 240. The combined instance
is superlinearly harder than its pieces.

**35b. Two consequences, and the second is the reporting one.** Splitting raised
the bound this module is verified at from **14 to 40** for the same wall time —
CI now proves each property in its own invocation. And a batch verdict is the
**minimum over its members**: reporting one number for `weight_prefetch_ctrl`
concealed that two of its properties hold at `-seq 80` while the third stops at
40.

> **A suite-level verdict tells you about its worst member and nothing about the
> rest.** Where members differ by two orders of magnitude in cost — here 0.2 s
> against 87 s — the aggregate is dominated by one of them and describes none of
> the others.

Splitting also attributes a failure. A batch that goes red says *something in
here broke*; per-property invocations name it.

**35c. A cheaper decomposition was attempted and withdrawn.** `a_no_overwrite`
bounds a 17-bit counter against a 16-bit input, which forces the solver to carry
that counter across the whole unrolling — the reason it is the expensive one.
The intended replacement was a *local* invariant, `writes == bram_addr + 1`,
leaning on `max_size_props` for the address never wrapping: a local invariant
plus an existing property, in place of one global count.

It refuted in 0.5 s, twice, on the alignment between a counter registered off
`bram_we` and an address assigned from `word_index` on the same edge. The idea
is sound and the alignment is not established. Withdrawn and recorded rather
than guessed a third time (Prop. 31's rule).

**35d. The blind spot is narrowed, not closed.** `a_no_overwrite` is proved at
`-seq 40` and undecided at 80. The module is no longer the outlier it was in
Prop. 34b, but it remains the shallowest-verified property in the design, and
that is now stated per property rather than per module.

Reproduce:

```bash
python3 - <<'EOF'
import re, subprocess
src = open('formal/weight_prefetch_props.sv').read()
for keep in re.findall(r'(a_[a-z_]+): assert', src):
    s = src
    for o in re.findall(r'(a_[a-z_]+): assert', src):
        if o != keep:
            s = re.sub(re.escape(o) + r': assert \([^;]*\);', o + ": assert (1'b1);", s)
    open('build/wp_one.sv', 'w').write(s)
    r = subprocess.run(['yosys', '-q', '-p',
        'read_verilog -sv -formal build/rtl/weight_prefetch_ctrl.sv build/wp_one.sv; '
        'prep -top wp_props -flatten; async2sync; chformal -lower; '
        'sat -verify -prove-asserts -seq 40 -set-init-zero -set-assumes'], capture_output=True)
    print(keep, 'PROVED' if r.returncode == 0 else 'NOT PROVED')
EOF
```

---

### Prop. 36 — ~~two suites were never bounded at all~~ **one is** — and the map shows the rest have enormous headroom — `MEASURED` (36a corrected in Prop. 41c)

**Gate:** `formal-yosys.yml` → the five *Prove … properties* steps (bounds raised where BMC, unchanged where inductive)

Prop. 35 split one module and found its aggregate verdict was hiding its
members. This maps the rest: every property of every module suite, isolated, at
1×, 2×, 4× and 8× the bound it is checked at.

**36a. Not every suite is bounded.** Two of the six run `sat -tempinduct`, which
proves by **k-induction** and therefore holds for *all* time, not to a depth:

| suite | mode | `-seq` means |
|---|---|---|
| `interrupt_controller` | **`-tempinduct`** | induction depth — proof is **unbounded** |
| `axi_lite_slave` | ~~`-tempinduct`~~ **bounded BMC** | **corrected, Prop. 41c** — the word appears only in that step's comment |
| `dma_controller` | bounded BMC | a ceiling |
| `layer_sequencer` | bounded BMC | a ceiling |
| `weight_prefetch_ctrl` | bounded BMC | a ceiling |
| `bitnet_engine_top` | bounded BMC | a ceiling |

**Prop. 34's scale-ceiling framing does not apply to the first two.** Worse, the
map measured them with plain BMC and reported "proved at 8× the CI bound",
which *understates* them: they are proved without any bound at all. A ceiling
was attributed to results that have none.

> **Before measuring how far a result extends, check whether it is the kind of
> result that extends.** The two are distinguished by one flag, and nothing in
> the aggregate output says which mode produced the verdict.

**36b. The near-mistake this caused.** Acting on "everything proves at 4× for
under 8 seconds", the CI bounds were raised — including `axi_lite_slave` from 10
to 80. For a `-tempinduct` run that is not a strengthening: the proof is already
unbounded, and `-seq` is the induction depth, so the only effect is cost.
Reverted. **A number that means one thing in one mode means something else in
another, and the parameter has the same name in both.**

**36c. The bounded suites have enormous headroom.** Every property of every
bounded suite, isolated:

| suite | properties | deepest **PROVED** (isolated) | slowest |
|---|---:|---|---:|
| `dma_controller` | 7 | ≥160 (8× the CI bound) | 8.8 s |
| `layer_sequencer` | 4 | ≥96 (8×) | 50.0 s |
| `weight_prefetch_ctrl` | 3 | 2 at ≥80, 1 at 40 | 87.2 s |

"≥" because 8× was the sweep's own cap, not the properties' limit. Only
`a_no_overwrite` (Prop. 35) has a measured ceiling below the cap.

**36d. Bounds raised where that is meaningful.** `dma_controller` 12 → **80**
(3.6 s) and `layer_sequencer` 12 → **48** (9.8 s), both verified. That is 6.7×
and 4× deeper verification for about thirteen seconds of CI time. The two
inductive suites were left alone, and `weight_prefetch_ctrl` stays per-property
at 40 (Prop. 35).

**36e. What the map is worth.** Before it, the design's verification was six
numbers, two of which meant something different from the other four and one of
which was the minimum over three wildly different members. After it, every
property has a measured depth and the one genuinely shallow property in the
design is named. **The aggregate was not wrong; it was uninformative in a way
that looked informative.**

Reproduce:

```bash
grep -c tempinduct .github/workflows/formal-yosys.yml   # which proofs are unbounded
```

---

### Prop. 37 — splitting helps when cost is property-dominated, not when it is model-dominated — `MEASURED`

**Gate:** `formal-mutation.yml` → *Scale ceiling*

Prop. 35 split one module's suite and gained a 2.9× deeper bound. The obvious
next step was to split the 20-property engine set the same way. It does not
work, and *why* it does not work is the useful part.

**37a. The measurement that looked like a per-property map.** Each engine
property isolated at `-seq 80` with a 240 s budget: **8 of 20 proved**, 12
undecided. That reads like a depth map — until you notice which properties are
in which group. `a_sanity` is `assert (bram_addr == bram_addr)`, a tautology,
and it is in the **undecided** group. A tautology has no depth.

**37b. The cost is the model, not the property.** Re-run with a real budget:

| property | verdict | time |
|---|---|---:|
| `a_sanity` (a tautology) | PROVED | 276.2 s |
| `a_no_read_before_write` (cross-layer, the hardest one) | PROVED | 299.2 s |

**Eight percent apart.** At `-seq 80` the engine costs ~280 s to unroll and
solve *regardless of what is being asserted*. The 240 s budget cut across that
plateau, and which properties landed on which side of it was near-arbitrary.

**37c. The dichotomy.** Two suites, opposite answers, same technique:

| | `weight_prefetch_ctrl` | `bitnet_engine_top` |
|---|---|---|
| cheapest property | 0.2 s | 276 s |
| dearest property | 87.2 s | 299 s |
| ratio | **436×** | **1.08×** |
| splitting gains | **2.9× deeper bound** | **nothing** |

> **Splitting a verification suite pays exactly when its members differ in cost.**
> Where one property dominates, isolating it removes the others' contribution to
> a shared instance and the bound rises. Where every property costs the same
> because the *model* is the expense, splitting buys attribution and no depth at
> all.

The diagnostic is one run: **time a tautology.** If a trivially true assertion
costs what a real one costs, the model is the bottleneck and splitting will not
help. That check costs one invocation and would have saved this wave's first
measurement from being over-read.

**37d-bis. The same error, found in this campaign's own record.** Wave 583
published the engine's ceiling as *undecided at `-seq 80`* on a 300 s budget.
Re-run with 1200 s it **proves in 396.1 s**. The published ceiling was a
property of the budget, recorded one wave before this proposition named the
mistake. Corrected in Prop. 34a. The engine's real ceiling is between 80 and
120, and the batch costs 396 s against ~280 s for any single property — a
1.4× overhead, not the superlinear blow-up seen at module scale, which is what
"model-dominated" predicts.

**37d. What was over-read, precisely.** "8 of 20 proved at `-seq 80`" was
reported by the sweep and is true. It invites the reading *these 8 properties
are deeper than those 12*, which is false — all 20 prove at `-seq 80` given
enough time, and the split was an artifact of where a timeout fell on a
plateau. **A partition produced by a timeout is a partition of the timeout, not
of the subject.**

Reproduce:

```bash
python3 formal/scale_probe.py 80 4 --each   # the per-property view, budget-bound
```

---

### Prop. 38 — ~~the MAC is 8× of the solve cost~~ **removing it lets the optimiser delete the datapath behind it** — `MEASURED` (38a corrected in Prop. 49)

**Gate:** `formal-mutation.yml` → *Scale ceiling*

Prop. 37 established that the engine's verification cost is **model-dominated**:
a tautology costs 276 s where the hardest property costs 299 s. The only lever
that helps every property at once is making the model cheaper. This locates the
cost and finds that the standard mitigation does not reach it.

**38a. The datapath is 31% of the cells and 87% of the time.** Replacing
`pipeline_stage2_compute` with a stub of identical interface and schedule:

| build | cells | flops | `-seq 80` solve |
|---|---:|---:|---:|
| full | 971 | 268 | **369.2 s** |
| MAC replaced | 667 | 267 | **46.0 s** |

**An 8× reduction from removing under a third of the cells**, and essentially no
change in flop count. The expense is the combinational 27-lane dot product and
its adder tree, not sequential state — which is why unrolling multiplies it so
sharply.

**38b. The stub is a cost measurement and not a model.** All 20 properties
"refuted" under it — including `a_sanity`, `assert (bram_addr == bram_addr)`. A
tautology cannot be refuted by changing a multiplier. The baseline check
(Prop. 25d) settled it: the stubbed build **does not prove with no properties at
all**, so every one of those 20 verdicts was noise.

> Third time this discipline has paid, and the first time it caught **my own
> replacement** rather than a design change. The timing number survives because
> it measures how long the solver ran; the verdicts do not, because they measure
> a build that was never sound.

**38c. `chparam` cannot reach this.** Memory depth is scalable because it *is* a
parameter — `chparam -set DEPTH 4 weight_bram` is a flag, not an edit. The
datapath has no such handle:

| quantity | where it lives | sites |
|---|---|---:|
| trit-word width (`[53:0]`, `54'`) | 6 emitters | **26** |
| lane count (`27`) | `trit_stdlib.rs` | **37** |

`trit27_dot_product`, `trit27_parallel_multiply` and `adder_tree_27` take no
parameters; their generate loops count to a literal 27, and the 54-bit word is
threaded independently through the buffers, the BRAMs, the requantizer and the
top. **The width is a repository-wide constant, not a knob.**

**38d. What that costs, stated plainly.** The engine proves at `-seq 80` in
396 s and is undecided at 120 (Prop. 34a, as corrected). An 8× cheaper datapath
would put `-seq 120` and beyond within the same budget — the single largest
available gain in the verification setup, and it is blocked on a refactor rather
than on a technique.

**38e. Not attempted here.** Threading a `LANES`/`WORD_W` parameter through six
emitters is a real change to every consumer of the datapath, and doing it at the
end of a long session to serve a proof budget is how correct RTL acquires
defects. Measured, scoped, and left for a wave that starts with it.

Reproduce:

```bash
grep -c "53:0\|54'" bootstrap/src/*.rs   # the width, site by site
```

---

### Prop. 39 — the read side, and the baseline that never existed — `PROVED` / `MEASURED` / open

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*, *No property is gated as an expected refutation*

Every property through Prop. 38 constrained **writes**. This adds the mirror —
and in doing so uncovered that the campaign's own baseline check has never done
what it claimed.

**39a. Two read-side properties, proved and non-vacuous.** The activation BRAMs
have a one-cycle read latency, so the address is issued one cycle before the
word arrives, while `activation_word = use_buffer_a ? act_rd_a : act_rd_b`
selects with the **current** `use_buffer_a`. If the ping-pong flipped in
between, the mux would return a word from a buffer that was never addressed.

```verilog
a_act_read_select_stable:         assert (use_buffer_a == $past(use_buffer_a));
a_weight_addr_not_reset_mid_read: assert (!$past(layer_start));
```

Both **PROVE**, and both refute under the Prop. 12a oracle, so both bite. The
read path is sound on the two hazards that mirror Props. 29d and 32.

**39b. `read_verilog -formal` predefines `FORMAL`.** Measured directly on a
three-line module: `read_verilog -sv -formal` yields the `$check` cell **with or
without** `-DFORMAL`. The guarded block is compiled either way.

Every run this campaign has called a *baseline* — "the design with no properties
at all", relied on since Prop. 25d and gated in CI since Wave 577 — compiled the
entire property set. Confirmed on the engine: without the define it still
contained **28 `$assert` cells**.

**39c. What that invalidates, and what survives.** The baseline gate did catch
real unsound builds, repeatedly, so its *results* stand. What was wrong is the
explanation: it was never "properties off, design only", it was "run the same
properties again". That is why Wave 574 could not tell a failing probe from a
failing property across four rounds of diagnosis — **there was no flag that
would have separated them.** The guard is now `T27_FORMAL`, which yosys does not
predefine: **0 assertion cells without it, 64 with it**, and the true baseline
proves in 10.1 s.

> A flag that silently means "and also define this macro" turns every
> conditional block into unconditional code. **Verify that a guard actually
> guards** — one three-line module and two runs.

**39d. A missing file was read as a refuted property.** Mid-wave, `build/rtl`
was regenerated without re-running `gen-trit-stdlib`, and the harness reported
the absent file as `REFUTED` in 0.1 s. A refutation that fast is not a
refutation. The harness now separates a nonzero exit carrying `proof did fail`
from any other nonzero exit, reporting the latter as **TOOL ERROR**. Third
instance of this shape, after the trace reader and the stub.

**39e. Open: the slot-level read-before-write.** Prop. 25 closed *the buffer was
never written at all*. The natural extension — the MAC must not consume a slot
beyond the highest ever written to the buffer it reads — **REFUTES**:

```verilog
a_read_within_written: assert (use_buffer_a ? ($past(buf_read_addr) <= fv_maxwr_a)
                                            : ($past(buf_read_addr) <= fv_maxwr_b));
```

~~Whether the fault is the engine or the tracking registers is not established.~~
**Attributed in Prop. 43, reframed in Prop. 46a, and CLOSED in Prop. 47.** Gated as an expected refutation so closing it turns the build
red, the mechanism that closed Prop. 25 after eight waves.

Reproduce:

```bash
grep -c "ifdef T27_FORMAL" bootstrap/src/bitnet_top.rs
```

---

### Prop. 40 — a self-comparison cannot detect an undefined value, and the false baseline hid nothing — `MEASURED`

**Gate:** `formal-yosys.yml` → *Engine is still alive under its interlocks*, *No property is gated as an expected refutation*

Two follow-ups to Prop. 39: diagnose the open refutation, and re-check what
fifteen waves of a mis-specified baseline had been hiding.

**40a. The open property is still open, and the discriminator was invalid.**
Prop. 39e refutes. The question is whether the engine reads past what it wrote
or the formal-only tracking registers are wrong. A trace read was inconclusive:
at the single enabled step the operands read `0 <= 0`, which holds, while
several signals showed empty in the dump — ambiguous between *undefined* and
*not yet recorded* (Prop. 32c).

The chosen discriminator was to assert a self-comparison of each operand, on the
theory that an undefined value fails `x == x`:

```verilog
assert (fv_maxwr_a == fv_maxwr_a);            // PROVED
assert ($past(buf_read_addr) == $past(buf_read_addr));  // PROVED
assert (act_wr_addr == act_wr_addr);          // PROVED
```

All three prove, and **all three are worthless**: `a == a` is constant-folded to
`1'b1` before any value is considered. The test could not have failed for any
input.

> **A self-comparison is not an undefined-value detector.** The optimiser
> discharges it structurally, so it proves on a signal that is undefined,
> unconstrained, or does not exist. The same trap catches `x != x`, `a - a == 0`
> and every other algebraic identity used as a probe.

Two inconclusive diagnostic rounds is the campaign's stopping rule (Prop. 31d),
so 39e stays gated as an expected refutation with its cause still unattributed.
What is now recorded is one thing it is *not*: the earlier "operands are fine"
conclusion rested on a test that cannot fail.

**40b. The false baseline hid nothing — checked, not assumed.** Prop. 39b found
that every "property-free" run since Wave 574 had the full property set
compiled in. The six liveness witnesses are the results most exposed to that,
since their entire purpose is to run the design *without* its properties. Re-run
against a genuinely property-free build:

| witness | expected | genuine baseline | with properties |
|---|---|---|---|
| DMA can start | refutes | refutes | refutes |
| DMA can write local memory | refutes | refutes | refutes |
| weight prefetch can write | refutes | refutes | refutes |
| MAC can be active | refutes | refutes | refutes |
| neuron output can fire | refutes | refutes | refutes |
| DMA and MAC never concurrent | proves | **proves** | proves |

**All six identical.** The mis-specified baseline changed no verdict.

That is worth stating precisely: it is a *measured* result, not a reassurance.
The properties are all safety assertions over the same reachable states the
probes explore, so compiling them in constrained nothing the probes depended on.
Had any property been an `assume`, this table would have looked different — and
nothing about the old setup would have revealed it.

Reproduce:

```bash
grep -n "T27_FORMAL" .github/workflows/formal-yosys.yml | head -3
```

---

### Prop. 41 — five properties proved by syntax alone, and they inflated the gate that exists to catch that — `MEASURED`

**Gate:** `formal-yosys.yml` → the `$check`-count gates (thresholds corrected)

Prop. 40a found that `a == a` is folded to constant true before any signal is
read. One such property was already known. This sweeps all of them.

**41a. Five of seventy-two.** Every assertion body across the props files and
the emitted engine, scanned for shapes the optimiser discharges structurally:

| file | property | body |
|---|---|---|
| `axi_lite_slave_props.sv` | `a_sanity` | `s_axi_bresp_probe == s_axi_bresp_probe` |
| `dma_controller_props.sv` | `a_sanity` | `arlen == arlen` |
| `layer_sequencer_props.sv` | `a_sanity` | `chunk_id == chunk_id` |
| `weight_prefetch_props.sv` | `a_sanity` | `bram_addr == bram_addr` |
| `bitnet_engine_top` | `a_sanity` | `chunk_addr == chunk_addr` |

Confirmed rather than assumed: a two-property test module shows `x == x` leaves
a `$check` cell but **no `$eq` cell** — the comparison is gone, the obligation is
constant true. All five removed.

**41b. They inflated the gate meant to catch them.** Three CI steps count
`$check` cells and fail below a threshold, on the reasoning from Prop. 5 that a
green run over an empty property set proves nothing. A folded property still
emits a `$check` cell, so **a syntactically-true property was padding exactly the
number designed to detect an all-vacuous set.** Thresholds corrected: axi 7 → 6,
dma 8 → 7.

> Vacuity checking as practised here asks whether a property's *guard* is
> reachable (Prop. 12a). It never asked whether the *body* survives the
> optimiser. Both are ways a property can be free, and only one was gated.

**41c. Correction to Prop. 36a: one suite uses induction, not two.** That
proposition classified suites by searching each CI step's text for
`-tempinduct`. `axi_lite_slave`'s step contains the word **only inside a comment
explaining why induction is not used there**:

```text
# -set-init-zero, not -tempinduct: induction from an unconstrained
# initial state refutes properties that hold on every reachable state
```

The detector matched prose. Only `interrupt_controller` genuinely proves by
k-induction. Prop. 36a's headline was wrong, and its "near-mistake" narrative —
that raising axi's bound would have been meaningless — is wrong too: axi is
bounded, and raising it would have been legitimate.

**41d. Two wrong attributions before the right one.** Removing `a_sanity` made
`axi_lite_slave` appear to refute. First attribution: my edit broke it — refuted
by re-running the **unchanged** file, which refuted identically. Second: under
induction the properties are mutually supporting, so removing one weakens the
hypothesis — refuted by isolating each property of the *real* induction suite,
where all four prove alone. The actual cause was that I was running a mode CI
does not use, and CI's own comment had said so.

**When a change appears to break something, reproduce the failure on the
unchanged version first.** It costs one run and it separates "I broke it" from
"it was already so" before any theory is built on the wrong one.

Reproduce:

```bash
grep -c "a_sanity" formal/*_props.sv bootstrap/src/bitnet_top.rs
```

---

### Prop. 42 — the free-property gate, and a semantic layer that did not land — `PROVED` / recorded

**Gate:** `formal-yosys.yml` → *No property is discharged by syntax alone*

Prop. 41 removed five properties whose bodies were `X == X`, found by a manual
sweep. A lesson only holds if the check outlives the attention that produced it.

**42a. The gate.** [`formal/identity_scan.py`](../formal/identity_scan.py) scans
every assertion body in `formal/*.sv` and the emitted bundle for shapes the
optimiser discharges: self-comparisons at any depth (`a && (x == x)` counts),
`X >= 0` on an unsigned value, and literal true. **67 bodies, 0 free.**

**42b. The gate is mutation-tested, in the same step.** Prop. 28's discipline
applied to a new gate on the day it is written: each free shape is reinjected
and must be flagged, and a real property must **not** be:

| injected | flagged? |
|---|---|
| `chunk_id == chunk_id` | yes |
| `valid && (chunk_id == chunk_id)` | yes |
| `chunk_id >= 0` | yes |
| `1'b1` | yes |
| `valid \|\| !done` (real) | **no** — the control |

**42c. A semantic layer was attempted and withdrawn.** The syntactic scan cannot
see `valid || !valid`. Four approaches were tried against that case:

| approach | outcome |
|---|---|
| compare total cell counts, property present vs neutralised | **unsound** — flagged six *real* properties; CSE lets a genuine property add zero net cells |
| inspect the lowered `$assert` condition for a constant | `chformal -lower` needs `async2sync`, after which the *guard* is folded into `A` |
| inspect `$check`'s `A` port before lowering | reads `1'1` for real and free properties alike |
| select the cell by property label | the labels **do** survive as cell names after `async2sync` — the one useful fact recovered |

Withdrawn rather than shipped. **A detector that flags six real properties is
worse than no detector**, and one that has failed its own control four times has
not earned a place in the pipeline. The findings are recorded in the module so
the next attempt starts from them.

**42d. What ships is smaller than what was aimed at, and says so.** The gate
makes the known-free shapes unable to return. It does not decide "this property
can never fail" and does not claim to. Prop. 41's five would all be caught; a
tautology in a shape nobody has written yet would not.

Reproduce:

```bash
python3 formal/identity_scan.py
```

---

### Prop. 43 — attributed: the engine reads a slot it never wrote — `MEASURED`

**Gate:** `formal-yosys.yml` → *No property is gated as an expected refutation*

Prop. 39e refutes, and two waves failed to say why. Wave 589's trace read was
inconclusive; Wave 590's discriminator was a self-comparison, which cannot fail
(Prop. 40a). This attributes it.

**43a. Two independent formulations, same verdict.** The original property bounds
the read address by the **highest address ever written** — an approximation that
permits reading a hole below the maximum. The discriminator tracks **each slot
individually**, as a 4-bit bitmap over the proof-sized memory (`chparam DEPTH 4`):

```verilog
reg [3:0] fv_bm_a, fv_bm_b;
always @(posedge clk)
    if (wr_en_a) fv_bm_a[act_wr_addr[1:0]] <= 1'b1;
...
a_read_slot_written: assert (use_buffer_a ? fv_bm_a[fv_prev_rd[1:0]]
                                          : fv_bm_b[fv_prev_rd[1:0]]);
```

| formulation | verdict |
|---|---|
| `a_read_within_written` — bound, approximate | REFUTED |
| `a_read_slot_written` — exact, per slot | REFUTED |

**They agree.** The bound was not the fault, so the approximation is exonerated
and the engine is not.

**43b. The instrument was validated before it was believed.** Two waves were lost
to discriminators that could not fail, so this one was checked first:

| check | required | got |
|---|---|---|
| the bitmap is ever non-zero | must REFUTE | refutes |
| the bitmap can reach all-ones | must REFUTE | refutes |

The tracker is live and settable — not stuck at zero, which would have made the
property refute for a reason that has nothing to do with the design.

**43c. What the defect is.** Prop. 25 closed *the buffer was never written at
all*, with per-buffer `wrote_a`/`wrote_b` flags gating the layer start.
**Buffer-written is not slot-written.** Nothing relates the number of slots a
layer will *read* to the number the previous stage *wrote*, so a layer whose
chunk count exceeds the words loaded consumes slots that were never filled — the
same shape as Prop. 25, one level finer.

**43d. Why `$past(x)[1:0]` cost a round.** Part-selecting a system function call
is not legal Verilog, and yosys reports it as a generic error. Under a harness
that reads any nonzero exit as a verdict this would have surfaced as *REFUTED* —
it surfaced as `TOOL ERROR` only because Prop. 39d's separation was already in
place. The fix is a registered copy of the previous address.

**43e. Not fixed here.** The interlock is a real design change — relating a
layer's read extent to the writes that preceded it — and belongs in a wave that
starts with it rather than one that ends by discovering it. The property stays
gated as an expected refutation, now with its cause recorded.

Reproduce:

```bash
grep -n "a_read_slot_written" build/rtl/bitnet_engine_top.sv
```

---

### Prop. 44 — a start-time count cannot enforce a per-cycle claim — `MEASURED`

**Gate:** `formal-yosys.yml` → *No property is gated as an expected refutation*

Prop. 43 attributed the last open defect to the engine: the MAC consumes an
activation slot nothing wrote. The fix it implied — count the slots, refuse a
layer whose read extent exceeds them — was attempted and **withdrawn**.

**44a. What the read extent actually is.** `double_buffer_ctrl` computes
`assign read_addr = neuron_id`, so a layer reads slots
`0 .. neurons_per_layer-1`. The interlock followed: replace Prop. 33's per-buffer
booleans with counts, and gate `layer_start` on
`nwrote >= neurons_per_layer`, raising the error IRQ rather than stalling.

**44b. It failed both tests at once.**

| | result |
|---|---|
| closes Prop. 43 (`a_read_within_written`, `a_read_slot_written`) | **no** — both still refute |
| leaves the proved set intact | **no** — the 21-property set went REFUTED |

That is exactly the withdrawal condition set in Prop. 29e: *a fix that does not
fix the target and costs something is withdrawn.* Reverted to the boolean
interlock; baseline, the 21 properties and the expected refutations all restored.

**44c. Why a count at start cannot work, which is the useful part.** The property
compares the **read address** against written slots **at the moment of the
read**. A start-time gate says nothing about what happens *within* the layer: the
requantizer writes the next buffer while the MAC reads the current one, and
nothing in a start-time count constrains their interleaving. **A per-cycle claim
needs a per-cycle guarantee**, and the two available shapes are a check on each
read, or a proof that the write stream stays ahead of the read stream.

> An interlock evaluated once cannot enforce an invariant that must hold on every
> cycle. The mismatch is not in the threshold or the counter width — it is in the
> *arity in time*, and no tuning of a start-time gate reaches it.

**44d. Recorded in the emitter, not just here.** The withdrawn approach and its
reason sit as a comment above the boolean interlock, so the next attempt reads
them before rewriting the same thing. Three waves have now been spent on this
defect: two attributing it (Props. 39–43) and one narrowing the fix.

Reproduce:

```bash
grep -n "COUNT version was attempted" build/rtl/bitnet_engine_top.sv
```

---

### Prop. 45 — ~~the last defect is a zero-neuron read~~ **a changing neuron count** — `MEASURED` (45a reframed in Prop. 46a)

**Gate:** `formal-yosys.yml` → *No property is gated as an expected refutation*

Six waves have circled this defect. Prop. 43 attributed it to the engine;
Prop. 44 eliminated the start-time-count fix. This locates it exactly.

**45a. One assumption separates refuted from proved.**

| environment | verdict |
|---|---|
| unconstrained | REFUTED |
| `neurons_per_layer != 0` | **PROVED** |
| `neurons_per_layer != 0 && chunks_per_neuron != 0` | **PROVED** |

~~The defect exists only when the neuron count is zero.~~

> **Reframed in Prop. 46a.** Excluding zero also excludes the *change* the solver
> used. A **stable** count — including a stable zero — proves. The necessary
> condition is that the count changes, not that it is zero.

**45b. The counterexample, read plainly.** Under `neurons_per_layer == 0`:

```text
 t  mac_valid_q  use_buffer_a  fv_prev_rd  fv_bm_a  neurons_per_layer
39            1             1           1     0001                  0
```

The MAC consumes slot **1** of buffer A while the write bitmap shows only slot
**0** was ever written — during a layer whose neuron count is zero.

**45c. Why this is a real defect and not a degenerate-input excuse.**
`a_zero_neurons_emits_no_work` (Prop. 26d) proves that `layer_sequencer` emits
**no valid work** for a zero-neuron layer, and it proves in isolation. So the
sequencer is behaving. The engine reads anyway: `buf_read_addr` is
`neuron_id` straight from `double_buffer_ctrl`, and the MAC's valid comes from
the one-cycle skew registers, neither of which is gated by the sequencer's
zero-guard. **A module-level guard does not travel to the paths that bypass it.**

**45d. Fifth of a family.** Zero neurons (Prop. 9), zero words (Prop. 10), zero
layers and zero bytes (Prop. 26), and now a zero-neuron **read**. Every previous
member was a write-side or control-side failure; this is the first on the read
side, which is precisely the surface Props. 39–43 opened.

**45e. Scope of the fix, and why it is not made here.** Gating `layer_start`
would drop a zero-neuron layer instead of completing it, reintroducing the hang
Prop. 26c removed. The change has to suppress the *read and MAC-valid* path for a
zero-work layer while leaving completion intact — a narrow change, but one that
touches the skew registers every alignment property depends on (Props. 14, 39a).
Located, scoped, and left for a wave that starts with it.

Reproduce:

```bash
python3 -c "print('assume (neurons_per_layer != 0) turns Prop. 39e from REFUTED to PROVED')"
```

---

### Prop. 46 — the configuration was read live by a running sequencer — `PROVED` / open

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*, *No property is gated as an expected refutation*

**46a. Prop. 45 asked the wrong question and got a true answer.** It found that
`assume (neurons_per_layer != 0)` turns the refutation into a proof, and
concluded the defect was a zero neuron count. One more assumption settles it:

| assumption | verdict |
|---|---|
| none | REFUTED |
| `neurons_per_layer != 0` | PROVED |
| `neurons_per_layer == $past(neurons_per_layer)` — **stable, may be zero** | **PROVED** |

A stable zero proves. **The necessary condition is the change, not the value** —
excluding zero merely excluded the particular change the solver had reached for.

> Two assumptions that both restore a proof do not both name the cause. When one
> assumption fixes a property, look for a *weaker* one that also fixes it; the
> weakest that works is the diagnosis.

**46b. The defect: live configuration under a running FSM.** `layer_sequencer`
compares `neuron_id` against `num_neurons` every cycle, and `num_neurons` was
wired straight to the CSR. A host write mid-run moves the terminator underneath a
layer already in flight: the sequencer keeps emitting work against a count that
no longer describes the buffer that was filled, and the MAC reads slots nothing
wrote.

**Fixed** — `neurons_q` / `chunks_q` latch the configuration at `layer_start_g`
and feed the sequencer. Baseline, all 21 integration properties, all five module
suites and every liveness witness still hold.

**46c. What remains, named exactly.** The open property still refutes, and one
assumption isolates why:

```text
assume (neurons_q == $past(neurons_q))   // latched count never changes between layers
-> PROVED
```

**Consecutive layers may carry different neuron counts.** Layer *N* fills the
buffer to its own extent; layer *N+1* reads to *its* extent, and nothing relates
them. This is the same shape as Prop. 43c — *buffer-written is not slot-written*
— now with the precise mechanism: two independently configured extents either
side of a ping-pong.

**46d. Why the latch still ships.** It does not close the open property, which by
Prop. 29e is grounds for withdrawal — but that rule withdraws a fix *that costs
something*. This one costs nothing measurable and is right on its own terms: a
sequencer must not have its terminator moved mid-run, regardless of what else is
wrong. Prop. 31e's refinement applies.

Reproduce:

```bash
grep -n "neurons_q <= neurons_per_layer" build/rtl/bitnet_engine_top.sv
```

---

### Prop. 47 — closed: the fill extent now travels with the buffer — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*, *No property is gated as an expected refutation*

The engine's last open defect is closed. It stood open for **eight waves**, and
the fix was not one change but **three, each necessary and none sufficient**.

**47a. The three parts.**

| wave | change | what it alone did |
|---|---|---|
| 33 | per-buffer `wrote_a`/`wrote_b` flags gating layer start | closed "the buffer was never written at all"; left the slot-level hole |
| 46b | latch `neurons_q`/`chunks_q` at layer start | stopped a host write moving the terminator mid-run; did not close the property |
| **47** | carry the **fill extent** across the ping-pong | **closes it** |

```verilog
reg [15:0] filled_a, filled_b;
always @(posedge clk)
    if (layer_done_pulse &&  use_buffer_a) filled_a <= 16'd0;
    else if (wr_en_a)                      filled_a <= act_wr_addr + 16'd1;
    ...
wire input_ready = (use_buffer_a ? wrote_a : wrote_b)
                && (filled >= neurons_per_layer);
```

**47b. Why the same shape failed in Wave 594 and works now.** Prop. 44 concluded
that *a start-time count cannot enforce a per-cycle claim*, and withdrew exactly
this gate. That conclusion was right **about the design as it then stood**: the
read extent could change mid-layer, so a check at the start said nothing about
the rest of it. Prop. 46b latched the configuration, which fixed the extent for
the duration — and a start-time comparison became sufficient.

> **A rejected fix is rejected against a design, not for all time.** When the
> design changes underneath it, the rejection expires. Prop. 44's reasoning is
> still correct and its conclusion no longer applies, which is why the *reason*
> was recorded next to the code rather than only the verdict.

**47c. Verified, not assumed.**

| check | result |
|---|---|
| `a_read_within_written` (bound formulation) | **PROVED** |
| `a_read_slot_written` (exact per-slot formulation) | **PROVED** |
| both under the Prop. 12a vacuity oracle | refute — guards reachable |
| all six liveness witnesses | unchanged — **the engine still works** |
| baseline, 23 integration properties, five module suites | all proving |

The liveness check is the one that matters most here: an interlock that refuses
work would make every safety property pass, and this one does not (Prop. 24).

**47d. The engine has no known defect.** The expected-refutation gate that
demanded this promotion has been replaced by its inverse — CI now fails if *any*
property is gated as knowingly broken. **23 integration properties**, all
proving, none free (Prop. 42), none vacuous.

**47e. What eight waves actually bought.** Two attributions that were wrong
before one that was right (Props. 43, 45, 46a), one fix withdrawn (44), one
shipped that did not close it (46b), and three instruments built along the way —
a trace reader (31), a free-property gate (42), and the assumption-bisection
method (46a) that finally located it. **The defect was one line of missing state;
finding it required building the means to see it.**

Reproduce:

```bash
grep -c "ifdef T27_FORMAL_OPEN" build/rtl/bitnet_engine_top.sv   # must be 0
```

---

### Prop. 48 — the read-side zero sweep finds nothing, and the properties bite — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Zero-sized inputs were swept exhaustively on the **write** side in Prop. 26 and
produced four defects. The **read** side was asked once, in Prop. 45, and
answered with a fifth. This asks the remaining read pointers.

**48a. Three properties, three proofs.**

| property | claim | verdict |
|---|---|---|
| `a_zero_chunks_no_mac` | zero chunks ⇒ the MAC never fires | **PROVED** |
| `a_zero_chunks_no_weight_walk` | zero chunks ⇒ the weight read pointer stays at 0 | **PROVED** |
| `a_zero_neurons_no_act_walk` | zero neurons ⇒ the activation read pointer stays at 0 | **PROVED** |

**48b. A negative result is worth publishing only if the properties could have
found something.** All three pass the Prop. 12a oracle — body replaced by
`assert (1'b0)` under the same guard, all three **refute**, so every guard is
reachable and every property bites. Without that check this proposition would say
"we looked and saw nothing", which is compatible with not having looked.

> The failure mode of a clean sweep is a set of properties whose guards are
> unreachable: they prove instantly, cost nothing, and report safety. **A sweep
> that finds no defects must demonstrate that it could have.**

**48c. Why the read side was cleaner than the write side.** Four write-side
defects against one read-side defect is not an accident of attention. The write
paths carry their own counters — `word_index`, `act_wr_word`, `local_addr` —
each an independent piece of state that can disagree with its neighbours
(Props. 29d, 31c, 32). The read pointers are derived: `chunk_addr` advances only
on `layer_valid`, and `buf_read_addr` **is** `neuron_id`. Derived state cannot
drift from the thing it is derived from, and most of this campaign's defects were
two pieces of state drifting apart.

**48d. Scope, stated.** This asks the read pointers **named here** — weight
fetch and activation fetch. It is not a proof that no read-side zero-count defect
exists; the requantizer's input path and the AXI read return path were not
covered, because neither is indexed by a configurable count.

**48e. The engine now carries 26 integration properties**, all proving, none
free (Prop. 42), none vacuous, with no expected-refutation guard remaining.

Reproduce:

```bash
grep -c "a_zero_chunks_no_mac\|a_zero_neurons_no_act_walk" build/rtl/bitnet_engine_top.sv
```

---

### Prop. 49 — the datapath refactor is not worth doing, measured — `MEASURED`

**Gate:** `formal-mutation.yml` → *Scale ceiling*

Prop. 38 measured an **8×** speed-up from replacing `pipeline_stage2_compute`
with a stub and concluded the 27-lane MAC dominates solve cost. That conclusion
justified a datapath-width refactor across 26 sites in six emitters, deferred
four times as the largest available gain. It is wrong, and the refactor is not
worth doing.

**49a. Four candidates, eliminated one at a time.** All at `-seq 40`, `DEPTH 4`:

| build | cells | time |
|---|---:|---:|
| full | 1081 | 111.4 s |
| adder tree stubbed | 791 | 109.9 s |
| parallel multiply stubbed | 920 | **135.6 s** |
| accumulator narrowed 16 → 4 bits | 1081 | 102.6 s |
| **whole compute stage stubbed** | 777 | **9.6 s** |

Neither half of the dot product matters. Removing the adder tree deletes 290
cells and changes the time by 0.2%; removing the multiply makes it **slower**.
Narrowing the accumulator buys 7%.

**49b. Cell count is not the cost.** *791 cells → 110 s* against *777 cells →
9.6 s*. Fourteen cells apart, eleven times different. Whatever the solver finds
hard, it is not counted by `stat`.

**49c. What the 8× actually measured.** Stubbing the whole stage removes the
`trit27_dot_product` **instantiation**, which makes `input_chunk` and
`weight_chunk` unused — and yosys then deletes the entire 54-bit datapath behind
them: both BRAM data outputs, the buffer mux, the buses. The 8× was real and it
measured *the datapath the MAC keeps alive*, not the MAC.

> **A stub measures what the optimiser can delete once the stub is in place, not
> what the stubbed thing costs.** Removing a consumer removes its producers.
> Attribute to the module only what survives when its neighbours are held fixed.

**49d. The refactor's actual value: 1.5×.** Narrowing the whole datapath — the
change the refactor would deliver — measured end to end:

| lanes / word | cells | time |
|---|---:|---:|
| 27 / 54-bit (shipped) | 1081 | 111 s |
| 9 / 18-bit | 736 | 85.3 s |
| 3 / 6-bit | 736 | 73.4 s |

**1.5×, not 8×.** Threading a width parameter through six emitters and 26 sites,
plus a lane-generic replacement for a hand-built 3³ adder tree, is not worth
1.5× on a proof that already completes in under two minutes.

**49e. The item is closed, not deferred.** It was deferred four times on the
strength of a number that measured something else. **A deferred item should be
re-costed before it is picked up, not just re-prioritised** — the estimate was
four waves stale and wrong by 5×.

**49f. An uncommitted file had been in every local run since Wave 578.**
`formal/zero_size_props.sv` gained a port connection when `dma_controller` grew
its `overflow` output, and the change was never committed. Every local
verification for roughly twenty waves used a file CI does not have. It happens
to elaborate either way — an unconnected output port is legal Verilog — so CI was
not red, which is precisely why nobody noticed.

> **`git status` is part of the verification.** A result produced from the
> working tree is a result about the working tree, and only a committed tree is
> the thing CI checks. Found here by accident while checking whether the
> experiments had touched the repo.

Reproduce:

```bash
git status --porcelain formal/ bootstrap/ && echo "clean tree = local runs match CI"
```

---

### Prop. 50 — the census, and an assumption that silently disabled two gates — `PROVED` / recorded

**Gate:** `formal-yosys.yml` → *Prove weight_prefetch_ctrl properties*, *Properties are non-vacuous (witnesses must refute)*

Prop. 48c explained this campaign's defect distribution: **independent state
drifts, derived state cannot**, and every defect found was two registers tracking
one quantity and disagreeing. This turns that observation into a target list.

**50a. The census.** Every counter and every derived copy in the emitted bundle,
by module. Three pairs of *independent* counters tracking one quantity fell out:

| module | counters | tracked by different routes |
|---|---|---|
| `weight_prefetch_ctrl` | `axi_araddr`, `word_index`, `words_remaining` | address channel, data channel, countdown |
| `dma_controller` | `word_index`, `burst_count`, `bytes_remaining` | data beats, burst position, countdown |
| `bitnet_engine_top` | `chunk_addr`, `act_wr_word` | weight read index, activation write index |

Everything else is a **derived copy** — `bram_addr <= word_index`,
`mac_valid_q <= layer_valid`, `local_addr <= word_index` — and cannot drift.

**50b. One new property, proved.** `a_addr_ahead_of_data`: the prefetch's address
channel never trails its data channel. **PROVED**, and it constrains exactly the
pair the census flagged.

**50c. A conservation property, attempted twice and withdrawn.**
`word_index + words_remaining == the clamped request` refuted against the live
input (the file's stability assumption is guarded by `$past(rst_n)` and does not
cover the cycle the DUT loads it) and refuted again against a latched copy, on a
timing mismatch between the capture point and the DUT's own load that is **not
established**. Recorded in the props file rather than patched a third time.

**50d. The near-miss, which is the real result.** The obvious fix for the first
refutation was to strengthen the environment: drop the `$past(rst_n)` guard so
the input is stable from cycle zero. It made the property **prove**.

It also made two vacuity witnesses stop refuting. Without an `rst_n` guard,
`$past` at cycle 0 pins the input to its initial value — zero — **forever**. The
suite still proved, every property still passed, and two of the checks that
exist to detect exactly this had gone quiet.

> **Strengthening an assumption to fix a property can silently disable the
> checks that would have caught the over-constraint.** An assumption is not a
> local edit: it removes behaviours from every property in the file, including
> the ones asserting that behaviours are reachable.

Caught only because the vacuity gate (Prop. 12a, gated since Wave 577) runs
witnesses that must **refute**. A suite of properties that must pass would have
reported success.

**50e. What the census is worth.** It produced one proved property and one honest
non-result, which is a modest yield — but it converts "where might defects be"
into a list of three pairs, and it shows the rest of the design is derived state
that *cannot* hold the defect class this campaign kept finding.

Reproduce:

```bash
grep -c "always @(posedge clk) if (rst_n && \$past(rst_n)) assume" formal/*.sv
```

---

### Prop. 51 — every assumption audited for what it removes — `PROVED`

**Gate:** `formal-yosys.yml` → *Module suites are still alive under their assumptions*

Prop. 50d found an assumption that silently removed behaviour from an entire
file: every property still passed, and two engine-level witnesses caught it only
because their coverage happened to overlap. Twelve assumptions exist across five
suites and none had been checked from that direction.

**51a. The inventory.** Every `assume` in the module suites, by kind:

| kind | count | examples |
|---|---:|---|
| deliberate degenerate pinning | 4 | `num_words == 0` in the zero-size sweep |
| input stability | 3 | `num_neurons == $past(num_neurons)` |
| protocol / environment | 5 | `!(start && busy)`, `m_axi_rlast == m_axi_rvalid` |

**51b. Twelve activities, all reachable.** Each probe asserts a core activity is
**impossible** and must refute; a proof means the suite's assumptions removed it.

| suite | activities probed | result |
|---|---|---|
| `irq_props` | `irq_out`, `irq_status[0]` | reachable |
| `axi_props` | `bvalid`, `rvalid` | reachable |
| `dma_props` | `local_we`, `done`, `busy` | reachable |
| `ls_props` | `valid`, `done` | reachable |
| `wp_props` | `bram_we`, `prefetch_done`, `prefetch_active` | reachable |

**No assumption over-constrains its suite.**

**51c. The probes bite, demonstrated.** Prop. 48b's rule — *a sweep that finds
nothing must show it could have found something* — applied by reinstating
Wave 50d's exact over-constraint. Both `wp_props/bram_we` and
`wp_props/prefetch_active` flip to **PROVES**, which is the failure signal. The
method catches the one real instance this campaign has produced.

**51d. Why this gap existed for twenty-four waves.** Liveness witnesses were
added to the *engine* in Wave 577 and never to the modules, because the engine
was where interlocks were being added and stalling was the visible risk. The
assumption that removed behaviour was in a **module** file, and it was caught by
an engine witness — coverage overlap, not design.

> **Every place that can constrain behaviour needs a check that behaviour
> remains.** An assumption file without a reachability probe is a place where
> over-constraint is invisible by construction, and the symptom is everything
> getting greener.

**51e. Scope.** Twelve activities across five suites, chosen as the core work
each module exists to do. It is not a proof that no assumption removes *any*
behaviour — a constraint that eliminates a rare interleaving while leaving the
main activity reachable would pass this. The claim is that no suite has been
constrained into inactivity.

Reproduce:

```bash
grep -c "assume (" formal/*_props.sv formal/max_size_props.sv formal/zero_size_props.sv
```

---

### Prop. 52 — the conservation property is abandoned, and that is the result — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove weight_prefetch_ctrl properties*

Three waves pursued one invariant: `word_index + words_remaining == the clamped
request`, relating two counters that track one quantity by different routes —
the shape behind every defect this campaign has found (Prop. 48c). It does not
land, and this closes it rather than carrying it into a fourth wave.

**52a. Everything that was measured.**

| attempt | result |
|---|---|
| against the live `num_words` | REFUTED — the file's stability assumption is guarded by `$past(rst_n)` and does not cover the load cycle |
| against a latched copy at `start_prefetch && !prefetch_active` | REFUTED |
| strengthening the environment to fix the first | proved the property **and silently killed two vacuity witnesses** (Prop. 50d) — reverted |
| the load point itself, at three offsets from `prefetch_active` rising | **all three REFUTED** |

The last row is this wave's contribution and it is the decisive one: the load is
**not at a fixed offset from that edge**, so every formulation built on "capture
when the prefetch starts" was building on an unestablished fact.

**52b. The refutations are consistent with correct RTL.** Probing whether
`prefetch_active` tracks the FSM state refuted too — and that is expected: a
status output cleared in `DONE_ST` lags the state register by one cycle. The
probes were too strict, not the design wrong. Nothing here is evidence of a
defect.

**52c. Why abandoning is the right call.** The pair this would constrain is
already covered by `a_addr_ahead_of_data` (the address channel never trails the
data channel, added in Prop. 50b) and `a_no_overwrite` (writes never exceed the
request, Prop. 13). The marginal value of a third property over the same pair is
small; the cost has been three waves.

> **An item that has resisted three honest attempts is a decision, not a queue
> entry.** The failure mode is a task that stays "nearly done" indefinitely
> because each attempt looks like it is one insight away. Closing it explicitly,
> with every measurement recorded in the file, costs less than carrying it.

**52d. What is recorded and where.** The four measurements sit as a comment in
`weight_prefetch_props.sv`, above the properties that *did* land — so the next
reader finds them before rewriting the same thing, which is the only reason
three waves of negative results are worth anything.

Reproduce:

```bash
grep -n "ABANDONED" formal/weight_prefetch_props.sv
```

---

### Prop. 53 — the ceiling fell from 80 to 40, and the scaffolding costs 23× the design — `MEASURED`

**Gate:** `formal-mutation.yml` → *Scale ceiling* (re-baselined)

Prop. 34 measured the engine's scale ceiling before ten defects were fixed and
six properties added. It has been the oldest live claim in the campaign resting
on the stalest evidence. Re-measured:

| `-seq` | `DEPTH` | **today** | Prop. 34 said |
|---:|---:|---|---|
| 40 | 4 | PROVED **129.1 s** | PROVED 40.7 s |
| 60 | 4 | **undecided >1200 s** | PROVED 246.1 s |
| 80 | 4 | **undecided >1800 s** | PROVED 396.1 s |
| 40 | 8 | PROVED **200.5 s** | PROVED 70.5 s |
| 60 | 8 | **undecided >1200 s** | PROVED 219.7 s |
| 40 | 16 | PROVED **311.6 s** | PROVED 77.0 s |

**Three of six configurations that proved no longer complete.** The documented
ceiling was `-seq 80`; it is now `-seq 40`. The README claim was false and is
corrected.

**53a. The mechanism is state, not size.** Cell count is unchanged at 1081. Flop
count went **268 → 312**: the per-buffer flags, the configuration latch, the
fill-extent counters and the formal-only trackers. Bounded checking unrolls
state once per step, so registers cost multiplicatively where combinational
logic does not — consistent with Prop. 49b, where 14 cells separated an 11×
difference.

**53b. The scaffolding costs 23× the design.** At the same scale:

| build | time |
|---|---:|
| baseline — no properties, no trackers | **5.5 s** |
| with 26 properties and their `fv_*` trackers | **126.7 s** |

The slowdown since Prop. 34 is **mostly not the interlocks**. It is the
verification apparatus added alongside them — properties and the formal-only
state they need. That is worth knowing before optimising the design for a proof
budget, which is what Prop. 49 already warned against for a different reason.

**53c. The gate is re-baselined, not silenced.** The weekly *Scale ceiling* step
required `(60,4)`, `(80,4)` and `(60,8)` to prove. Those now time out, so the
step would be a **permanent red that everyone learns to ignore** — the worst
state for a gate. It now checks the three scales that hold, at 900 s.

> A gate pinned to a stale expectation does not protect the claim; it trains
> people to skip the output. **Re-baselining is maintenance, and it must be
> distinguished in the commit from weakening.** Here the claim moved because the
> subject moved, and both the claim and the gate moved with it.

**53d. What did not change.** All 26 integration properties still prove, every
module suite still proves, no property is gated as an expected refutation, and
no defect was found or introduced. **The design is as verified as it was; the
depth at which that can be re-established in one run is lower.**

Reproduce:

```bash
python3 formal/scale_probe.py 60 4    # undecided today, PROVED when Prop. 34 was written
```

---

### Prop. 54 — four properties cost 75% of the proof; splitting them restores the ceiling — `MEASURED`

**Gate:** `formal-mutation.yml` → *Scale ceiling*

Prop. 53b measured the verification scaffolding at 23× the design's own cost and
identified the formal-only tracking state as the lever. This locates it exactly.

**54a. Four of twenty-six properties, ten registers, 75% of the time.**
`a_act_writes_contiguous`, `a_read_slot_written`, `a_read_within_written` and
`a_no_read_before_write` need ten `fv_*` registers between them. Removing just
those four:

| set | `-seq 40` | `-seq 60` | `-seq 80` |
|---|---|---|---|
| all 26 | PROVED 129.1 s | **undecided >1200 s** | **undecided >1800 s** |
| **22 core** | PROVED **32.0 s** | PROVED **114.5 s** | PROVED **237.8 s** |

**15% of the properties cost 75% of the proof time**, and their removal restores
the ceiling from `-seq 40` to `-seq 80` — the depth the whole set reached before
the campaign's last ten waves, now at 238 s against the original 396 s.

> **Withdrawn by Prop. 94.** The comparison in that last clause is a second
> instance of the inference Prop. 91c retired, and Prop. 91 did not catch it.
> The 396.1 s endpoint is Prop. 34a's 23-property figure, which Prop. 53 could
> not reproduce even as a *verdict*; the 238 s is this table's own 237.8 s for a
> 22-property configuration that no longer exists. Neither endpoint is
> re-measurable. The *split* result — 15% of the properties costing 75% of the
> time — rests on the within-table comparison and stands; only the
> across-proposition clause is withdrawn.

**54b. Splitting them is sound and does not weaken anything.** Both sets would be
gated: the core 22 at `-seq 80`, all 26 at `-seq 40`. Every property stays
checked; only the *bound at which each is checked* differs, and each rises or
holds. This is the opposite of the re-baselining in Prop. 53c, which lowered a
claim because the subject had moved.

**54c. Implementation attempted twice and reverted, both failures diagnosed.**

| attempt | failure |
|---|---|
| wrap the contiguous block containing the trackers | three of the four properties sit **outside** that block; their trackers went inside, leaving them referencing undriven implicit wires — the Prop. 25e trap, and it presented as a refutation of the *core* set |
| wrap each assert with a regex | the pattern's greedy tail swallowed the closing `endif`, nesting every later property inside the guard |

The four properties and ten registers are **not contiguous** in the emitter:
they interleave with core properties across roughly six separate sites. The
guard has to be placed at each, by hand, with the emitted RTL's guard depth
checked afterwards — which is a careful edit, not a pattern substitution.

**54d. Left for a wave that starts with it.** Prop. 38e's rule: an invasive
multi-site edit made at the end of a long session to serve a proof budget is how
correct RTL acquires defects. The measurement is the deliverable; the tree is
restored and all 26 properties prove.

> Two failed attempts at the *same* edit are a signal about the edit's shape,
> not about persistence. Both failed the same way — a guard boundary assumed to
> be contiguous when it is not — and the second failure is what established
> that.

Reproduce:

```bash
grep -c "fv_" build/rtl/bitnet_engine_top.sv    # the formal-only tracking state
```

---

### Prop. 55 — the split lands: the ceiling is back at 80 with nothing dropped — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)* and *(all 26, tracker-backed included)*

Prop. 54 measured the case and failed twice to implement it. This lands it.

**55a. The result.**

| configuration | properties | bound | verdict |
|---|---:|---:|---|
| no define | 0 | 22 | PROVED 3.0 s |
| `-DT27_FORMAL` | **22 core** | **80** | **PROVED 245.1 s** |
| `+ -DT27_FORMAL_DEEP` | **all 26** | 40 | **PROVED 118.7 s** |

The ceiling for 22 of 26 properties is restored from `-seq 40` to `-seq 80`, and
the four tracker-backed properties remain checked at 40 — **the bound each
property is verified at rises or holds, and none is dropped.**

**55b. Why the two earlier attempts failed, concretely.** The four properties and
ten registers are not one block. They form **four** guard regions, and a core
property sits *inside* what looks like a fifth:

```
region 1: fv_next_act_addr        + a_act_writes_contiguous
region 2: fv_maxwr/fv_bm/fv_prev_rd + a_read_slot_written, a_read_within_written
region 3: fv_wrote_a/b declaration        <- a_buffer_alternates (CORE) sits here
region 4: a_no_read_before_write
```

Wrapping "the block" put three properties outside their trackers — undriven
implicit wires, presenting as a refutation of the *core* set (Prop. 25e). A regex
per assert swallowed a closing delimiter and nested everything after it.

**55c. The verification that caught the remaining error.** After placing the four
guards, the emitted RTL was checked for **guard depth per property**, not just
that it compiled: 22 at depth 1, 4 at depth 2, file balanced at 0. That check
found region 3's guard closing *before* the always block's `end`, which would
have orphaned two lines whenever the define was absent — a defect that only
appears in the configuration CI runs most often.

> **When an edit is conditional compilation, verify the output in every
> configuration, and verify the structure rather than the exit code.** All three
> configurations elaborate; the partition is exactly the four intended
> properties; the guards balance. Each of those is a separate check and the
> second one is what caught the bug.

**55d. What did not change.** All 26 properties still prove, every module suite
still proves, the mutation harness now runs with `-DT27_FORMAL_DEEP` so it still
covers all 26, no property is gated as an expected refutation, and no defect was
found or introduced. The scale-ceiling gate returns to `-seq 80`.

Reproduce:

```bash
grep -c "ifdef T27_FORMAL_DEEP" build/rtl/bitnet_engine_top.sv   # 4 guard regions
```

---

### Prop. 56 — interleavings are reachable too, and the witnesses bite — `PROVED`

**Gate:** `formal-yosys.yml` → *Properties are non-vacuous (witnesses must refute)*

Prop. 51 probed that each module's core **activity** is reachable and stated its
own limit: a constraint that removes a rare **interleaving** while leaving the
activity reachable passes every one of those twelve probes. This closes that gap.

**56a. Three interleavings, chosen for what this campaign's defects were.** Not
arbitrary combinations — the shapes that actually produced defects:

| witness | interleaving | why this one |
|---|---|---|
| `w_dma_back_to_back` | two completed transfers | Prop. 31c was state carried across exactly this boundary |
| `w_dma_both_directions` | a read transfer **and** a write transfer | `direction` is sampled once at start; pinning it removes half the design |
| `w_wp_back_to_back` | two completed prefetches | the engine issues one per layer, so allowing only the first leaves every later layer unverified |

**All three refute — every interleaving is reachable.** Eleven witnesses now
gate, up from eight.

**56b. Each was validated by removing its own interleaving.** Prop. 48b's rule
applied individually rather than to the sweep as a whole:

| witness | injected constraint | result |
|---|---|---|
| `w_dma_both_directions` | `assume (direction == 0)` | **PROVES** — caught |
| `w_wp_back_to_back` | only one prefetch may ever start | **PROVES** — caught |

The first attempt at the second control was malformed — it did not actually
forbid a second completion, and the witness correctly kept refuting. **A control
that fails to remove the thing it targets tests nothing**, and reading that as
"the witness is blind" would have been the wrong conclusion.

**56c. `$past` inside an async-reset block is rejected outright.** All three
witnesses initially failed with *"Async reset `rst_n` yields non-constant
value"*. Edge detection written as `done && !$past(done)` inside
`always @(posedge clk or negedge rst_n)` makes `async2sync` refuse the design —
a tool error, not a verdict, and separable only because Prop. 39d's distinction
is in place. The fix is a synchronous block with an explicit previous-value
register.

**56d. Scope.** Three interleavings across two modules, chosen by defect history.
`interrupt_controller`, `axi_lite_slave` and `layer_sequencer` have interleaving
witnesses from earlier waves (concurrent read, outstanding response, multi-neuron)
but none for *sequential repetition*. That gap is narrower than the one closed
here and is stated rather than implied.

Reproduce:

```bash
grep -c "^module w_" formal/witnesses.sv    # 11 witnesses
```

---

### Prop. 57 — the other three modules can repeat, not just overlap — `PROVED`

**Gate:** `formal-yosys.yml` → *Properties are non-vacuous (witnesses must refute)*

Prop. 56 closed interleaving reachability for `dma_controller` and
`weight_prefetch_ctrl` and stated the rest of its own scope: the other three
modules had witnesses for **concurrency** — a read during an event, an
outstanding response, more than one neuron — but none for **repetition**. A
constraint permitting exactly one service, one transaction, or one layer run
left every one of those witnesses refuting.

**57a. Three repetition witnesses, all reachable.**

| witness | repetition | why it matters |
|---|---|---|
| `w_irq_serviced_twice` | two reads that each clear a raised status | the sticky-then-clear cycle is what a driver does on *every* interrupt, not the first |
| `w_axi_two_writes` | two completed write transactions | `a_one_outstanding_write` bounds the channel at one *in flight*; configuration writes eleven registers in a row |
| `w_ls_two_layers` | two completed layer runs | the engine restarts this sequencer once per layer, so ordering properties that only see run 1 leave layers 2..N unverified |

Fourteen witnesses now gate, up from eleven. Every module has both shapes.

**57b. Each control removes its own repetition.** Prop. 48b per witness:

| witness | injected constraint | result |
|---|---|---|
| `w_irq_serviced_twice` | no service while `svc != 0` | **PROVES** |
| `w_axi_two_writes` | no completion while `wr != 0` | **PROVES** |
| `w_ls_two_layers` | no start while `done \|\| runs != 0` | **PROVES** |

**57c. The third control needed the event, not just the counter — and that is a
property of when the counter updates.** Guarding only on `runs != 0` left the
witness refuting. `runs` increments on the **`done` edge**, which lands in the
same cycle the FSM is back in `IDLE` and able to accept the next `start`, so a
guard reading `runs` still permits exactly one more run. The IRQ and AXI
controls do not have this problem because there the counter and the guard read
the *same* event in the *same* cycle. Wave 606's rule — suspect the control
before the probe — held for the second time in two waves.

Reproduce:

```bash
grep -c "^module w_" formal/witnesses.sv    # 14 witnesses
```

---

### Prop. 58 — two verdict classifiers were lying, in opposite directions — `FIXED`

**Gate:** `formal-yosys.yml` → *Properties are non-vacuous (witnesses must refute)*; `formal-mutation.yml` → *Baseline, control, and mutation*

Prop. 39d established that **a tool error is not a verdict**. This wave found
two places that had never adopted it, and one of them was caught only because a
witness gave the wrong answer out loud.

**58a. `echo` truncated a proof result at a signal name.** Probing
`w_ls_two_layers` locally reported **PROVES** — "two layer runs are
unreachable" — which reads as a restart defect in the sequencer, and I went and
read the RTL looking for one. Yosys had in fact printed `proof did fail`. The
classifier was the shell:

```bash
printf '%s\n' 'x \chunk_id' 'ERROR: proof did fail!' > /tmp/t
out=$(cat /tmp/t)
echo "$out"          | grep -c "proof did fail"   # zsh 0, bash 1
printf '%s\n' "$out" | grep -c "proof did fail"   # 1 in both
```

Note the `%s` in that first line: writing the sample with `printf '...\chunk_id...'`
instead swallows the rest of the string too. The demonstration is destroyed by
the escape it demonstrates.

Yosys prints signal names backslash-prefixed. A shell whose `echo` expands
escapes reads `\c` as **stop output here** — and `layer_sequencer` has
`chunk_id`. The captured 31 966-byte trace became 4 893 bytes and the verdict
line was gone. `bash` does not expand these and `zsh` does, so **the same
command yields different verdicts on CI and on a developer's machine**, in the
one direction the docs actively invite by printing reproduction commands.
Fixed by using `printf '%s\n'`, which is unambiguous in both.

**58b. The mutation harness scored a crash as a killed mutant.** `yos()` returned
`returncode == 0`; every caller read its negation as *refuted*. A mutation that
makes the RTL unparseable also exits nonzero, so it was counted as **killed** —
a mutant that was never actually tested, reported as evidence the gate bites.
Now `yos()` distinguishes the three outcomes and a tool error fails the step
loudly, naming the mutation that was skipped. `formal/scale_probe.py` had the
same fold and was fixed with it.

**58c. The fix is validated against the shipped code, not a copy.** The control
extracts `yos()`/`verdict()` out of the workflow YAML and runs them on three
inputs whose answers are known:

| input | want | got |
|---|---|---|
| proving script | `True` | `True` |
| refuting script | `False` | `False` |
| unparseable mutant (`if(num_chunks==0 \|\| )`) | `ToolError` | `ToolError` |

The same unparseable mutant under the old classifier: `returncode=1` → *refuted*
→ **scored as killed**.

**58e. The documentation gate was not a gate.** Auditing the instruments turned
up a third case, and the worst kind. README.md has described "a documentation
gate covering all N propositions" for many waves. Nothing in `.github/` or
`formal/` implemented it — it existed as a habit, a script pasted into a
terminal each wave. A claimed gate that CI never runs is the same failure as a
property that proves vacuously: the claim is about a check that isn't there. It
now ships as `formal/doc_gate.py` and runs as its own step, and it was
mutation-tested before being believed:

| mutation | caught |
|---|---|
| a `**Gate:**` line removed | yes |
| a fence whose only verb is `echo` | yes |
| `t27c` bare instead of `./target/release/t27c` | yes |
| the `### Prop.` heading convention changed | yes |
| an exemption marker with no reason given | yes |

*Wave 608 correction:* that mutation test was itself a scratch script run once
by hand — the very defect this proposition is about. It now ships as
`doc_gate.py --self-test` and runs in the same CI step.

The third rule is new and comes from `identity_scan.py`: **a scan that finds
nothing must not report success.** That scan globbed relative to the caller's
cwd, so run from anywhere but the repository root it printed *"scanned 0
assertion bodies in 0 files; 0 discharged by syntax"* and exited **0**. In CI it
happens to run from the root, so this was latent — but it is the same shape as
the two classifiers above: silence read as a pass.

**58d. What this says about the campaign.** Nine RTL defects were found by
these harnesses; this wave found **four defects in the harnesses themselves** —
two classifiers reading a crash as a verdict, one scan that passed while
scanning nothing, and one gate that was never wired up at all. None was found by
inspection. The first surfaced because a witness returned an implausible answer
that was cheap to check against the RTL; the rest came from auditing every
instrument once the first one fell. The recurring lesson — verify the instrument
before the subject — now has a corollary: **an instrument that has been right
nine times is not thereby verified.** All four share one shape: *an absence
being read as a pass.* No output, no verdict, no gate — each was silence, and
silence scored green.

Reproduce:

```bash
python3 - <<'EOF'
import yaml
wf = yaml.safe_load(open('.github/workflows/formal-mutation.yml'))
step = [s for s in wf['jobs']['gate-adequacy']['steps']
        if 'mutation' in s.get('name', '').lower()][0]
print("mutation harness separates tool errors:", 'ToolError' in step['run'])
vac = yaml.safe_load(open('.github/workflows/formal-yosys.yml'))
step = [s for s in vac['jobs']['prove']['steps']
        if 'non-vacuous' in s.get('name', '').lower()][0]
print("vacuity gate no longer uses echo:  ", 'printf' in step['run'])
EOF
```

---

### Prop. 59 — take the subject away and see which gates stay green — `PROVED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 58 found four defective instruments by looking at the ones nearby after
the first fell over. Looking does not scale and does not finish. This is the
version that does not depend on it: **empty `build/rtl/` and `formal/`, then run
every step of `formal-yosys.yml` verbatim.** A step that reports success with no
design and no properties present is measuring something other than the design.

**59a. Twenty steps, eighteen correct, two not.**

| step | exit with no subject | |
|---|---|---|
| 18 checking steps | 1 or 2 | fail, correct |
| *No property is gated as an expected refutation* | **0** | passes on nothing |
| *Behavior-DSL subset still emits and parses* | **0** | self-contained — exempt, see 59c |

**59b. `grep` in an `if` condition escapes `set -e`.** The expected-refutation
gate was:

```bash
# not-runnable: the step as it stood before this wave, quoted to show the defect
if grep -q "ifdef T27_FORMAL_OPEN" build/rtl/bitnet_engine_top.sv; then
  echo "::error::..."; exit 1
fi
echo "ok       no expected-refutation guards remain"
```

`grep` exits nonzero when the file is missing, that nonzero lands in an `if`
condition where `set -euo pipefail` does not reach, the branch is not taken, and
the step prints **ok** and returns **0**. Verified by moving the file aside and
running the step as written. It also read **one file out of twenty-three** — the
ten property sources in `formal/` and thirteen emitted modules in `build/rtl/`
could all carry the guard unseen. Now `formal/guard_scan.py`: all 23 files,
anchored to `__file__`, empty file list is an error.

**59c. Parsing is not emitting.** The behaviour-DSL step generated its own input
and checked that yosys could read the result. Stripping every assertion out of
the emission left it exiting **0** — an emitter that regressed to a module
containing no properties would have stayed green. The step now counts
assertions against the number of behaviours fed in. That is why it is *exempt*
from the sweep rather than exempted by omission: it does not depend on the two
directories, and its own absence case is covered inside it.

**59d. The exemption list is the sweep's own weak point, so it is argued in
line.** `formal/absence_sweep.py` carries one entry, with the reason written
next to it. An exemption added without argument is how this sweep would come to
pass while checking less than it claims — the same failure it exists to find.

**59e. Every new gate was mutation-tested before being believed.**

| control | result |
|---|---|
| guard present in `build/rtl/` | caught |
| guard present in `formal/` (the 22 files the old step never read) | caught |
| no files present at all | fails, does not print ok |
| assertion-free DSL emission | caught |
| a decorative `echo` step injected into the workflow | flagged by the sweep |

**59f. Scope.** The sweep covers `formal-yosys.yml`. `formal-mutation.yml`'s own
two steps are not swept — the sweep runs inside that workflow, and having it
audit the job it is part of needs a fixed point this does not attempt. Six
harness defects in three waves came from *not* asking this question, so the
remaining unswept surface is two steps, stated rather than implied.

Reproduce:

```bash
python3 formal/absence_sweep.py
python3 formal/guard_scan.py
```

---

### Prop. 60 — the sweep now covers the workflow it runs inside — `PROVED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 59f stated the hole it left: `formal-mutation.yml`'s own two steps were
never swept, because the sweep runs as a step of that workflow and auditing the
job you are part of needs a fixed point Wave 608 did not attempt. It is
attempted here. **22 steps across both workflows, 0 passing on nothing.**

**60a. Exclude by content, not by name.** `collect()` drops any step whose
script invokes `absence_sweep.py`. Excluding by step *name* would mean renaming
the step silently reintroduces the recursion; excluding by what it *runs* does
not. The skipped step is reported and counted, never swallowed.

**60b. Self-exclusion is itself a way to check nothing.** A workflow whose only
step is the sweep collects zero steps — and a sweep that examines zero steps
and returns 0 is the exact failure of Props. 58–59, reintroduced by the
mechanism added to fix them. `--self-test` covers it with four synthetic
workflows whose answers are known:

| synthetic workflow | want |
|---|---|
| a step that passes on nothing (`echo`) | fail |
| a step that reads the subject (`test -f build/rtl/...`) | pass |
| a workflow with no `run` steps | fail |
| **a workflow whose only step is this sweep** | **fail** |

**60c. Both unswept steps failed correctly — and one lied about why.** *Scale
ceiling* printed `REFUTED -- a property fails at a larger bound` when nothing
had been refuted and yosys simply could not read the design. This is the last
instance of the Prop. 58 fold: `returncode != 0` treated as a verdict. The step
failed, which is the safe direction, but **a false diagnosis in CI sends someone
hunting a property failure that does not exist**. It now reports
`TOOL ERROR -- returned no verdict`. *Baseline, control and mutation* died three
frames deep inside `copytree`; it now names the missing modules.

**60d. Report what happened, not what is configured.** The sweep printed
`1 exempt` on runs where nothing was exempted, because it was printing
`len(EXEMPT)` — the size of the exemption list rather than the exemptions
actually applied. A small lie of precisely the kind this file exists to find,
and worth fixing for that reason rather than for its consequences.

**60e. Scope, honestly.** Every `run:` step of both formal workflows is now
swept except the sweep itself, which is covered by its self-test. The repository
has other workflows (docs, notebooks, seals); they are outside this campaign's
subject and are not swept. That is a boundary, not an oversight — stated here so
the next reader does not have to discover it.

Reproduce:

```bash
python3 formal/absence_sweep.py --self-test
python3 formal/absence_sweep.py
```

---

### Prop. 61 — how much of the design do 24 properties actually constrain — `MEASURED`

**Gate:** `formal-mutation.yml` → *Generated mutants land in code, not in comments*

"Neutralise a property and re-prove the rest" is the wrong experiment: these are
independent assertions about the same design, so removing one never makes
another fail. The question with content is **detection power** — for each way
the design can break, which properties notice? Method: mutate the DUT
mechanically, keep the mutants that parse, and for each (mutant, property) pair
run that property **alone** with every sibling neutralised.

**61a. The first run measured ASCII art.** 76 mutants of `interrupt_controller`,
zero detected — which reads as a damning verdict on the suite. Every one of the
76 had landed in a **comment**. Each module here opens with a banner made of `=`
characters, so an `==` operator generated 75 mutants inside `// =========` and
one inside an English sentence. All parsed, all proved, none touched a line of
code. The CI mutation harness catches an `interrupt_controller` mutation, which
is what made the zero implausible enough to check.

Two corrections. Mask comments before matching. And use operators that occur in
**this** RTL: the textbook list (`+`→`-`, `1'b1`→`1'b0`) matched nothing, because
the module is 23 non-comment lines of `?:`, `|`, `{}` and sized literals.
Operators are a property of the code under test, not of the mutation literature.

**61b. The measurement.** 1 485 isolation proofs across five suites:

| module | properties | code mutants | detected | undetected |
|---|---|---|---|---|
| `interrupt_controller` | 6 | 6 | 4 | 2 |
| `layer_sequencer` | 3 | 12 | 10 | 2 |
| `weight_prefetch_ctrl` | 3 | 41 | 11 | 30 |
| `axi_lite_slave` | 6 | 59 | 12 | 47 |
| `dma_controller` | 6 | 84 | 8 | 76 |
| **total** | **24** | **202** | **45 (22%)** | **157** |

**61c. Most of what they miss is real, and that took a second instrument.**
Mutation testing's standing confound is the equivalent mutant — an edit the
design is insensitive to. "157 undetected" would have been a number that sounds
like a verdict on the suite and is partly a verdict on the operators. A bounded
sequential equivalence miter separates them:

| of the 157 undetected | 90 s cap | 20 s cap |
|---|---|---|
| behaviourally **different** — the suite genuinely does not notice | **133** | **133** |
| equivalent to the miter depth — nothing to notice | 20 | 18 |
| tool error | 4 | 4 |
| undecided within the timeout | 0 | 2 |

Run twice at different timeouts, and the two agree on everything either of them
decided: **133 both times**. The only movement is two mitres that finished at
90 s and did not at 20 s, and the cheaper run reported those as *undecided*
rather than counting them equivalent. That is the Prop. 58 discipline paying
for itself in the instrument built to check it — an unfinished proof is not a
verdict, and had it been folded, the cheap run would have silently disagreed
with the expensive one.

So: **24 properties constrain about a fifth of the mechanically reachable
behaviour changes in these five modules.** That is a measurement, not an
indictment — safety properties are not a functional specification, and each of
these was written to pin one defect class. It is the first time the number
exists.

**61d. Two properties detect nothing, and only one of them is weak.**
`a_addr_ahead_of_data` (`weight_prefetch_ctrl`) and `a_wvalid_stable`
(`dma_controller`) detected zero mutants. For the second, a targeted mutation
explains why, and the explanation is not "the property is useless":

```bash
grep -n -A1 "a_wvalid_stable" formal/dma_controller_props.sv
```

The guard lives in the `always` header — `$past(wvalid) && !$past(wready)` — so a
mutation that suppresses `wvalid` does not violate the property, it makes the
guard **unreachable**, and the property proves *vacuously*. Probed directly with
a witness asserting the guard is impossible: **REFUTES on the original** (guard
reachable), **PROVES on the mutant** (guard gone).

**Mutation adequacy and vacuity interact, and a naive detection matrix cannot
tell them apart.** A mutant that kills a property's reachability is recorded
identically to a property too weak to see it. Any mutation score over guarded
properties is measuring both effects at once.

**61e. Five subsumption relations, and one that means less than it looks.**

| relation | mutants in that module |
|---|---|
| `a_bvalid_stable` ⊂ `a_one_outstanding_write` | 59 |
| `a_rvalid_stable` ⊂ `a_one_outstanding_read` | 59 |
| `a_no_read_accept_while_pending` ⊂ `a_one_outstanding_read` | 59 |
| `a_arvalid_stable` ⊂ `a_rready_implies_burst` | 84 |
| `a_read_burst_not_abandoned` ⊂ `a_rready_implies_burst` | 84 |
| four `interrupt_controller` properties with identical detection sets | **6** |

The last row is reported with its denominator because it is nearly meaningless:
identical behaviour over six mutants is what one expects from almost any pair.
**A subsumption claim is exactly as strong as the mutant set behind it**, and
these are stated so nobody deletes a property on six data points.

**61f. What ships.** The measurement is an analysis, not a gate — 1 642 proofs
is too much for CI. What ships is the guard against the mistake that nearly
published a false result: `formal/mutate.py --self-test` asserts every generated
mutant differs from the original on a non-comment line, and that a
fully-commented-out copy of a module yields **no** mutants at all. The eight
hand-written mutations in the harness are checked the same way.

**61g. Scope.** Five module suites, 24 of the repository's properties; the 26
integration properties on `bitnet_engine_top` are not covered — one mutant there
costs a full integration proof. Single-token mutations only. Equivalence is
bounded (seq 8–12, per the table above), so "equivalent" means *to that depth*.

Reproduce:

```bash
python3 formal/mutate.py build/rtl/interrupt_controller.sv
python3 formal/mutate.py --self-test
```

---

### Prop. 62 — one of the properties had never read the design — `FIXED`

**Gate:** `formal-yosys.yml` → *No property references a signal that does not exist*

Wave 610 ended with a list of 133 behaviourally-real gaps and the plan to write
properties against the biggest clusters. Four candidates were written for
`dma_controller`; all four were rejected on the first bar — *does it hold on the
real design?* Reading the counterexample rather than adjusting the property is
what turned this wave into something else.

**62a. The counterexample had two signals with the same name.** The trace showed
`\dut.word_index` **one bit wide** and `\dut.word_index_1` **twelve bits wide**
holding the real value. That is the signature of a fresh implicit wire with the
real register renamed around it — the property was reading a wire that did not
exist. Yosys had been saying so all along:

```bash
# not-runnable: the two warnings that were printed and never read
Warning: Identifier `\dut.word_index' is implicitly declared.
Warning: Wire wp_props.\dut.word_index is used but has no driver.
```

**62b. A shipped property was fake.** `a_addr_ahead_of_data` in
`weight_prefetch_props.sv` used the same form. It compared an undriven wire
against `bram_addr + 1`, which is why it proved — and Wave 610's detection matrix
had already recorded it detecting nothing. Decisive check: make the real
`word_index` advance by **two** instead of one, a change no correct form of this
property could survive.

| | verdict |
|---|---|
| `a_addr_ahead_of_data` on the real design | PROVED |
| the same, with `word_index` advancing by two | **PROVED** |
| `dut.busy == busy` (comparing a reference against its own port) | REFUTED |

It had proved for four waves without reading the design, and it was counted in
the property total, in the doc gate, and in the non-empty-property gate.

**62c. `identity_scan.py` cannot catch this, by construction.** That gate is a
syntactic scan for bodies the optimiser folds to constant true (Prop. 41). This
body is an ordinary comparison between two ordinary-looking operands. **The
signal is what is fake, not the shape** — a different failure needing a
different instrument. `formal/phantom_scan.py` elaborates each property module
and fails on those two warnings. It is cheap (no proof, only elaboration) and it
covers the class rather than the instance: hierarchical references, misspelled
signals, renamed ports. Its `--self-test` ships and checks all three.

**62d. Removed, not replaced, and the reason is not laziness.** The intent —
*the address channel never trails the data channel* — is not expressible from
this wrapper's ports. The controller streams one address per beat, and
`arready`/`rvalid` are free inputs here, so the solver may return data for an
address it never accepted; a port-level form was written and refutes for exactly
that reason. Stating it properly needs an AXI-slave assumption this suite does
not make, and adding one carries the over-constraint risk Prop. 50d recorded the
hard way — a strengthened environment that made a property prove and silently
killed two vacuity witnesses. **Left as work rather than shipped broken.** The
property count drops 42 → 41 and README says why.

**62e. The gate caught me while I was writing the replacement.** The first
port-level attempt used `axi_arvalid`, the DUT's port name, where the wrapper's
local wire is `arvalid`. `phantom_scan` reported it immediately — the same class
of defect, found in seconds instead of four waves.

Reproduce:

```bash
python3 formal/phantom_scan.py --self-test
python3 formal/phantom_scan.py
```

---

### Prop. 63 — an environment, and the three bars a property has to clear — `PROVED`

**Gate:** `formal-yosys.yml` → *Module suites are still alive under their assumptions*

Prop. 62 deleted `a_addr_ahead_of_data` and named the blocker: its intent needs
an environment, because `rvalid` is a free input and the solver may return read
data for an address the controller never issued. That is not a design behaviour
being explored — it is a testbench that cannot exist in silicon. This supplies
the environment and puts back a property that earns its place.

**63a. The environment.** One counter pair and one assumption: a slave returns
at most one beat per address it accepted.

```verilog
always @(posedge clk) if (rst_n)
    assume (!(rvalid && rready) || fv_r_acc < fv_ar_acc);
```

**63b. Three bars, not one.** Waves 41, 50d and 62 each recorded a property that
cleared "it proves" and nothing else. A property now has to clear:

| bar | question | how it is checked |
|---|---|---|
| **TRUE** | does it hold on the real design? | prove it, alone and with its suite |
| **ALIVE** | did the assumption buy that by making the design idle? | each activity must still be reachable **with the assumption active** |
| **BITING** | does it detect anything? | run it against the behaviourally-real gaps from Prop. 61 |

**63c. `a_writes_within_addresses` clears all three.** Every BRAM write is data
that was asked for: writes never outrun the addresses issued for them — what
the deleted property was reaching for, stated in ports.

| bar | result |
|---|---|
| TRUE | PROVED alone and with the suite |
| ALIVE | 5 probes — write, address accepted, beat accepted, two writes, prefetch completes — **all still refute** |
| BITING | detects **2** behaviourally-real mutants the whole suite missed, both spurious `bram_we` |

And the control that matters: with the property removed but the environment
kept, **0** of those two still refute. The detections belong to the property, not
to the assumption. Module property count returns to 42.

**63d. The assumption is gated, permanently.** An environment that is safe today
can over-constrain after any RTL change, silently. The *Module suites are still
alive* step now probes `arvalid && arready` and `rvalid && rready` **inside**
`wp_props`, with the assume active — 11 probes, all reachable. Prop. 50d's
failure is now something CI notices rather than something a future wave
rediscovers.

**63e. The same technique on `dma_controller`: environment yes, properties no.**
The environment transfers cleanly — `local_we`, `done` and both handshakes stay
reachable. Two candidate properties were written and **neither ships**:

| candidate | verdict | why it was rejected |
|---|---|---|
| `a_writes_within_request` | REFUTED | the port-only shadow of the request is wrong; not patched into passing |
| `a_beats_within_addresses` | PROVED, detects **0/64** | it restates the assumption — `fv_r_acc <= fv_ar_acc` given `fv_r_acc < fv_ar_acc` is assumed |

The second is the instructive one. **A property that restates its own
assumption proves, reads as meaningful, and constrains nothing.** It would have
passed every gate in this repository before this wave: non-vacuous guard,
non-free body, real signals, proves at depth. Only the BITING bar caught it —
which is the argument for keeping that bar even though it is the expensive one.

Reproduce:

```bash
python3 formal/phantom_scan.py
grep -c "a_[a-z0-9_]*: assert" formal/weight_prefetch_props.sv
```

---

### Prop. 64 — a verdict for every property, and none of them is dead — `MEASURED`

**Gate:** `formal-mutation.yml` → *Generated mutants land in code, not in comments*

Prop. 61 measured detection power. Prop. 63 showed the BITING bar catches a class
no cheap gate does. Neither had been applied to the properties already shipped.
This applies it to all 24 in the five module suites, and for every property that
detects nothing it decides **why** — because "detects nothing" has three quite
different causes and only one of them is a problem.

**64a. The verdicts.** 202 mutants, one property at a time with every sibling
neutralised, plus a guard-reachability probe for each zero-detection property.

| verdict | count | meaning |
|---|---|---|
| **BITES** | 18 | detects mutants, and is not contained in another property |
| **INNOCENT** | 1 | detects nothing because mutations that could violate it kill its *guard* |
| **SUBSUMED** | 5 | every mutant it catches is caught by another property |
| **DEAD** | **0** | guard reachable, not subsumed, still catches nothing |

**No property is dead weight.** That is the first evidence the suites are lean
rather than merely large, and it is the answer to a question left open since
Wave 609.

**64b. The innocent one, confirmed mechanically.** Prop. 61d diagnosed
`a_wvalid_stable` by hand-probing one mutation. The sweep now measures it: of 84
mutants, **4 make its guard unreachable**. Its guard sits in the `always` header,
so a mutation that suppresses `wvalid` does not violate the property — it
removes the state in which the property has anything to say, and it proves
vacuously. A detection matrix cannot tell that apart from weakness; a guard
probe can, and it is now run automatically for every zero-detection property.

**64c. Subsumed is not the same as deletable, and every one was kept.**

| property | subsumed by | why it stays |
|---|---|---|
| `a_bvalid_stable` | `a_one_outstanding_write` | states the AXI handshake rule in the specification's own form |
| `a_rvalid_stable` | `a_one_outstanding_read` | same |
| `a_no_read_accept_while_pending` | `a_one_outstanding_read` | the rule the fix implements, stated directly |
| `a_arvalid_stable` | `a_rready_implies_burst` | same |
| `a_read_burst_not_abandoned` | `a_rready_implies_burst` | it is the **regression witness** for the defect Prop. 9 fixed |

A property suite is read as well as run. Deleting the last row because a newer
property happens to cover it would discard the record of what went wrong — and
each verdict is now written next to the property, so the next reader of a
detection matrix does not mistake it for cleanup.

**64d. Symmetric properties need not have symmetric detection power.**
`a_awvalid_stable` **bites uniquely** — one mutant nothing else catches — while
its read-side twin `a_arvalid_stable` is subsumed and its write-data sibling
`a_wvalid_stable` is innocent. Three properties of identical shape over three
channels, three different verdicts. Reasoning about a suite by symmetry would
have got all three wrong.

**64e. Scope, and one thing this sweep does not report.** Five module suites, 24
properties; the 8 zero-size, 4 maximum-size and 26 integration properties are
not covered. The classifier flags strict containment only, so four
`interrupt_controller` properties with *identical* detection sets are reported as
`BITES (2, 0 uniquely)` rather than as duplicates — over six mutants that
equality means nothing (Prop. 61e), and the "0 uniquely" is the honest signal.

Reproduce:

```bash
grep -c "SUBSUMED" formal/axi_lite_slave_props.sv formal/dma_controller_props.sv
```

---

### Prop. 65 — the last twelve properties, an inverted sweep, and one dead — `MEASURED`

**Gate:** `formal-mutation.yml` → *Generated mutants land in code, not in comments*

Prop. 64 gave a verdict to the 24 module-suite properties. The 12 in the
zero-size and maximum-size suites were left. This finishes them, and doing so
required fixing the sweep itself.

**65a. Four properties are expected refutations, and the sweep did not know.**
The first run reported *ISOLATION BROKEN* on `a_zero_layers_never_completes`,
`a_zero_length_never_completes`, `a_zero_words_never_completes` and
`a_zero_neurons_never_completes` — because it assumed every property proves on
the real design. These four **refute by design**, and always have: they record
that a zero-sized job *does* report done, which is safe only because the sibling
`*_emits_no_work` proves it did not pretend to have done anything (Prop. 26).

The fix generalises the sweep: measure each property's **expected verdict**
first, then define detection as *the verdict differs from the expected one*. For
an inverted property that means a mutant made it **prove** — the mutation removed
the completion. A sweep that hard-codes "detection = refutation" cannot measure
an inverted property at all; it can only mislabel it.

**65b. The verdicts.**

| suite | property | verdict |
|---|---|---|
| `zs_multilayer` | `a_zero_layers_never_completes` *(inverted)* | BITES 3, 1 uniquely |
| | `a_zero_layers_emits_no_work` | BITES 6, 4 uniquely |
| `zs_dma` | `a_zero_length_never_completes` *(inverted)* | BITES 3, 3 uniquely |
| | `a_zero_length_moves_no_data` | BITES 7, 7 uniquely |
| `zs_prefetch` | `a_zero_words_never_completes` *(inverted)* | BITES 3, 1 uniquely |
| | `a_zero_words_writes_nothing` | BITES 6, 4 uniquely |
| `zs_layer` | `a_zero_neurons_never_completes` *(inverted)* | **DEAD** over 12 mutants |
| | `a_zero_neurons_emits_no_work` | BITES 2, both uniquely |
| `ms_prefetch` | `a_bram_addr_never_wraps` | SUBSUMED by `a_bram_writes_contiguous` |
| | `a_bram_writes_contiguous` | BITES 6, 1 uniquely |
| `ms_dma` | `a_local_addr_never_wraps` | SUBSUMED by `a_local_writes_contiguous` |
| | `a_local_writes_contiguous` | BITES 4, 1 uniquely |

**65c. The campaign's first DEAD verdict, reported with its denominator.**
`a_zero_neurons_never_completes` detects nothing across **12** mutants — and 12
is a weak denominator, which Prop. 61e is explicit about. `layer_sequencer` is
23 non-comment lines; no single-token edit diverts the path from the zero guard
to `DONE_ST`. **It is kept**, and not out of timidity: it is an expected
refutation whose job is documentary — it pins a completion policy Prop. 26
decided deliberately, and its sibling is what makes that policy safe. *A property
whose value is the record it leaves does not have to earn its place by
detection.* The verdict is written beside it so nobody re-derives this.

**65d. Both max-size subsumptions were predictable, and that is the point.**
`addr_never_wraps` (strictly increasing) is implied by `writes_contiguous`
(increases by exactly one). The measurement confirming an implication anyone
could see on paper is not wasted — it is the calibration that makes the
*unexpected* verdicts credible, like `a_awvalid_stable` biting uniquely while its
read-side twin does not (Prop. 64d).

**65e. Where the campaign now stands.** 36 of the 42 module properties carry a
measured verdict:

| | BITES | INNOCENT | SUBSUMED | DEAD |
|---|---|---|---|---|
| Prop. 64 (24 module-suite) | 18 | 1 | 5 | 0 |
| Prop. 65 (12 size-sweep) | 9 | 0 | 2 | 1 |
| **total (36)** | **27** | **1** | **7** | **1** |

The 26 integration properties are not covered: one mutant there costs a full
integration proof, and the sweep above already cost more than the CI budget for
a week. Stated as scope, not implied.

**65f. A process failure worth recording.** The corrected sweep was launched
while the first was still running, both writing the same file. The merged output
was self-inconsistent — a summary line that disagreed with the rows above it —
and was discarded rather than read. Two runs sharing an output path produce
something that *looks* like data.

Reproduce:

```bash
grep -c "SUBSUMED\|DEAD over" formal/max_size_props.sv formal/zero_size_props.sv
```

---

### Prop. 66 — the engine's 26, sampled, and a limit that does not lift — `MEASURED`

**Gate:** `formal-mutation.yml` → *Generated mutants land in code, not in comments*

Props. 64 and 65 gave verdicts to 36 module properties. The engine's 26 were the
stated gap: one integration proof costs ~125 s, and `bitnet_engine_top` has 212
mechanical mutants. So this samples — seven, one per subsystem named by the
defects this campaign actually found — and reports the sample size everywhere.

**66a. The generator was mutating the properties.** The engine carries its 26
integration properties **inline**, behind `T27_FORMAL` guards, and **68 % of that
file is comment or formal-only text**. Two of the first eight sampled mutants
changed `a_mem_port_is_prefetch` and `a_status_reflects_engine` — assertion text,
not logic. *A property suite that "detects" a mutation of itself measures
nothing.* This is Wave 610's comment bug in a second costume: the operator has to
know what it is allowed to touch.

`code_mask` now masks comments, `` `ifdef T27_FORMAL* `` regions (nesting-aware),
and any labelled `assert`/`assume` line. Across the 13 emitted modules the mutant
count drops **627 → 481**; the self-test gained a case for it, and the eight
hand-written mutations in the harness were checked by hand against the same mask
— all eight land in design code.

**66b. One of seven.** Baseline control first: the unmutated engine PROVES in
125 s, so the verdicts below are not noise.

| subsystem | mutation | verdict |
|---|---|---|
| input readiness | `&& (filled >= neurons_per_layer)` → `\|\|` | **REFUTED** (12 s) |
| double-buffer ping-pong | `layer_done_pulse && !use_buffer_a` → `\|\|` | proved |
| config latch | `neurons_q <= 16'd0` → `16'd1` | proved |
| dma / overflow | `input_loaded <= 1'b1` → `1'b0` | proved |
| activation / requant | `act_wr_word + 12'd1` → `- 12'd1` | proved |
| layer sequencing | `chunk_addr + 12'd1` → `- 12'd1` | proved |
| interrupt / status | `{30'd0, done, busy}` → `{30'd1, …}` | proved |

**66c. And the limit that does not lift: this cannot be turned into "six gaps".**
Prop. 61c's rule is that *undetected* is not *missed* until equivalent mutants
are ruled out. At module scale a bounded miter did that. At engine scale it does
not, and the validation step is what proved it rather than a hunch:

| miter depth | on a mutant the properties **do** detect | cost |
|---|---|---|
| `seq 6` | **EQUIVALENT** | 6 s |
| `seq 12` | **UNDECIDED** | 420 s (cap) |

A miter that calls a known-different mutant equivalent is too shallow to mean
anything, and one step deeper does not finish. The properties see this mutant at
`seq 40`; the miter cannot reach `seq 12`. **So the six undetected mutants are
recorded as undetected, not as gaps** — the equivalence question is open for the
engine and this method will not close it.

**66d. What the number is worth.** "1 of 7" is a measurement of *these seven
mutations* against the 26 properties, with the equivalent-mutant fraction
unknown. It is not a coverage percentage and must not be quoted as one. What it
does establish is a floor: at least one subsystem mutation is caught, the
baseline control passes, and six mutations of subsystems this campaign has
previously found defects in produce no reaction from the integration suite.

**66e. Scope.** Seven of 212 mutants, one per subsystem, chosen by defect
history rather than at random — so the sample is *representative of where
defects came from*, not of the mutant population. Single-token mutations only.
No equivalence classification, per 66c.

Reproduce:

```bash
python3 formal/mutate.py --self-test
grep -c "T27_FORMAL" build/rtl/bitnet_engine_top.sv
```

---

### Prop. 67 — half the gate set, a phase-blind suite, and a bound that lies — `FIXED`

**Gate:** `formal-yosys.yml` → *Engine is still alive under its interlocks*

Prop. 66 reported **1 of 7** sampled engine mutations detected. That number was
measured against the 26 **safety** properties. The engine's gate set is safety
**∪ liveness**, and nobody had run the other half.

**67a. Measured against half the gates.** Re-running the six "undetected"
mutants through *Engine is still alive under its interlocks*: the **dma /
overflow** mutation is caught outright — `weight prefetch can write` and `MAC can
be active` both stop refuting, meaning those activities become impossible. The
honest figure was **2 of 7**, not 1. Prop. 66's number is corrected here rather
than quietly amended in place.

**67b. Every liveness probe was phase-blind.** The double-buffer mutation clears
`filled_b` throughout the phase where B is the read buffer, so `filled` reads 0,
`input_ready` never asserts, and the engine **stalls in that phase**. A stalled
phase violates no safety property, and the five existing probes ask only whether
an activity can happen **at all** — which it still can, in the other phase. One
phase-conditioned probe closes it:

```verilog
assert (!(mac_valid_q && !use_buffer_a));   // must REFUTE
```

| | real engine | double-buffer mutant |
|---|---|---|
| `!(mac_valid_q && !use_buffer_a)` @ seq 40 | refutes | **proves** |

That takes the sample to **3 of 7**. Four remain undetected by anything: config
latch, activation/requant, layer sequencing, interrupt/status.

**67c. And the bound was lying.** The existing step runs every probe at
`seq 22`. At 22 this same probe **proves** on the real engine — reporting the
activity *unreachable* when it is merely further away than the bound. **A probe
run at too shallow a depth does not return "unknown", it returns the wrong
answer**, and the wrong answer here is the one that reads as a passing build.
Probes now carry a per-probe depth; this one runs at 40.

Three earlier candidates were rejected by measurement rather than taste, and one
of them shows why the degenerate case matters: `!(input_ready && !use_buffer_a)`
refutes on the mutant too, because `filled >= neurons_per_layer` is satisfiable
with `neurons_per_layer == 0` and the solver simply picks that configuration.

**67d. The `proves`-direction probe was re-checked, and holds.** A bound that is
too shallow threatens only the *proves* direction — a refutation at depth 22 is a
real counterexample at any depth. The single probe expecting `proves`,
`!(dma_busy && mac_valid_q)`, was re-run at 22 / 40 / 60: **proves at all three**,
in 5 s / 11 s / 20 s. Cheap, and now known rather than assumed.

**67e. What this says about the sample.** Two of the seven were caught only
because the measurement was redone — once for including the other half of the
gate set, once for adding a probe. "1 of 7" was not wrong arithmetic; it was a
complete count of an incomplete question. The scope line on a measurement has to
name *which gates were run*, not only which mutants.

Reproduce:

```bash
grep -c "^          probe " .github/workflows/formal-yosys.yml
```

---

### Prop. 68 — auditing the bounds, and a generalisation that did not hold — `MEASURED`

**Gate:** `formal-yosys.yml` → *Engine is still alive under its interlocks*

Prop. 67c found a probe whose `seq 22` verdict was **wrong**, not unknown. Every
`PROVED` in this repository carries the same hidden qualifier, and only that
direction can fail this way — a refutation found at depth N is a real
counterexample at any depth. This audits them, and separately tests whether
Prop. 67b's phase-conditioning generalises.

**68a. Four wrappers re-proved at 2× and 4× their CI bound. No verdict flips.**

| wrapper | CI bound | 2× | 4× |
|---|---|---|---|
| `irq_props` | 6 → PROVED | 12 → PROVED | 24 → PROVED |
| `axi_props` | 10 → PROVED | 20 → PROVED | 40 → PROVED |
| `dma_props` | 80 → PROVED | 160 → PROVED | 320 → PROVED |
| `ls_props` | 48 → PROVED | 96 → PROVED | 192 → *undecided* |

`ls_props` at 4× exceeds the solver's reach inside the time budget. **Undecided
is not a flip** — it is the honest outcome, and it is reported rather than
retried until it produced a number. `dma_controller` surviving to `seq 320` is
the strongest single result: its bound was raised 12 → 80 in Prop. 35, and this
says that raise was not merely convenient.

**68b. Scope, and it is partial on purpose.** Four of twelve wrappers audited;
`wp_props`, `ar_props`, the four zero-size and two maximum-size wrappers are not
yet done — each 4× run costs more than the wave's remaining budget. Naming which
were audited is the point of Prop. 67e; a partial audit reported as partial is
worth more than a complete one reported without its cost.

**68c. Phase-conditioning does not generalise, and that is a real answer.**
Prop. 67b fixed a phase-blind probe and I expected the blindness to be general —
the previous wave said so in writing. Five phase-conditioned candidates were
built and measured against the four mutants nothing currently catches:

| candidate probe | real engine | four undetected mutants |
|---|---|---|
| `!(neuron_out_valid && !use_buffer_a)` | refutes | all four refute |
| `!(neuron_out_valid && use_buffer_a)` | refutes | all four refute |
| `!(dma_local_we && !use_buffer_a)` | refutes | all four refute |
| `!(pf_bram_we && !use_buffer_a)` | refutes | all four refute |
| `!(mac_valid_q && use_buffer_a)` | refutes | all four refute |

**Not one bites.** The double-buffer fault was catchable by phase-conditioning
because it *stalls one phase*; the remaining four (config latch, activation/
requant, layer sequencing, interrupt/status) do not stall anything, so no
reachability probe of any phase will see them. A prediction from the previous
wave, refuted by measurement rather than carried forward — which is cheaper than
shipping five probes that pass and prove nothing.

**68d. What the remaining four would actually need.** Not liveness. Each changes
a *value* while leaving every activity reachable: a config latch reset to 1, an
accumulator decrementing instead of incrementing, a status word with a stray
bit. Those are safety claims about data, and the 26 existing safety properties
are about control. That is the shape of the gap, stated so the next attempt does
not begin with another probe.

Reproduce:

```bash
grep -c "^          probe " .github/workflows/formal-yosys.yml
```

---

### Prop. 69 — eight properties counted as proved, run by no job — `FIXED`

**Gate:** `formal-yosys.yml` → *Prove zero-size properties*

Prop. 68 audited four wrapper bounds and reported the rest as "not yet audited —
each 4× run costs more than the wave had left". That was true of two of them. It
was **wrong** about the other six, and the audit tool had already said so in a
way I read as its own bug: *no bound found in the workflow*.

**69a. There was no bound because there was no step.** `zero_size_props.sv`
appears **once** in the whole of `.github/` — inside the *weekly* mutation
harness, as gate definitions for two of its four wrappers. `zs_prefetch` and
`zs_layer` appear **nowhere at all**.

| suite | proved by a CI step? |
|---|---|
| `max_size_props` (4 properties) | yes — *Oversized requests do not wrap*, `seq 30` |
| `zs_dma`, `zs_multilayer` (4) | only as mutation-gate definitions, weekly |
| `zs_prefetch`, `zs_layer` (4) | **no job in this repository ran them** |

README counted all eight among "42 properties proved". Four of them had never
been proved by CI at all, and the other four only as a side effect of mutation
testing.

**69b. Why it was probably never gated.** Four of the eight are **expected
refutations** — a zero-sized job *does* report done, which is safe only because
its sibling proves it emitted no work (Prop. 26). A prove step that expects
everything to prove cannot gate this suite; it needs a per-property expected
verdict. The same shape that made Prop. 65's sweep report *ISOLATION BROKEN*
made this suite awkward to gate, and awkward-to-gate is how something ends up
ungated.

**69c. Now gated, and all eight behave as documented.**

| wrapper | property | expected | got |
|---|---|---|---|
| `zs_multilayer` | `a_zero_layers_never_completes` | refutes | refutes |
| | `a_zero_layers_emits_no_work` | proves | proves |
| `zs_dma` | `a_zero_length_never_completes` | refutes | refutes |
| | `a_zero_length_moves_no_data` | proves | proves |
| `zs_prefetch` | `a_zero_words_never_completes` | refutes | refutes |
| | `a_zero_words_writes_nothing` | proves | proves |
| `zs_layer` | `a_zero_neurons_never_completes` | refutes | refutes |
| | `a_zero_neurons_emits_no_work` | proves | proves |

**Nothing was broken** — which is the good outcome and also the reason this could
sit unnoticed for as long as it did. An ungated property that happens to hold
looks exactly like a gated one until someone counts the steps.

**69d. Correction to Prop. 68b.** Of the six wrappers reported there as
unaudited-for-cost: four had no step to audit, and `ms_prefetch` in fact
completed — `seq 30 → 60 → 120`, PROVED throughout. The genuine cost-limited
cases are `wp_props` and `ms_dma`. **`wp_props` also revealed a method mismatch**:
CI proves its three properties **one at a time**, and the audit ran them
together, which does not complete at the same bound. An audit has to reproduce
the gate's method, not merely its bound.

**69e. The lesson is about the instrument again.** The audit's "no bound found"
was the finding, not the failure. When a measuring tool reports that it cannot
find something, the first hypothesis should be that the thing is absent.

Reproduce:

```bash
grep -c "zero_size_props" .github/workflows/formal-yosys.yml
```

---

### Prop. 70 — count the steps, not the properties — `FIXED`

**Gate:** `formal-yosys.yml` → *Every property file is run by some workflow*

Prop. 69 found eight properties counted as proved that no job ran, and found
them by accident — one line of a bound audit I nearly dismissed as my own bug.
This is the systematic version, and it found another on its first run.

**70a. The gate.** `formal/orphan_scan.py` cross-references every `formal/*.sv`
against every workflow in `.github/`, with two levels of finding:

| level | meaning |
|---|---|
| **ORPHAN** | no workflow references the file — an error |
| **WEEKLY** | referenced only by schedule-triggered workflows, so a defect is invisible on a pull request — reported, not failed |

Weekly-only is a legitimate choice for expensive harnesses; **silence is what is
not allowed**. The scan ships with a self-test covering an injected orphan, the
clean tree, and an empty tree (which must fail rather than pass on nothing).

**70b. It found `axi4_read_slave_model.sv` immediately** — 88 lines, fully
documented, referenced by nothing. It constrains `arready`/`rvalid`/`rlast` to
what AXI4 requires of a compliant read slave, and **asserts** its single-burst
precondition rather than assuming it, precisely so it cannot silently
over-constrain.

Wave 612 had hit exactly this need on the DMA, failed to state a property
without an environment, and **rebuilt a weaker version inline for a different
module** — not knowing this file existed. The cost of orphaned work, made
concrete: not a stale file, a solved problem solved twice, worse the second time.

**70c. Wired in, and validated on three bars first.**

| bar | result |
|---|---|
| TRUE | `dma_props` proves at `seq 80` with the model's assumptions active |
| ALIVE | `local_we`, `done`, both handshakes and `rlast` all still reachable |
| FAITHFUL | `a_model_precondition_single_burst` **PROVES** — the DMA really does issue one burst at a time, so the model is not lying about its subject |

The five liveness probes are now gated in *Module suites are still alive under
their assumptions*, so the model cannot come to over-constrain silently later.
Check-cell floor raised 7 → 8 for the assertion the model contributes.

**70d. Three call sites broke, and all three broke correctly.** The liveness
step, the weekly mutation harness, and `phantom_scan.py` each read only the DUT
and its props file, so a wrapper that instantiates something else fails to
elaborate. Every one reported **an elaboration error** — not "unreachable", not
"mutant killed", not a clean bill of health. Prop. 39d's distinction, Wave 608's
`ToolError` path, and Prop. 62's did-not-elaborate branch each earning their keep
on a change none of them anticipated. All three now take an explicit
extra-sources field.

That is the real return on those three waves: a change to one property file
propagated into three unrelated harnesses, and not one of them turned the
breakage into a passing result.

**70e. What the gate is really for.** Counting properties tells you nothing
about whether they run. This repository has now twice shipped properties that
held, were counted, and were never executed — and in both cases *nothing was
broken*, which is exactly why nobody noticed. The check costs one `grep` per
file and would have caught both at any point in the preceding twenty waves.

Reproduce:

```bash
python3 formal/orphan_scan.py --self-test
python3 formal/orphan_scan.py
```

---

### Prop. 71 — the DMA data property, six waves late — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove dma_controller properties*

Prop. 29 fixed a defect where an oversized request wrapped the local address,
overwrote data already transferred, and reported done. It has had **no property**
since. Wave 610's gap list named it, Wave 612 could not state it, and Prop. 70's
environment is what finally made it statable.

**71a. `a_writes_within_request`.** The transfer never writes more words than the
request covers: a shadow latches the clamped length when the transfer starts and
counts local writes, and the assertion is
`fv_writes <= (fv_owed + 7) >> 3`.

| bar | result |
|---|---|
| TRUE | PROVED, alone and with the suite |
| BITING | detects **13 of the 64** behaviourally-real mutations the whole suite missed — the largest bite of any property in this campaign |

**71b. Two false starts, both settled by reading a counterexample.** Wave 612's
shadow armed on `start && !busy`; the FSM triggers on `IDLE: if (start)`, and
`start` is also high in states where no transfer begins. The observable that
tracks the FSM exactly is the **rising edge of `busy`** — and `length` is latched
the cycle before, so the shadow must read `$past(length)`.

Then the corrected shadow still refuted, and the trace showed why: with
`length = 12` the DMA performs a **second** write while only 4 bytes are owed.
That is correct — twelve bytes occupy two words of a word-addressed memory, the
second partially. **The property was wrong about the design's contract, not the
design wrong about the property.** Restated in words rather than bytes, it
proves. Two wrong properties, two counterexamples, no guessing.

**71c. It broke the step that proves it, and Prop. 35 already knew why.** Adding
it took `dma_props` at `seq 80` from ~10 s to **over 11 minutes without
terminating**. `-prove-asserts` solves every assertion in one SAT instance,
superlinearly harder than its parts. Split one-per-invocation, as
`weight_prefetch` already was:

| | bound | time |
|---|---|---|
| six existing properties | `seq 80` | 4–6 s each |
| `a_writes_within_request` | `seq 20` | 16 s (**undecided at 30**) |

The whole step now runs in ~48 s and **six properties keep bound 80** that the
batch would otherwise have cost the suite entirely. The new property's bound is
20, and that asymmetry is stated rather than averaged into a single number — it
carries a 13-bit write counter and a 32-bit byte count, which is both why it is
expensive and why it bites.

**71d. A second candidate was measured and dropped.**
`a_owed_never_underflows` proved, and detected **2** mutants — both already in
the 13. Subsumed, and unlike the subsumptions kept in Prop. 64c it has no
documentary value either: its subject is my own shadow register, not the design.
Shipping it would have been shipping a property about the harness.

**71e. The check-cell floor was three under the truth.** Raised 8 → **12**, the
measured count. A floor set comfortably below the real number lets that many
properties disappear before the gate notices — which is the failure this gate
exists to prevent.

Reproduce:

```bash
grep -c "a_[a-z0-9_]*: assert" formal/dma_controller_props.sv
```

---

### Prop. 72 — the gap list was measured one suite at a time — `MEASURED`

**Gate:** `formal-mutation.yml` → *Generated mutants land in code, not in comments*

Prop. 61 reported 64 behaviourally-real gaps for `dma_controller`. Prop. 71 shut
13 of them. Re-measuring the rest exposed something about the *first* number.

**72a. The 13 are real, and measured rather than inferred.** The design is
unchanged, so the six older properties detect exactly what they detected before;
only `a_writes_within_request` needed re-running. It catches **13 of the 64** —
matching Wave 619's figure on an independent run — leaving 51.

**72b. Prop. 61 measured each suite in isolation, and three suites constrain
this module.** `dma_props` was the only one consulted. But `ms_dma`
(maximum-size) and `zs_dma` (zero-size) are wrappers around the **same DUT**, and
a mutation "undetected" by one is not therefore undetected.

| | count |
|---|---|
| Prop. 61's reported gap | 64 |
| closed by `a_writes_within_request` (Prop. 71) | −13 |
| **caught all along by `ms_dma` (3) and `zs_dma` (5)** | **−8** |
| true remaining gap | **43** |

The eight were never gaps. Prop. 61's number was an overcount by construction —
a per-suite measurement reported as a per-module one. Every gap figure in Props.
61 and 66 carries that same caveat, and it is corrected here rather than left to
be rediscovered.

**72c. The residue is flat, so the method that produced Prop. 71 is spent here.**
The 51 spread across **42 distinct lines**, 33 of them singletons. The largest
remaining cluster is 4. Compare Wave 611, where the top clusters (8 mutants on
two lines of transfer accounting, 9 on burst arithmetic) are exactly what became
a property. Nothing of that shape is left: the residue is reset values, state
encodings, and one-off arithmetic, each worth roughly one mutant.

**Cluster-and-write extracted what it could from this module.** Continuing would
mean one property per mutation, which is not a property suite but a restatement
of the RTL.

**72d. What this says about gap numbers generally.** A gap count is a claim about
*a set of properties*, and it must name which set. "64 gaps in `dma_controller`"
sounds like a fact about the module; it was a fact about one wrapper. The right
form is "43 mutations of `dma_controller` are detected by none of its three
suites" — longer, and the only version that survives contact with a second
wrapper.

Reproduce:

```bash
python3 formal/mutate.py build/rtl/dma_controller.sv
```

---

### Prop. 73 — the campaign's most-quoted number, corrected — `MEASURED`

**Gate:** `formal-mutation.yml` → *Generated mutants land in code, not in comments*

Prop. 72 corrected `dma_controller`'s gap figure and named the cause: a per-suite
measurement reported as a per-module one. Two other modules have more than one
suite. Correcting them corrects the headline.

**73a. The remaining multi-suite modules.**

| module | suites | Prop. 61 gap | caught | true gap |
|---|---|---|---|---|
| `weight_prefetch_ctrl` | `wp_props` (2), `zs_prefetch` (5), `ms_prefetch` (5) | 24 | **8** | **16** |
| `layer_sequencer` | `ls_props` (0), `zs_layer` (0) | 2 | 0 | 2 |

`interrupt_controller`, `axi_lite_slave` and `multilayer_sequencer` have one
suite each, so their figures needed no correction — stated so an absent row is
not read as an omission. `layer_sequencer` needed none either: having a second
suite does not guarantee an overcount, it only makes one possible.

**73b. The headline, recomputed from the recorded data.**

| | mutants | detected | real gaps |
|---|---|---|---|
| Prop. 61 as published | 202 | 45 (**22 %**) | 133 |
| corrected | 202 | **74 (36 %)** | **104** |

Of the 29 newly-counted detections, **15 come from properties added after Prop.
61 was measured** (13 from Prop. 71, 2 from Prop. 63) and the rest from suites
that existed the whole time and were never consulted.

**73c. The error ran against the suite, not for it.** A measurement mistake that
flatters its subject is the one to expect; this one did the opposite, reporting
the property set as catching 22 % of mutations when it catches 36 %. Worth
recording because it is evidence about the *process* rather than the result: the
method was wrong in a direction nobody had an incentive to notice, and it stood
for twelve waves.

**73d. What made it wrong is a sentence, not a bug.** No harness misbehaved.
The matrix did exactly what it was told — measure this property set against these
mutants — and the caption said "gaps in `dma_controller`" where the data said
"gaps with respect to `dma_props`". Every instrument in this campaign has been
audited for lying; this was the *label* lying while the instrument told the
truth.

**73e. Scope.** The equivalent-mutant classification is unchanged: whether a
mutation alters behaviour does not depend on which properties are watching. Only
the detected/undetected split moves. The 26 integration properties are still
sampled rather than swept (Prop. 66), and that figure keeps its own caveat.

Reproduce:

```bash
python3 formal/mutate.py --self-test
```

---

### Prop. 74 — twenty waves auditing the tools; this one audits the prose — `FIXED`

**Gate:** `formal-yosys.yml` → *Numbers in the documentation match the tree*

Prop. 73 corrected a figure quoted for twelve waves, and nothing had
malfunctioned: the instrument measured exactly what it was told to, and the
caption named a *module* where the data described a *wrapper*. **That class of
error is invisible to every gate built so far**, because all of them check
whether the tools lie. This one checks whether the prose does.

**74a. `formal/claims_check.py`.** Re-derives each countable claim from the tree
and compares it to README.md. It found two numbers already adrift:

| claim | README said | tree has |
|---|---|---|
| propositions covered by the doc gate | 58 | **73** |
| integration properties | 26 | **28** |

And the CI step *names* had drifted with them — *"Prove integration properties
(core 22)"* and *"(all 26)"* against a tree of 24 and 28. The steps prove
whatever the file contains, so the numbers were pure label.

**74b. It only polices the current-state document, and that is a decision.**
Propositions here are dated records: *"22 of the 26 prove at seq 80"* was true
when measured, and rewriting it would destroy the record rather than fix a
number. Corrections belong in a later proposition, as Prop. 67a did for Prop. 66.
README is the one document asserting the present, so it is the one held to the
tree.

**74c. The checker had the disease it was built to find, twice.** First it
counted the engine's assertions in total (28) and compared that against a
documented "26", flagging drift that might have been its own miscount. Then a
per-line count said 26 and appeared to vindicate the docs — but **two assertions
wrap the label and `assert` onto separate lines**, so the per-line figure
undercounts by exactly two. The truth needed a guard-aware count over the text,
not the lines: **24 core + 4 tracker-backed = 28**. Two properties had been added
to the core set without any label following them.

I nearly published "the docs are stale by 2" from the first count and "the docs
are correct" from the second. Neither was established. *A checker that compares
two numbers must first establish that both range over the same set* — which is
the same failure as Prop. 73, committed inside the tool built to prevent it.

**74d. It caught its own author within the wave.** Writing this proposition took
the count from 73 to 74 while README still said 73, and the gate failed on the
next run. That is the smallest possible demonstration that the check is
load-bearing rather than decorative: the number it polices drifts *whenever
anyone documents anything*, which is precisely why it had drifted 15 behind.

**74e. What it cannot check, stated.** "13 of 64 gaps" is a measurement, not a
property of the tree; re-deriving it means re-running an hour of proofs, so it is
out of scope. What is in scope is every number that *is* a property of the tree —
where drift is both most likely and least visible, because nobody re-counts
propositions by hand.

Reproduce:

```bash
python3 formal/claims_check.py --self-test
python3 formal/claims_check.py
```

---

### Prop. 75 — properties live in two places, and one module has no file at all — `MEASURED`

**Gate:** `formal-yosys.yml` → *Numbers in the documentation match the tree*

Prop. 74 left one number unexplained rather than publishing it: the checker
derived **39** module properties where README claims **43**. Prop. 74c's rule is
that a mismatch is not a finding until both sides are established. Establishing
them found something about the repository's structure, not about the count.

**75a. The two sides.**

| in `formal/*.sv` | | in `build/rtl/*.sv` | |
|---|---|---|---|
| `interrupt_controller_props` | 6 | **`activation_requant`** | **6 inline** |
| `axi_lite_slave_props` | 6 | `bitnet_engine_top` | 28 (counted separately) |
| `dma_controller_props` | 7 | | |
| `layer_sequencer_props` | 3 | | |
| `weight_prefetch_props` | 3 | | |
| `zero_size_props` | 8 | | |
| `max_size_props` | 4 | | |

25 + 8 + 4 + 6 = **43.** README was right the whole time. The checker was
counting `formal/` only, and **`activation_requant` has no file in `formal/` at
all** — its six properties are emitted inline into the RTL, like the engine's.
A `formal/`-only count silently omits an entire module.

**75b. Two things in `formal/` are not module properties, and the boundary is
now written down.** `assume_liveness_check.sv` checks the *prover* — that
`-set-assumes` is in effect — and `axi4_read_slave_model.sv` asserts a
precondition on the *environment*, not a property of any module. Excluding them
is a judgement, so it is recorded next to the code rather than left implicit:
39 = 37 + those two, 43 = 37 + activation_requant's six.

**75c. The orphan scan has the same blind spot, and it is stated not fixed.**
`orphan_scan.py` (Prop. 70) asks whether every file in `formal/` is run by some
workflow. It cannot ask that of `activation_requant`'s properties, because there
is no file — they are inside the emitted RTL, gated by whatever step proves that
module. That is a real limit of the scan, discovered here, and widening it to
emitted RTL is a separate piece of work rather than a line added in passing.

**75d. Five claims now gated, up from three.** `module properties` and
`engine liveness probes` join the set. README did not state the probe count at
all, so it now does — a number that exists only in a workflow file and nowhere a
reader would look is one more place for drift to hide.

**75e. What this cost, and why it was worth not skipping.** One wave to resolve
a four-count discrepancy that turned out to be *no discrepancy at all*. The
alternative was to "fix" README from 43 to 39 and gate the wrong number — which
would have been a correct-looking gate enforcing a false claim, the exact shape
of Props. 73 and 74.

Reproduce:

```bash
grep -c "a_[a-z0-9_]*: assert" build/rtl/activation_requant.sv
```

---

### Prop. 76 — twenty-three modules, and six that nothing reaches — `MEASURED`

**Gate:** `formal-yosys.yml` → *Every property file is run by some workflow*

Prop. 75c named the limit: a scan over `formal/` cannot see properties that have
no file. Closing it answers a question deferred four times — *does every emitted
module have properties at all?* — and the answer needed the scan rewritten twice
before it meant anything.

**76a. Per module, not per file.** The first version keyed on the filename, and
`trit_stdlib.sv` defines **eleven** ternary primitives. A file-stem classifier
reports one module that does not exist and misses eleven that do. It also has to
follow instantiation **transitively**: `trit27_dot_product` is reached from the
engine only through `pipeline_stage2_compute`.

**76b. The map.** 23 modules in the emitted bundle:

| coverage | count | meaning |
|---|---|---|
| **DIRECT** | 8 | a `formal/` suite instantiates it, or it carries inline assertions |
| **INDIRECT** | 8 | no properties of its own; reachable from the engine, so its integration properties constrain it at one remove |
| **UNREACHED** | 6 | no properties **and instantiated by nothing** — `trit_not`, `trit_and`, `trit_or`, `trit_multiply`, `trit_compare`, `trit3_add` |
| **EXEMPT** | 1 | `behavior_sva_v2` — concurrent SVA this flow cannot check at all (Props. 2/5/6) |

The six unreached primitives are read into **every** engine proof as source and
constrained by none of them. They are a library, so this is not a defect — but
"a library nobody instantiates, carried in the bundle" is a fact that should be
visible rather than implied.

**76c. The module the campaign's longest defect lived in has no properties of its
own.** `double_buffer_ctrl` is INDIRECT. The ping-pong took three changes across
eight waves to get right (Props. 33, 46b, 47), every one of them diagnosed and
fixed at the *engine* level, and the 33-line module implementing it has never had
a property. That is not an accusation — engine-level was where the defect was
observable — but it is the single most interesting line in the table.

**76d. Reported, not failed.** An unexercised library module is not a build
error, and a permanently red gate is one everyone learns to ignore — the
workflow's own comment on the scale ceiling says exactly that. Errors stay for
the unambiguous case (a property file no workflow runs); coverage is warnings
plus a count. **Silence is what is not allowed.**

**76e. The exemption is argued, per Prop. 59d.** `behavior_sva_v2` emits
concurrent SVA — `##N`, `s_eventually` — which Yosys cannot check *at all*. It is
the artifact whose uncheckability is the documented reason
`gen-behavior-sva-yosys` exists, so proving it is not merely undone but
impossible in this flow.

Reproduce:

```bash
grep -c "^module " build/rtl/trit_stdlib.sv
```

---

### Prop. 77 — the ping-pong finally has properties of its own — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove double_buffer_ctrl properties*

Prop. 76's most interesting row: `double_buffer_ctrl` is 33 lines, implements the
ping-pong, produced the campaign's longest-running defect — three changes across
eight waves (Props. 33, 46b, 47) — and had **never had a property of its own**.
Every one of those fixes was made at the engine level, where the symptom was
observable, and nobody went back to constrain what produced it.

**77a. Four properties, all proving.**

| property | what it pins |
|---|---|
| `a_toggles_on_layer_done` | the buffers alternate, and on the layer boundary |
| `a_stable_without_layer_done` | *nothing else* moves the phase — the half a fix for the first can break |
| `a_reset_reads_a` | layer 0 reads A; the engine's selects assume this polarity |
| `a_addresses_agree` | read and write index the same slot, in different buffers |

**77b. It catches the harness's own mutation at module level.** The weekly
harness carries *"double buffer stops alternating"* as a hand-written mutant, and
until now only the **engine** gate caught it. `db_props` refutes it directly —
which is the difference between "some integration property noticed something" and
"the ping-pong is wrong".

**77c. `-set-init-zero` makes a reset property refute on the real design.** The
guard `rst_n && !$past(rst_n)` reads as "the cycle after reset released". Under
`-set-init-zero` every register starts at 0, so at time zero `$past(rst_n)` is 0
whether or not a reset ever happened, and the guard fires on an artifact of the
initialisation convention. The fix is a register that is 0 only at time zero:

```bash
grep -n "fv_started" formal/double_buffer_props.sv
```

**77d. And that artifact nearly produced a fabricated result.** With
`a_reset_reads_a` refuting, the *whole suite* refuted on the unmutated design —
so every mutant also refuted, and the first bite measurement read **4 of 4
detected**. The honest figure is **2 of 4**; the two misses are mutations of an
unused lint-suppression wire, which no property should catch.

**A detection measurement is meaningless unless the suite proves on the real
design first**, and the harness now refuses to run without that baseline. This is
Prop. 28's baseline gate, rediscovered from the other side: that one exists so a
*probe* verdict means something, and this is the same requirement for a *bite*
verdict.

**77e. Adding a suite is four edits, not one.** The prove step, the
assumption-liveness probes, `phantom_scan`'s suite list, and the property count
in README. Miss the third and the new suite is exempt from the gate that catches
properties referencing signals which do not exist; miss the fourth and
`claims_check` fails — which it did, immediately.

---

### Prop. 78 — the memory axiom, over a symbolic address — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove weight_bram properties*

Second of the INDIRECT modules from Prop. 76. `weight_bram` is the memory the
prefetch fills and the compute stage reads; Prop. 34's `DEPTH` scaling and the
`memory_map` pass in every engine proof both exist because of it, and nothing
stated what it is supposed to do.

**78a. One property, and it is the whole memory axiom.**
`a_read_returns_last_write`: a read of an address returns the last value written
to it. The address is **symbolic** — a free input held constant by assumption —
which is what makes one property also cover **non-interference**: if a write to
any other address disturbed this one, the shadow would disagree. A fixed address
would have proved something far weaker.

**78b. Collision semantics are load-bearing.** `rd_data <= mem[rd_addr]` and
`mem[wr_addr] <= wr_data` are both non-blocking, so a read concurrent with a
write to the same address returns the **old** value. The shadow is therefore
compared as of the read cycle, before that cycle's write — which is what
`$past(fv_mem)` expresses. Get this backwards and the property refutes on a
correct memory.

**78c. It refuted first, and the counterexample named the cause exactly.** At
cycle 2 the solver wrote to address **2048** of a four-entry array. `DEPTH` is
scaled to 4 by `chparam` for tractability while `ADDR_WIDTH` stays 12, so most
representable addresses are out of bounds. The fix is an in-range assumption —
and its provenance matters: **at the real depth the assumption is vacuous**,
because `DEPTH` is 4096 and `ADDR_WIDTH` is 12, so every representable address is
legal. *It constrains nothing about the design and exists solely to keep the
scaled-down proof faithful.* An assumption that would be vacuous at full scale is
the only kind that can be added to a scaled proof without weakening it.

**78d. Zero mechanical mutants detected, and that is a fact about the mutants.**
The module is 28 lines and yields **three** parsing mutants, all of them
width-expression edits (`ADDR_WIDTH-1` → `+1`, `DEPTH-1` → `+1`) which widen a
port or an array without producing a memory fault. Per Prop. 48b, a sweep that
finds nothing must demonstrate it could have — so the property was run against
faults a memory can actually have:

| targeted fault | verdict |
|---|---|
| read the write address | **caught** |
| ignore the write enable | **caught** |
| write to the read address | **caught** |
| read one address early | **caught** |

Four of four. The mechanical operator set is simply blind to this module, which
is worth stating rather than reporting "0/3" and letting it read as weakness.

**78e. Coverage: 9 direct → 10, 7 indirect → 6.** Four remain INDIRECT
(`pipeline_stage2_compute`, `adder_tree_27`, `trit27_dot_product`,
`trit27_parallel_multiply`, `trit_full_adder`, `trit_half_adder`).

---

### Prop. 79 — the accumulator, checked without trusting the primitive — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove pipeline_stage2_compute properties*

Third INDIRECT module from Prop. 76 and the last non-trivial one — the six that
remain are combinational primitives inside `trit_stdlib.sv`. This is the MAC
datapath, and Wave 615's undetected activation/requant mutation lived beside it.

**79a. A shadow instance, not an assumption.** The properties are about the
**accumulation**, so a second `trit27_dot_product` is driven with the same inputs
to supply the expected per-chunk contribution. That assumes nothing about whether
the primitive is correct — it lets each property say exactly what the surrounding
logic must do with *whatever* the primitive returns. The primitive's own
correctness remains a separate, unmade claim.

**79b. Four properties, all proving, 4 of 4 mutants caught.**

| property | what it pins |
|---|---|
| `a_first_chunk_restarts` | a first chunk **restarts** the sum — drop the test and the accumulator runs across neuron boundaries |
| `a_accumulates_one_chunk` | every later chunk adds exactly its own contribution |
| `a_result_held_when_idle` | the result is held while no chunk is accepted |
| `a_valid_out_follows_last` | `valid_out` is exactly "a last chunk was accepted last cycle" — not sticky, not "any chunk" |

Every mechanical mutant is detected: both `+`→`-` edits and both ternary swaps,
which are precisely the accumulate-vs-restart confusions the suite is aimed at.

**79c. The coverage map needed fixing before it could record this.** `DIRECT`
was defined as "a `formal/` suite instantiates it" — and this wrapper
instantiates `trit27_dot_product` a **second** time as a shadow. That would have
reported the primitive as directly verified while no property says anything about
it. Coverage now requires the instance named `dut`, the convention every wrapper
here follows: **an auxiliary instance is not coverage.**

A wave that adds a property can corrupt the map that measures properties, and
the corruption reads as progress — one more module apparently covered.

**79d. Coverage: 10 direct → 11, 6 indirect → 5.** The five remaining are all
combinational primitives (`adder_tree_27`, `trit27_dot_product`,
`trit27_parallel_multiply`, `trit_full_adder`, `trit_half_adder`), which are
better served by one exhaustive-over-inputs proof than by five wrappers — stated
as the next step rather than left as an implied gap.

---

### Prop. 80 — an exhaustive proof, a real defect, and a step that could not run — `FIXED`

**Gate:** `formal-yosys.yml` → *Prove trit_stdlib primitives (exhaustive)*

The last five INDIRECT modules from Prop. 76 are purely combinational, and that
changes what a proof means: with no state, `sat -seq 1` quantifies over **every
input combination**. These verdicts carry no depth caveat, no induction argument,
and are the only module results in this campaign exempt from the bound audit of
Prop. 68. Proving them found two things neither the mutation harness nor twenty
waves of gates had.

**80a. `adder_tree_27` was wrong — the campaign's tenth RTL defect.** The tree
returned **−14** for a vector whose balanced sum is **+2**: a difference of
exactly 16, the signature of a four-bit wrap. Level 2 sums three level-1 values
of range [−3,+3], so it spans **[−9,+9]** — and was declared `signed [3:0]`,
which spans [−8,+7]. Any group of nine trits summing to ±8 or ±9 wrapped.

**The RTL's own comment said `range [-9, +9] -> signed [3:0]`.** The correct
range was written directly above the declaration that could not hold it, for
every wave since Wave 33.

**80b. A test was pinning the defect.** `adder_tree_27_has_three_reduction_levels`
asserted `body.contains("wire signed [3:0] l2 [0:2];")`. The bug was not merely
untested — it was **protected** by a passing test. *A test that asserts a width
without checking the range it must cover locks in whatever the emitter first
produced.* Fixed in the emitter (`bootstrap/src/trit_stdlib.rs`), not the
generated file, and the test now demands five bits and says why.

This propagated: `trit27_dot_product` uses the tree, `pipeline_stage2_compute`
uses the dot product, and the engine uses that. Prop. 79a deliberately left the
dot product's correctness unstated while checking the accumulator around it —
and the thing it declined to assume was in fact broken.

**80c. And the engine steps could not run in a clean checkout.**
`build/rtl/trit_stdlib.sv` is **not in the bundle**: `BUNDLE_ORDER` lists twelve
files and this is not one of them. Every engine-level step lists it as a source.
With CI's exact source list and a fresh emit:

```bash
grep -c "gen-trit-stdlib" .github/workflows/formal-yosys.yml
```

| emit | `prep -top bitnet_engine_top` |
|---|---|
| `gen-bitnet-bundle` alone | **exit 1** — `File 'build/rtl/trit_stdlib.sv' not found` |
| plus `gen-trit-stdlib` | exit 0 |

It worked locally only because an older `gen-trit-stdlib` run had left the file
behind — a stale artifact standing in for a build step. The emit step now
generates it.

**80d. I nearly published this wrong, twice.** The first test globbed `*.sv`,
which includes `behavior_sva_v2.sv` — concurrent SVA that yosys cannot parse at
all (Props. 2/5/6) — so it failed for a reason having nothing to do with the
finding. And an earlier check read exit status through a `grep` that missed the
error line entirely, briefly suggesting the engine elaborated fine. **Both times
the harness was wrong, not the tree**, and the claim only became sound when the
test used CI's *exact* source list.

**80e. Five exhaustive results, and the non-vacuity oracle that makes them
mean something.** All five prove: the half- and full-adder balanced-ternary
axioms, lane-wise multiplication, the 27-trit tree sum, and the dot product. Each
assumes inputs are valid trits, so a wrapper asserts that **no** valid vector
exists and must refute — Prop. 12a's oracle in the only form stateless logic
admits.

**80f. The reserved encoding is handled inconsistently, and that is recorded
rather than fixed.** `adder_tree_27` maps `2'b11` to 0 via an else branch;
`trit27_parallel_multiply` tests `ai == bi` and so treats `2'b11 * 2'b11` as +1.
Both are defensible readings of "reserved" and they disagree. The proofs assume
it away; the disagreement is stated so the assumption is not mistaken for its
absence.

**80g. Coverage: 11 direct → 16, 5 indirect → 0.** No module in the emitted
bundle is now constrained only at one remove.

---

### Prop. 81 — nothing moved, and that is the finding — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Prop. 80 fixed a real arithmetic defect in `adder_tree_27`, which feeds
`trit27_dot_product`, which feeds `pipeline_stage2_compute`, which feeds the
engine. Every engine verdict in this campaign — Props. 25, 34, 53, 55, 66, 67 —
was obtained with that defect present in the design. Re-establishing them is not
optional bookkeeping; a defect was fixed underneath every one of them.

**81a. All six engine-level steps pass on the corrected RTL.**

| step | exit | seconds |
|---|---|---|
| Baseline — unprobed design must prove | 0 | 4 |
| Integration, core 24 at `seq 80` | 0 | **422** |
| Integration, all 28 at `seq 40` | 0 | 183 |
| Engine is still alive under its interlocks | 0 | 58 |
| Oversized requests do not wrap the local address | 0 | 7 |
| `pipeline_stage2_compute` | 0 | 2 |

Nothing moved. The state space is unchanged — the fix widened a *wire*, not a
register — so the bounded results stand exactly as measured.

**81b. And that is the uncomfortable part.** The 28 integration properties
proved **both before and after** a genuine arithmetic defect in a module they
transitively depend on. A tree that returned −14 instead of +2 for ordinary
inputs did not disturb a single one of them.

That is not a failure of those properties; it is a precise statement of what
they constrain. They are claims about **control** — handshakes, buffer phase,
address contiguity, readiness — and the defect was in **data**. Prop. 68d
predicted exactly this from the other direction: the engine mutations that
nothing caught were the ones that "change a *value* while leaving every activity
reachable". Here is the same boundary, drawn by a real defect rather than a
mutant.

**81c. What actually caught it.** Only the exhaustive combinational proof, and
only because a module classified INDIRECT two waves earlier was given properties
at all. The chain that found this defect is: map coverage (Prop. 76) → notice a
module constrained only at one remove → prove it directly (Prop. 80). No
mutation, no witness, and no integration property was involved at any point.

**81d. A measurement for the scale ceiling.** ~~Prop. 55 recorded 22 core
properties at `seq 80` in 238 s. The same bound now costs **422 s for 24** — the
two properties added since carry most of that. The ceiling documented in
Prop. 34 has not moved, but the headroom under it has narrowed, and that is
worth knowing before the next property is added at that bound.~~

> **WITHDRAWN by Prop. 91c**, annotated here by Prop. 94 because the withdrawal
> was recorded 650 lines away and this paragraph still read as a live
> conclusion. The 422 s endpoint re-measures at **309.9 s** (27% low); the
> 238 s endpoint describes a 22-property configuration that no longer exists.
>
> The citation is also wrong, and Prop. 91c repeated it: **Prop. 55a records
> 245.1 s**, not 238. The 238 is Prop. 54a's **237.8 s**, measured *before* the
> deep/core split landed — a different configuration again.

Reproduce:

```bash
grep -c "seq 80" .github/workflows/formal-yosys.yml
```

---

### Prop. 82 — the defect was written down next to itself for 595 waves — `FIXED`

**Gate:** `formal-yosys.yml` → *No declaration is narrower than the range it carries*

The Wave 628 defect was not hidden. `adder_tree_27` carried this, verbatim:

```
// Level 2: 3 groups of 3, range [-9, +9] -> signed [3:0].
wire signed [3:0] l2 [0:2];
```

The correct range and the width that cannot hold it sat on adjacent lines from
Wave 33 to Wave 628. A unit test asserted the wrong width verbatim, so the
defect was not merely untested but *protected*. Nothing mechanical ever
compared the two numbers, and no human read them as numbers either.

**82a. The gate.** `formal/width_scan.py` reads emitted RTL and makes three
comparisons: a documented range against its declaration's width; a reduction's
operands against what the target is declared to hold; and those same operands
against what the target's own comment claims. The second exists because the
first is defeated by "correcting" the comment; the third catches documentation
drifting from a design that still happens to fit.

**82b. Ranges must propagate from comments, not from widths.** The obvious
implementation — worst-case arithmetic over declared widths — is *unsound here*
and fails a correct design. `val` is `signed [1:0]` but holds only {−1, 0, +1}:
a trit needs three values and two bits carry four. Reasoning from bits gives
[−2,+1] per element, makes level 1 span [−6,+3] against a declared [−4,+3], and
reports a defect in correct RTL. Worst-case-by-width is wrong wherever an
encoding is narrower in value than in bits, which for ternary hardware is
everywhere. So a declaration's range is what its comment says, and only falls
back to its width when unannotated.

**82c. The first draft passed by checking less than it claimed.** It reported
zero findings on the shipped tree — and zero on the *injected defect* too. Two
causes: the comment block above `l2` runs eight lines and the lookahead was
three; and `val[i*3+1]` puts a `+` inside an index, so an operand-count guard
saw five terms where three exist and silently declined to check level 1. A
clean result, from a check that never ran. The summary line now reports
reductions checked (3), and zero of them is a failure.

**82d. What it catches, verified by injection.** Against the shipped tree, 0
findings; the Wave 628 defect re-injected, 2; the same defect with its comment
"corrected" away, 1; a wide-enough width whose comment understates it, 1. Each
injection is asserted to have actually changed the text, because an injection
that silently no-ops grades the scan on unmodified source and calls it a pass.

**82e. Scope, stated rather than implied.** 16 signed declarations across 13
emitted files, 3 range-annotated, 3 reductions checked. That is small, and it is
the honest number: these are the only conventions the emitters currently write.
This is not a general Verilog width checker. It is the specific check that would
have caught the specific defect, generalised as far as the emitters' own habits
allow — and every level of the tree that produced the defect is now covered.

**82f. Adding the gate exposed a stale claim in the README.** Wiring the new
step in made `absence_sweep` report 32 steps while the README said it runs "all
**22** checking steps". Nothing broke — the sweep had been walking every step
all along — but the number in the prose had drifted from the tree across roughly
twenty waves as steps were added, exactly the failure Prop. 73 described. It is
now the sixth gated claim in `claims_check.py`, and the count is derived by
**importing `absence_sweep.collect`** rather than re-counting: two independent
counters of the same thing drift, which is how this number got to be wrong.
True value **31** — 32 walked, less one exempt step.

Reproduce:

```bash
python3 formal/width_scan.py --self-test && python3 formal/claims_check.py
```

---

### Prop. 83 — the accumulator is safe because of a contract written nowhere — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove the accumulator cannot overflow (unbounded)*

Wave 630 closed by asking whether any *other* test pinned a value that could be
wrong. Auditing the 36 distinct width pins across the `t27c` suite found none
stale — but one of them, `reg signed [15:0] accumulator`, pointed at a question
nothing in the tree answered: **is 16 bits enough?**

**83a. The existing property could not have caught an overflow.**
`a_accumulates_one_chunk` (Prop. 79) asserts
`result == $past(result) + $past(fv_dot)`. That is a **16-bit equation**, so it
holds *modulo 2¹⁶* — an accumulator that wraps satisfies it exactly. Four
properties constrained this datapath and none of them said anything about width
sufficiency.

**83b. The module cannot answer the question itself.**
`pipeline_stage2_compute` has **no chunk counter and no `num_chunks` input**. It
accumulates for as long as `valid_in` is held with `first_chunk` low. In
isolation it overflows after **1214** chunks of +27. The width is sufficient
only because of a **caller contract**: `layer_sequencer` walks `chunk_id` over
an 8-bit port, so at most 255 chunks separate two `first_chunk` strobes, and
255 × 27 = 6885 sits well inside [−32768, +32767].

That reasoning appeared **nowhere in the tree** — not in the module, not in a
comment, not in an assumption. It is safe by accident of an unrelated port
width. Widening `num_chunks` to 16 bits to support larger layers — an ordinary
change to a different file — silently reintroduces the wrap.

**83c. Induction, not a bound.** The overflow is 1214 cycles away, so *every
feasible depth reports "proves" and means nothing*. `ps2_bound` states the
inductive invariant |acc| ≤ 27·n and the corollary that the accumulator always
retains headroom for one more chunk. Both prove by **k-induction at length 4** —
unbounded, no depth caveat. Base case and induction step both discharged, 3
`$check` cells.

**83d. The per-chunk bound is proved, not assumed.** `a_dot_product_correct`
(Prop. 80) states the dot product's exact value but only under `all_valid`,
which excludes the reserved code `2'b11`. The *bound* needs no such assumption —
the decoder maps every code that is neither `TRIT_N` nor `TRIT_P` to zero — so
`dot_range_props` asserts |dot| ≤ 27 unconditionally and exhaustively. Stated
separately because a fact a proof depends on should be proved rather than left
implicit in another property's cone; had it been available only under
`all_valid`, the accumulator bound would have silently inherited an assumption
about BRAM contents that nothing enforces.

**83e. Cost, and why the first structure was abandoned.** Stating these
properties inside the existing `ps2_props` wrapper — which carries a shadow
`trit27_dot_product` — put **two** 27-input adder trees inside an inductive
proof: killed at 18 minutes without finishing. The invariant needs only
`result` and the chunk counter, so a separate lean wrapper without the shadow
proves the same claims in **1.3 s**. Same properties, same design, 800× the
speed, because one of them was in the cone and the other did not need to be.

**83f. Three bars, and I nearly recorded the third from a tool error.** The
suite proves (83c); `ps2_bound_alive` asserts the accumulator is always zero and
**refutes**, so the contract does not freeze the datapath. For the third — is
the contract load-bearing? — a relaxed variant should refute. Two runs reported
exit 1 and were nearly written down as "refuted, the assumption is
load-bearing". They were **`ERROR: File not found`**: a `cd` in an earlier
command had moved the shell, and `returncode != 0` folded a tool error into a
verdict. This is Prop. 39d exactly, in the wave that cites it. Re-run with
absolute paths, the relaxed control did **not complete in 40 minutes** — the
solver must drive the adder tree to ±27 to build the counterexample — so it is
recorded as not completed, not as a verdict. The contract's necessity therefore
rests on the arithmetic (27 × 1214 > 32767), which is stated, not on a
refutation, which is not.

**83g. Two more stale README claims, and a gate that was silently not gating.**
Adding one CI step moved the swept count 31 → 32. Worse, the README still
described Prop. 76's Wave 618 module split — "8 have properties of their own, 8
constrained only at one remove" — when four waves of properties had since made
it **16 and 0**: *every module the engine reaches now has properties of its
own.* Both are now derived claims. And `claims_check` itself had the campaign's
oldest bug: **a claim whose regex matches nothing printed nothing and was
counted as covered**. Rewording the module-coverage sentence retired that check
silently — caught only because the new UNMET guard fired on its own first run.

**83h. A boundary correction.** The published "58 module properties" included
`at27_alive`, a non-vacuity **oracle** that asserts something false so its
refutation proves an assumption admits inputs. An assertion that must fail is
not a proved property. `*_alive` modules are now excluded: the previous 58 was
57 properties and one oracle, and the figure is **60** with this wave's three.

Reproduce:

```bash
python3 formal/claims_check.py && python3 formal/orphan_scan.py
```

---

### Prop. 84 — fifteen growing registers, and what each is safe relative to — `MEASURED`

**Gate:** `formal-yosys.yml` → *Every growing register says what bounds it*

Prop. 83 was not an incident but a class: a register that grows is safe only
relative to a bound, and the interesting question is never "is it wide enough"
but **"wide enough for what, and where is that written"**. `formal/bound_scan.py`
answers the second question mechanically for every `X <= X + k` in the bundle.

**84a. The map.** 15 growing registers across 13 emitted files:

| class | n | meaning |
|---|---|---|
| LOCAL | 4 | compared against a constant in its own module — the bound travels with the logic |
| CONTRACT | 4 | compared only against an **input port** — the bound is real but lives in the caller |
| FREE | 7 | nothing in the module compares it at all |

FREE does not mean broken. It means the argument is somewhere else, or nowhere,
and the RTL cannot tell you which. That indistinguishability is the finding.

**84b. The gate requires the argument, not a proof.** Every CONTRACT and FREE
register must carry a `// BOUND: <name> <reason>` note. All 15 now do, and
writing them was the work: each had to be traced to a real limit. This proves
nothing safe. It makes a *missing argument* visible, which is the step that was
absent when a 16-bit accumulator went 600 waves without anyone asking what
limited it.

**84c. Two clamps that are tight to the bit.** `dma_controller.word_index` is
12 bits; `length` is clamped to 32768 bytes and one beat is 8 bytes, so at most
**4096** beats issue — exactly the 4096 values 12 bits hold. Likewise
`weight_prefetch_ctrl.word_index` against a 4096-word clamp. Neither bound is a
comparison on the index; both live in a separate countdown. Raising either clamp
by one wraps an index, and nothing in either module would say so.

**84d. One 32-bit address where the others are 64.** `weight_prefetch_ctrl`
advances `axi_araddr` 8 bytes per word from a caller-supplied `src_addr`, up to
32768 bytes, in a **32-bit** register — while `dma_controller`'s equivalents are
64-bit. Wrapping the DMA's needs a buffer within 32 KiB of 2⁶⁴; wrapping the
prefetcher's needs one within 32 KiB of the 4 GiB ceiling, which is reachable on
a real memory map. Nothing checks it. Recorded as a caller contract, not
claimed as a defect — and found only because the sweep forced an argument for
each register rather than each module.

**84e. I nearly wrote that finding about the wrong module.** The first draft
said the DMA's address registers were the 32-bit ones, from a `grep` of
assignment lines that never showed a width. The emitter says
`output reg [63:0] m_axi_araddr`. Checking the declaration rather than the use
moved the finding to a different module and changed what it means.

**84f. The scan's first draft misclassified the register it exists because of.**
It accepted `<=` as a comparison. In Verilog `<=` at statement level is the
nonblocking **assignment**, so `accumulator <= first_chunk ? ...` read as
"accumulator is compared against first_chunk", and the Prop. 83 accumulator —
bounded by nothing — was reported as bounded by a contract. Every LOCAL verdict
in that draft came from a reset assignment `X <= 0` read as a bound: the whole
table measured assignments. Dropping `<=` and `>=` loses genuine `if (c <= lim)`
bounds, which then read as FREE and demand a note — over-reporting, in the
direction that asks for an argument rather than inventing one.

The acid test for an instrument is a case whose answer you already know. This
one had exactly one, and failed it.

Reproduce:

```bash
python3 formal/bound_scan.py --self-test && python3 formal/bound_scan.py
```

---

### Prop. 85 — the countdowns that enforce the tight bounds — `PROVED`

**Gate:** `formal-yosys.yml` → *The prefetch countdown cannot underflow (unbounded)*

Prop. 84 found two 12-bit word indices sized at *exactly* their 4096-entry
limit, and noted that neither bound is a comparison on the index: both are
enforced by a **separate countdown**. That makes the countdowns load-bearing,
and a countdown has its own failure mode — the mirror of overflow. `X <= X - k`
wraps to near 2ᴺ the moment `X < k`, and a wrapped countdown does not stop; it
runs for another 2ᴺ steps. `bound_scan` now classifies these as DRAIN and
requires the same written argument.

There are exactly three, in two modules.

**85a. `weight_prefetch_ctrl.words_remaining` — proved, unbounded.** The
terminator fires at exactly 1, so the register reaches 0 and never wraps; the
`num_words == 0` case is guarded separately. Stated as two **inline** properties
in the module rather than in `weight_prefetch_props.sv`, because
`words_remaining` is internal and a wrapper referencing `dut.words_remaining`
would not error — it would declare an undriven one-bit wire of that name and
prove against it, which is exactly how a property here spent four waves reading
nothing (Prop. 62). Proved on the module directly by **k-induction**, so the
verdict covers every request length rather than a depth.

Three bars, all cleared: it **proves** (`-tempinduct`, exit 0); the run compiles
**2 `$check` cells**, so the properties are present rather than silently
excluded by the guard; and it **bites** — changing the terminator from
`== 16'd1` to `== 16'd0`, the off-by-one that would produce the underflow,
refutes it.

**85b. `dma_controller.bytes_remaining` — underflows by design, and that is
fine, conditionally.** It counts down by 8 per beat while the exit test is
`bytes_remaining <= 8`. For any length that is not a multiple of 8 the final
beat wraps it: a 12-byte request goes 12 → 4 → `0xFFFFFFFC`. This is harmless for two
distinct reasons, worth separating. The exit test `bytes_remaining <= 8` is in
the same always block and so samples the **pre-decrement** value: it sees 4, not
the wrap, and sends the FSM to `DONE_ST`. `beats_owed = (bytes_remaining + 7)
>> 3` is a *continuous* assignment and therefore does track the wrapped value —
it is safe not because it misses the wrap but because it is only read in
`READ_ADDR`/`WRITE_ADDR`, states the FSM does not re-enter after leaving. The
next `start` then overwrites the register.

That argument has a condition: it holds while the slave honours the `arlen` the
controller issued. An extra beat past `rlast` would feed the wrapped value into
`beats_owed` and request a 2³²-byte burst. Recorded as an AXI-protocol
dependency rather than proved, because it is a claim about the environment.

**85c. Where the argument had to live.** `words_remaining` could not be
constrained from the existing wrapper's ports at all. The observable
consequence — an underflow would write past the request — *is* covered by
`a_no_overwrite`, but only to that step's depth, and the terminator for a large
`num_words` sits far beyond it. Moving the property inside the module is what
made an unbounded verdict possible. Sometimes the right place for a property is
not the property file.

**85d. Two module properties cost the engine 58%, and that decided where they
live.** These were first guarded with `T27_FORMAL` — the same define the
engine's integration steps pass, and those steps read this file. Two
module-level assertions therefore joined the engine's obligation set without
anyone intending it.

Measured on an idle machine, same invocation, only the guard differing:

| engine step *"all 28 at `seq 40`"* | `$check` cells | seconds |
|---|---|---|
| without the drain properties | 31 | **153** |
| with them | 33 | **241** |

**1.58×, +88 s from two properties** — against a ceiling Props. 34 and 81d
already flagged as narrowing. They now sit behind their own `T27_FORMAL_DRAIN`;
the engine keeps its 31 cells, the module step keeps its 2, and nothing is lost,
because induction already proves them for *every* request length while the
engine would re-prove them to a depth of 40.

The general point: an inline property is compiled by whoever passes its guard,
not by whoever wrote it. Check the `$check` count of every step that reads the
file, and pick the guard for cost as deliberately as for taxonomy.

**85f. That table is a correction — the first version of this proposition
reported 4×.** The original figures were 723 s with the properties and 332 s
without, giving "183 s → 723 s, a 4× cost", and they were published in this
file, the README, a commit message and issue #2061. Both were measured while
**three other yosys processes were running**, a condition I did not control for
and did not record. The clean re-run put the no-properties case at 153 s — 
*faster* than the 183 s baseline it was supposedly a regression against, which
is what exposed the contamination.

The direction of the finding survived; its magnitude was wrong by 2.5×, and the
sentence "a 4× cost from two properties" asserted a causal attribution that the
measurement could not support. The lesson is narrow and mechanical: a timing
figure is a claim about a machine state, so record the state or do not publish
the number. Two of this campaign's most-quoted corrections — Props. 67a and 73 —
were also caption errors on instruments that worked correctly.

**85e. And a comment that claimed more than its property.** The second inline
assertion was introduced as establishing non-vacuity — "the drain does reach 1,
where the terminator has to do the work". It asserts `words_remaining <= 4096`,
which says nothing of the sort. It is a real property (removing the clamp would
refute it, and the index bound depends on that clamp) but it is not a
reachability argument, and the comment was rewritten to claim only what the
assertion checks. Non-vacuity here comes from the `$check` count and the
mutation, not from this.

Reproduce:

```bash
python3 formal/bound_scan.py --self-test && python3 formal/bound_scan.py
```

---

### Prop. 86 — the six unreached primitives are an algebra, and it is now proved — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove the trit algebra (exhaustive)*

Prop. 76 found six ternary primitives instantiated by nothing in the bundle
while being read into every proof as source. They stayed UNREACHED for five
waves as an open question — retire them, or wire them in? Neither. They are not
dead code and not missing plumbing: they are an **algebra**, and an algebra can
be stated as theorems and proved outright.

Every property is combinational over at most 12 input bits, so `-seq 1`
quantifies over **every** input combination. No depth caveat, no induction, no
assumption beyond trit validity.

**86a. The theorems.** Writing T = {−1, 0, +1} with the encoding
`2'b00 = −1`, `2'b01 = 0`, `2'b10 = +1`:

> **T1 (negation).** `trit_not` computes −a, and is an involution: not(not a) = a.
>
> **T2 (De Morgan algebra).** `trit_and` = min and `trit_or` = max under
> −1 < 0 < +1. Hence (T, and, or, not) is a **De Morgan (Kleene) algebra**:
> both operations commute, absorption holds, and
> ¬(a ∧ b) = ¬a ∨ ¬b.
>
> **T3 (multiplication).** `trit_multiply` computes a·b. Zero absorbs, and the
> units {−1,+1} are closed — the group ℤ/2ℤ.
>
> **T4 (comparison).** `trit_compare` computes sgn(a − b).
>
> **T5 (balanced addition).** `trit3_add` satisfies, over all 4096 input pairs,
> `val(sum) + 27·val(cout) = val(a) + val(b)` where
> `val(w) = w₀ + 3w₁ + 9w₂`.

All five hold. `algebra_alive` asserts no valid word exists and **refutes**, so
the validity assumption is not what makes them true.

**86b. T4 is the one that earns its place, because it is not about the
mathematics.** T1–T3 and T5 are facts about ternary arithmetic and would survive
any faithful implementation. T4 is a fact about *this* implementation:
`trit_compare` compares the raw two-bit encodings with `<`, and that computes
the right sign only because `2'b00 < 2'b01 < 2'b10` happens to agree with
−1 < 0 < +1. The encoding's monotonicity is therefore load-bearing, and it is
written nowhere — the Prop. 83 shape in pure combinational logic. It is now a
first-class assertion, `a_encoding_is_monotone`, rather than a remark.

**86c. The experiment that tested 86b found a second, unpredicted result.**
Permuting the encoding consistently — swapping the codes for −1 and 0 in both
the RTL and the value macro, so the encoding order no longer matches the value
order — should break exactly `cmp_props` and nothing else.

| theorem | shipped encoding | permuted encoding |
|---|---|---|
| T1 not | proves | proves |
| T2 lattice | proves | proves |
| T3 multiply | proves | proves |
| T4 compare | proves | **refutes** — as predicted |
| T5 add3 | proves | **refutes** — *not* predicted |

**86d. Why T5 broke: `trit_full_adder` had the encoding baked in as
literals.** Every other primitive — including the `trit_half_adder` instances
inside this very module — routes through the named `TRIT_N`/`TRIT_Z`/`TRIT_P`
constants. The full adder compared against `2'b10`/`2'b01` and emitted
`2'b10`/`2'b00`/`2'b01` directly. Permuting the encoding moved the localparams
and left this one module behind, silently.

It was also *inconsistent with its own sibling* on the reserved code: the full
adder's default arm mapped `2'b11` to −1 where `trit_half_adder` maps it to 0.
Unreachable in practice — its carries come from half adders, which only emit
legal codes — but two primitives in one file answering the same question
differently is how a later change picks the wrong answer.

Fixed in the emitter to use named constants. **The fix is verified by re-running
the experiment that found it**: under the permuted encoding T5 now proves, and
only T4 refutes — which is correct, since T4 is genuinely encoding-dependent by
design.

**86e. The coverage map is closed.** 23 emitted modules: **22 DIRECT, 0
INDIRECT, 0 UNREACHED**, 1 EXEMPT (concurrent SVA this flow cannot parse). The
question Prop. 76 opened is answered, and answered by proving rather than
deleting — a module with no callers is not necessarily dead, it may simply be a
specification nobody had written down yet.

---

### Prop. 87 — timings get the provenance discipline everything else here has — `FIXED`

**Gate:** `formal-yosys.yml` → *Benchmark harness self-test*

Prop. 85f corrected a published figure — two properties reported as costing an
engine proof 4×, really 1.58× — because both measurements were taken while three
other provers competed for the machine. Nothing malfunctioned. The stopwatch was
accurate; it described a machine I had not.

Correctness results here are reproducible: a proof discharges or it does not,
independent of what else runs. **Timings are not.** They are claims about
contention, core count and thermal state, and this campaign spent twenty waves
gating whether its tools lie while reading its performance numbers off a wall
clock with no provenance at all.

**87a. `formal/bench.py`, three rules enforced rather than remembered.**
*Paired* — both arms run in one invocation, alternating, so they see the same
machine; comparing today's number against one recorded eight waves ago is not a
comparison. *Witnessed* — load average and competing-prover count are sampled
around every run and printed beside the seconds. *Repeated* — each arm runs N
times and the observed range is reported.

**87b. It refuses rather than caveats.** A caveat is something a reader skips.
The harness prints no ratio at all when the machine was contended, when either
arm exited nonzero, or when the two arms' **observed ranges overlap** — because
if some run of the slower arm beat some run of the faster one, no ordering
between them is supportable. That last criterion is deliberately conservative:
it is what would have refused to print Prop. 85f's 4×.

**87c. Its first real use produced an impossible answer, and that was the
finding.** Re-measuring Prop. 85d's comparison, it reported **0.88× — adding two
properties made the proof 19 s *faster*** — with disjoint ranges, zero competing
provers, and every guard satisfied.

The cause was not the machine. It was me: I regenerated the RTL bundle roughly a
third of the way through the run, so the early and late samples measured
different inputs. **A benchmark whose inputs move mid-run is exactly as broken as
one whose machine is contended, and neither is visible in the seconds.**

The harness now fingerprints every file under test before and after the run and
rejects the comparison if the digest moved (`--watch`). Six self-test cases: a
clean run reports; a failing command is not timed; identical arms yield no
ratio; contention blocks the report; an input edited mid-run blocks the report;
and stable inputs still report.

**87d. The general rule.** An implausible measurement is evidence about the
instrument, not a surprising fact about the world. 0.88× was not a discovery
that properties make proofs faster — it was the harness telling me, in the only
way it could, that it was not measuring what its labels said. The three
contaminations this campaign has now recorded — a contended machine (85f), a
tool error read as a verdict (83f), and inputs changing underfoot (87c) — share
one shape: **the number was fine, the thing it described was not what the
caption claimed.** That is the same shape as Props. 67a and 73.

---

### Prop. 88 — the DMA drain, proved where it is actually consumed — `PROVED`

**Gate:** `formal-yosys.yml` → *The DMA drain is never consumed after it wraps (bounded)*

Prop. 85b left one hand-argument standing. `bytes_remaining` underflows **by
design** — a 12-byte request goes 12 → 4 → `0xFFFFFFFC` — and is safe only
because the wrapped value is never read. That was reasoning in a comment.

**88a. The claim is not "it never wraps".** It cannot be: the wrap is
intentional. The safety claim is that *wherever the value is consumed, it is
still a sane residue*. `beats_owed = (bytes_remaining + 7) >> 3` is a continuous
assignment and does track the wrap; it is safe purely because it is read only in
`READ_ADDR`/`WRITE_ADDR`, states the FSM does not re-enter after leaving on that
beat. So the property is stated over exactly those states:
`bytes_remaining <= 32768`, plus that the surviving residue is non-zero.

**88b. It needs the environment, and the environment already existed.** In
isolation the claim is **false**: an extra data beat past `rlast` with fewer than
8 bytes owed wraps the counter while the FSM stays in `READ_DATA`. The AXI
read-slave model written in Wave 612 already assumes exactly what is needed —
`rlast` lands on the (arlen+1)-th beat — so the proof runs in the existing
`dma_props` wrapper against that model. The protocol dependency Prop. 85b
recorded in prose is now the assumption the proof is stated under.

**88c. Bounded, and the bound is reported honestly.** Proves at **`seq 12` in
1.4 s** and at **`seq 24` in 285 s**. At `seq 80` — the bound the rest of
`dma_props` runs at — it **did not complete in 30 minutes**, so it is recorded as
not completed rather than retried until it produced a number (Prop. 68's rule).
CI runs it at 24. The steepness is itself worth knowing: 200× the time for 2× the
depth on the same wrapper.

---

### Prop. 89 — the lemmas T5 is made of, and a specification that caught itself — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove the trit algebra (exhaustive)*

Prop. 86's T5 proves `trit3_add`'s equation directly over all 4096 input pairs.
That is a fact about the assembled tree and says nothing about *where* a failure
would be. Two lemmas now sit under it, both exhaustive:

> **H (half adder).** `val(sum) + 3·val(carry) = val(a) + val(b)`
>
> **F (full adder).** `val(sum) + 3·val(cout) = val(a) + val(b) + val(cin)`

T5 follows from F by the positional argument — three full adders chained by
carry, the k-th weighted 3ᵏ, telescoping to the 27s place. That derivation is
mathematics rather than something this flow performs, so T5 remains
independently machine-checked. What the lemmas buy is **localisation**: if T5
ever refutes while H and F prove, the arithmetic is right and the wiring is
wrong; if F refutes, the carry rule is wrong. A flat exhaustive proof of the
tree distinguishes neither case.

**89a. F is the non-obvious one.** Its carry is `sign(carry1 + carry2)` from two
chained half adders, which is exact only because those two carries can never be
simultaneously non-zero with the same sign — so their sum never leaves
{−1,0,+1} and the "sign" is in fact the exact sum. That is a claim about
reachable states of an internal pair: cheap to get wrong, free to check
exhaustively.

**89b. The first thing this lemma caught was itself.** F's third assertion was
first written as a rounding formula, `(x+1 − (x+1) % 3) / 3`, and it **refuted**.
The adder is correct; the *specification* was not. Verilog's `%` takes the sign
of its dividend, so for a total of −3 the formula yields 0 where the carry is
−1. Isolating the assertion confirmed the design proves without it.

It was replaced by two statements that are correct and say something
conservation does not: the carry is non-zero **exactly** when the total leaves
{−1,0,+1}, and it takes the total's sign. The discarded version was also
redundant — conservation plus trit-validity already determines `cout` uniquely —
so it was both wrong and unnecessary, and only the wrongness surfaced it.

A refuting property is not evidence of a defect until you have checked which of
the design and the property is wrong. This campaign has now recorded that in
both directions: Prop. 80 found real RTL defects this way, and here the RTL was
innocent.

---

### Prop. 90 — the encoding permutation becomes a standing gate — `MEASURED`

**Gate:** `formal-yosys.yml` → *Permuting the trit encoding breaks exactly the right theorems*

Prop. 86c tested one claim by breaking it and found a defect nobody predicted.
An experiment with that hit rate should not be run once. `formal/encoding_gate.py`
permutes the trit encoding — swapping the codes for −1 and 0, in both the RTL
localparams and the property file's value macro — and checks the resulting split
against a declared table.

**90a. Two-sided, and the second side is the one that would rot.**

- **No new breaks.** A theorem that survives today must survive the permutation.
  A new failure means some primitive has acquired a hidden dependency on the
  literal encoding — the Wave 634 defect recurring.
- **No lost breaks.** `cmp_props` must *still* refute. It is encoding-dependent
  by design, and if it stops, either the comparison was rewritten (fine, but the
  table is stale) or **the permutation stopped permuting** — and a gate that
  asserts only "nothing broke" passes when its own perturbation has become a
  no-op. That is this campaign's oldest failure shape (Props. 58–60), and it is
  the reason the expected-refutation entry exists rather than being exempted.

**90b. Result.** 9 theorems permuted, 18 localparam sites and the value macro
rewritten, 0 disagreements: the eight encoding-independent theorems prove and
`cmp_props` refutes. Four self-test cases, including **re-injecting the exact
Wave 634 defect** — `trit_full_adder` comparing against literals — and confirming
it is caught.

**90c. Why permute both sides.** Permuting only the RTL would break every
theorem trivially and prove nothing about any of them; permuting only the macro
likewise. The perturbation has to be *semantics-preserving* for the result to
mean anything, which is what makes a surviving theorem evidence of
encoding-independence rather than of a broken experiment.

---

### Prop. 91 — the scale ceiling, re-measured, and a conclusion withdrawn — `MEASURED`

**Gate:** `formal-yosys.yml` → *Benchmark harness self-test*

Prop. 87 built a harness because Prop. 85f published a timing that was wrong.
The obvious next question is what *else* rests on timings taken the same way.
The answer is the campaign's scale-ceiling argument.

**91a. The provenanced baseline.** Both arms alternating in one invocation,
2 runs each, **0 competing provers**, load 5.1 on 8 cores, and the input
fingerprint **identical before and after** the run:

| engine step | median s | observed range |
|---|---|---|
| all 28 at `seq 40` | **154.5** | [150.0, 159.0] |
| core 24 at `seq 80` | **309.9** | [307.5, 312.2] |

Ranges disjoint, ratio 2.01×. This is the first performance figure in this
document that carries the conditions it was measured under.

**91b. Both published numbers were high.** Prop. 81a recorded **183 s** and
**422 s** for these same two steps. The clean re-runs are **154.5 s** and
**309.9 s** — 16% and **27%** lower. Nothing about the design changed to explain
that; the earlier figures were single samples on an undescribed machine.

**91c. And that withdraws Prop. 81d's conclusion.** It read: *"Prop. 55 recorded
22 core properties at `seq 80` in 238 s, and the same bound now costs 422 s for
24. The ceiling has not moved but the headroom under it has narrowed."*

That inference needs both endpoints. The 422 s endpoint is now known to be
27% high. The 238 s endpoint **cannot be re-measured at all** — it described a
configuration of 22 properties that no longer exists, and it too was a single
unprovenanced sample. With the defensible figure, 238 → 310 for two additional
properties is a far weaker statement than 238 → 422, and it is built on one
number nobody can reproduce.

**The narrowing claim is therefore withdrawn**, not restated with a smaller
coefficient. It was an inference from two measurements, one of which was wrong
and the other unreproducible. What remains is a baseline: 309.9 s at `seq 80`
for the current 24, measured on a described machine, which a future wave can
actually compare against.

**91d. What this says about the other timings in this document.** Props. 34, 55
and 81a all quote seconds recorded the same way. They are left as dated records
rather than deleted — that is this file's convention — but **no argument should
rest on them**, ~~and Prop. 81d was the only one that did~~. Any future
performance claim goes through `formal/bench.py` or is not made.

> **Corrected by Prop. 94.** "Prop. 81d was the only one" is wrong, and the
> list of three propositions is far too short. A systematic audit found **at
> least five further live inferences** built on unprovenanced seconds — Props.
> 37c, 37d-bis, 38d, 54a and 85d — and unprovenanced timings in Props. 35, 36,
> 37, 38, 49, 53, 54, 66, 71, 83, 85 and 88. Withdrawing one inference and
> declaring the rest sound was itself an unaudited claim.

**91e. The instrument reported what it was supposed to.** `inputs: 28 files,
8d5d7d0c62edf664 → 8d5d7d0c62edf664` — the fingerprint that Prop. 87c added
after a mid-run regeneration silently corrupted the harness's first real
measurement. It is printed on every run precisely so that a reader can see the
guard held rather than assume it.

---

### Prop. 92 — the composition itself, proved rather than argued — `PROVED`

**Gate:** `formal-yosys.yml` → *Prove the trit algebra (exhaustive)*

Prop. 89 wrote that T5 "follows from F by the positional argument". That
sentence was doing real work, and it was prose. This discharges it.

> **Corrected by Prop. 93.** 92a's generality claim is empty — the class of
> F-satisfying adders is a *singleton* — 92b's vacuity oracle was defeated, and
> 92c's `mirror_check` claim was false as written. The theorem itself stands.
> Read 93 alongside this.

**92a. The abstraction is every F-satisfying adder at once.**
`fv_abstract_fa` is a full adder about which *nothing* is known except lemma F.
Its outputs are `(* anyseq *)` free signals, assumed only to be valid trits with
`val(sum) + 3·val(cout) = val(a) + val(b) + val(cin)`. It has no case split, no
encoding, no structure. `add3_abstract` chains three of them exactly as
`trit3_add` chains the real ones, and the balanced-addition equation **proves
there**.

So the composition holds for *any* three-stage ripple built from *any*
F-satisfying adder. The concrete `trit3_add` satisfies T5 because
`trit_full_adder` satisfies F — a separate exhaustive proof (Prop. 89). This is
what makes H and F load-bearing rather than decorative: before, they were extra
checks sitting under an independently-proved T5, and the link between them lived
in a comment. Now T5-on-the-real-tree is a corollary of two proved facts rather
than three separate proofs happening to agree.

**92b. The abstraction must be satisfiable, or the proof means nothing.** Had
the assumptions inside `fv_abstract_fa` contradicted each other for some input,
`add3_abstract` would hold vacuously and say nothing about any real adder.
`abstract_alive` asserts the abstract carry is never positive — false of any
F-satisfying adder — and **refutes**, which is the evidence that F admits
something. Gated in CI alongside the proof.

**92c. The abstraction duplicates the wiring, and that is a real weakness.**
There is no way in this flow to instantiate the concrete module's structure with
a different leaf, so `add3_abstract` restates the carry chain by hand. A future
rewiring of `trit3_add` — reordering the trit slices, changing which carry feeds
which stage, passing a different first `cin` — would leave the abstraction
behind, and **the composition proof would keep passing while describing a
circuit no longer in the bundle**. Both modules would still discharge their own
assertions; nothing else in the suite would notice.

`formal/mirror_check.py` compares the two instantiations structurally,
port by port and stage by stage, and fails on any disagreement. Three self-test
cases: the shipped tree mirrors; an abstraction rewired to take the wrong carry
is caught; and a renamed concrete module fails rather than passing silently.
3 concrete stages against 3 abstract, 0 disagreements.

**92d. And the proof genuinely uses F.** A composition proof that held without
its lemma would be proving something about the wiring alone, and would say
nothing about adders. Removing the conservation assumption from
`fv_abstract_fa` — leaving only trit-validity, so the abstract adder may return
any legal trit pair — makes `add3_abstract` **refute**. Three bars, as for any
property here: it proves, the abstraction it rests on is satisfiable (92b), and
it depends on the assumption it claims to depend on.

The general point is one this campaign keeps re-learning in new forms: a proof
about a *copy* of the design is a proof about the copy. The copy has to be
pinned to the original by something mechanical, or the phrase "exactly as" is a
claim nobody is checking.

---

### Prop. 93 — an adversarial review of Prop. 92, and four holes it found — `FIXED`

**Gate:** `formal-yosys.yml` → *Prove the trit algebra (exhaustive)*

Prop. 92 was published, committed and pushed before it was reviewed. An
independent adversarial audit — instructed to try to refute its value rather
than confirm it — found the theorem sound and **four of the claims built around
it false or defeatable**. All four are corrected here; the theorem stands.

**93a. The vacuity oracle was defeated.** `abstract_alive` asserts a single
*unchained* `fv_abstract_fa` cannot produce a positive carry, and refutes. It
therefore establishes "lemma F admits *something*, for *some* input, in *one*
instance". The risk it was supposed to cover is per-input emptiness **inside the
chain**, which it cannot see.

Adding one clause to lemma F — forbidding `sum = 0 ∧ cout = 0`, which makes F
unsatisfiable for any stage whose total is zero — collapses the covered input
space from 4096 pairs to **242 (5.9%)**. Under that injection `add3_abstract`
still proves, `abstract_alive` still refutes, and the CI step stays green: the
theorem would cover six percent of its stated domain with every gate passing.
This campaign's oldest failure shape, an absence read as a pass, reappearing in
a *guard* rather than in a proof.

The replacement, `abstract_is_inhabited`, asserts that the **real** adder
satisfies exactly what the abstraction assumes. It has **no free variables** —
`sum` and `cout` are driven by `trit_full_adder` — and a property with nothing
free cannot hold vacuously.

The first attempt at it did not work, and the reason is worth recording: it
hand-copied the constraint, so an injection into the abstraction's assume block
left the guard's assertion untouched and it kept proving. Lemma F is now written
**once**, as the macro `FA_LEMMA`, assumed by the abstraction and asserted of the
concrete adder. With that sharing, the injection is caught — the guard proves on
the shipped tree and **refutes** under it. Residual risk, stated rather than
hidden: a clause added to the assume block *outside* the macro is still
invisible. The macro makes the honest edit safe; it cannot make a deliberately
split one safe.

**93b. The newest theorem sat outside the gate written to catch its failure
mode.** `add3_abstract` needs a zero to drive the first stage's `cin` and
declares its own `localparam TRIT_Z` in the property file. Prop. 90's encoding
gate substituted over the RTL localparams and the value macro — not over
localparams declared in property files. So under permutation the abstraction
kept the old code while the RTL moved, the two genuinely disagreed, and
`add3_abstract` **refuted**.

It refuted for a real reason, but the reason was an **incomplete perturbation**,
not a defect in the design. A perturbation that is not semantics-preserving
cannot distinguish "this theorem depends on the encoding" from "this experiment
is broken". The substitution now runs over both texts (19 sites, up from 18),
`add3_abstract` is in the expected table, and it proves. It was also *absent
from the table entirely* — the gate reported a clean sweep over nine theorems
while the tenth, the newest one, was never run.

**93c. `mirror_check` compared uses, not declarations.** It compared the
port-connection *text* of the two instantiations. `TRIT_Z` is declared
**separately** in each file — `build/rtl/trit_stdlib.sv` for the concrete tree,
`formal/trit_algebra_props.sv` for the abstraction — so two independent
declarations sharing a name compare equal as strings whatever they hold. Setting
the concrete tree's `TRIT_Z` to `2'b10` while the abstraction kept `2'b01` leaves
the two circuits genuinely different, and the gate reported *0 disagreements*.

Prop. 92c's claim that it catches "passing a different first `cin`" was
therefore **false as written**. It now resolves localparams to their values
before comparing, and the case is a permanent self-test. The irony is exact:
"read the declaration, not the use" is a rule this campaign wrote down in Wave
632, and the gate written to enforce a mirror broke it.

**93d. The generality claim is empty.** 92a said the abstraction "is every
module satisfying F at once". The audit proved `fv_abstract_fa`'s outputs are
bit-identical to `trit_full_adder`'s on all valid inputs — because F plus
trit-validity **uniquely determines** `(sum, cout)` for every total in [−3, 3].
The class of F-satisfying adders has exactly **one element**.

The claim is true, and buys nothing. What the composition proof actually
establishes is narrower and should be stated as such: **T5 does not depend on
`trit_full_adder`'s internal structure** — the two half adders and the
sign-combine — only on its input/output function. Correspondingly, "a corollary
of two proved facts rather than three separate proofs happening to agree"
overstates independence: once the leaf is proved equal, `add3_abstract` and
`add3_props` are the same theorem modulo the leaf's implementation, not two
independent routes to it.

**93e. What the review confirmed.** The wiring does mirror `trit3_add` exactly;
the abstraction is genuinely free at the netlist level (6 `$anyseq` cells
survive `prep -flatten`, nothing folded to a constant); the proof is
mutation-sensitive (rewiring `fa1.cin` refutes, deleting lemma F refutes); and
running without `-set-assumes` refutes rather than proving, so a dropped flag
surfaces as a red rather than a false pass. Swapping `fa2`'s operands still
proves — correctly, since addition commutes — which is a concrete demonstration
that `mirror_check` does work the prover cannot.

**93f. The lesson about the review itself.** Prop. 92 cleared three bars I
designed and named — it proves, its oracle refutes, and it depends on its
assumption. All three were satisfied while four separate claims around it were
wrong. **Bars you choose yourself test what you thought of.** The audit was
instructed to attack rather than confirm, and everything it found lay outside
the checks I had built.

---

### Prop. 94 — every timing in this file, audited — `MEASURED`

**Gate:** `formal-yosys.yml` → *Benchmark harness self-test*

Prop. 91 re-measured two figures and withdrew one inference, then asserted in
91d that **"Prop. 81d was the only one"** resting on unprovenanced seconds. That
sentence was itself an unaudited claim, and it is false. A systematic pass over
every duration quoted in `FORMAL_FOUNDATIONS.md` and `README.md` found the
problem to be structural rather than isolated.

**94a. The scale of it.** Roughly **60 quoted durations**. None is guarded —
`claims_check.py` polices only numbers that are properties of the *tree*, and
Prop. 74e records why: re-deriving a measurement means re-running the proofs.
So no gate has ever checked a single timing in this document.

**94b. At least five further live inferences rest on them.** Beyond the
withdrawn Prop. 81d:

| proposition | the inference | why it does not hold up |
|---|---|---|
| 37c | "splitting pays exactly when members differ in cost", from a **436×** spread | the cheap endpoint is `a_sanity`, **deleted in Wave 591**. The property no longer exists anywhere in the tree, so neither ratio has a reproducible endpoint. |
| 37b/37d-bis | a "~280 s plateau" for any single engine property, and a **1.4× batch overhead** | the plateau interpolates from two samples, one of which is the deleted tautology; the other endpoint is Prop. 34a's 396.1 s, which Prop. 53 could not reproduce even as a *verdict*. |
| 38d | "an 8× cheaper datapath would put `seq 120` within budget — the single largest available gain" | the 8× was **corrected to 1.5×** by Prop. 49d and the item closed by 49e. Prop. 38's strikethrough is scoped to 38a; 38d still reads as a standing recommendation. |
| 54a | "now at 238 s against the original 396 s" | the same withdrawn comparison as 81d, missed by Prop. 91. Annotated in place above. |
| 85d | **"1.58×, +88 s from two properties"**, which drove a shipped design decision and is quoted in the README | see 94d. |

**94c. A citation error, propagated.** Props. 81d and 91c both write "Prop. 55
recorded 22 core properties at `seq 80` in 238 s". **Prop. 55a records 245.1 s.**
The 238 is Prop. 54a's 237.8 s — measured *before* the deep/core split, a
different configuration. The withdrawn inference and the account of its
withdrawal both cite the wrong proposition for their endpoint, and the two
candidate values differ by 3%.

**94d. The most load-bearing unreproduced number is Prop. 85d's.** Its cheap
endpoint is corroborated — Prop. 91a re-measured the same step at 154.5 s
against 153 s. Its expensive endpoint, **241 s, has never been reproduced**: the
single attempt (Prop. 87c) returned an impossible 0.88× and was discarded
because the RTL had been regenerated mid-run. So a causal attribution that moved
properties behind a separate guard, and is quoted verbatim in the README, rests
on one corroborated sample and one whose only re-measurement was thrown away.

It also fails Prop. 87a's own three rules — paired, yes; **repeated, no;
witnessed, no**. The text says "measured on an idle machine" with no load
figure, no range and no input fingerprint. The harness built in Prop. 87 would
have refused to print it.

**94e. Two ratios are unsound in kind, not merely unprovenanced.** Prop. 83e's
**"800× the speed"** divides a completion (1.3 s) by an **abort** (18 minutes),
so the true figure is a lower bound (≥830×) and cannot be a point value.
Prop. 35a's "the parts sum to under 90 seconds; the whole exceeds 240" compares
a sum of completions against a *timeout*. Both commit the completion-versus-budget
confusion that Prop. 34a's own correction note warns about — the campaign
identified the error and then made it twice in ratios.

**94f. And one unreconciled contradiction.** Prop. 36d records the DMA step at
**3.6 s**; Prop. 71c records the same step at **~48 s** after the
one-property-per-invocation split. Nothing in the file reconciles them, and 36d's
figure is the entire cost side of "deeper verification for about thirteen
seconds of CI time".

**94g. What is *not* being claimed.** None of these conclusions is asserted to
be wrong. Prop. 49's decision not to refactor, Prop. 35's split, Prop. 54's
deep/core guard — all may be entirely correct, and several were confirmed by
later verdicts that do not depend on seconds. What is established is narrower
and worse: **their evidence is not re-derivable**, and in five cases an endpoint
describes a property or configuration that no longer exists. The honest position
is the one Prop. 91c took — retire the inference, keep the baseline — applied to
five more cases than Prop. 91 recognised.

**94h. Why this was invisible.** Every one of these numbers is a *fact about a
run*, and this document's gates check facts about the *tree*. Prop. 74e chose
that boundary deliberately and for good reason. The consequence, unstated until
now, is that the single largest category of claim in the campaign's central
document has no gate at all — and the one instrument that could check them was
built four waves ago and has been used on two figures out of sixty.

**94i. The harness's own self-test was ambient-state-dependent.** Running the
full gate suite on a machine loaded by this wave's proof runs, `bench.py
--self-test` **failed** — not on its guard cases, but on its two *pass* cases.
The contention threshold defaults to `0.75 × cores`, and the self-test inherited
it, so "a clean run reports" became a question about how busy the laptop was
rather than about the harness's logic.

Every case except the contention one now runs with the thresholds explicitly
disabled; the contention case sets its own so it still fires. A self-test that
can fail because of what else is running would have flaked in CI and been
re-run until green, which is the worst possible outcome for a guard — it teaches
the reader that red means nothing. Fixing the timing harness's provenance
problem and then giving its self-test an unprovenanced dependency is a small
joke at this campaign's expense, and it is recorded rather than quietly patched.

---

### Prop. 95 — two gates that were not checking what they claimed — `FIXED`

**Gate:** `formal-yosys.yml` → *Every growing register says what bounds it*

The same adversarial sweep that produced Prop. 93 was pointed at the campaign's
*existing* gates rather than at its newest theorem. It found two defects, one of
which had been failing CI since Wave 633.

**95a. The liveness probes for one suite were landing in the wrong module.**
The step *"Module suites are still alive under their assumptions"* injects a
reachability probe by writing it before `src.rindex("endmodule")` — that is,
into whichever module happens to be **last in the file**.

`formal/pipeline_stage2_props.sv` contained exactly one module until Wave 633
appended `ps2_bound` and `ps2_bound_alive`. Since then, all four `ps2_props`
probes have been injected into `ps2_bound_alive`; `prep -top ps2_props` prunes
that module as unused, the probe never enters the cone, and the **unprobed**
suite proves. The step reads `rc == 0` as *"UNREACHABLE — an assumption removed
it"*, so it has been **failing, with a message naming the wrong cause**, for
three waves. Reproduced exactly:

| probe | before the fix | after |
|---|---|---|
| `valid_in` | `rc=0` → reported UNREACHABLE | reachable |
| `valid_out` | `rc=0` → reported UNREACHABLE | reachable |
| `valid_in && first_chunk` | `rc=0` → reported UNREACHABLE | reachable |
| `valid_in && !first_chunk` | `rc=0` → reported UNREACHABLE | reachable |

The injection now targets the **named** top module and errors if that name is
absent. Of the six suites in the table only this one has more than one module,
and the engine probe's file has one — so the blast radius was one suite, but the
mechanism was general: **a file gaining a module silently redirected another
module's probes**, and the defect arrived with a wave that was adding coverage.

**95b. `bound_scan` was crediting assertions as design bounds.** It searched the
whole module text for a comparison against the register. Assertions inside
`` `ifdef T27_FORMAL `` are module text. So `bitnet_engine_top.chunk_addr` was
classified `LOCAL — bounded in-module: 12'd0`, on the strength of
`a_chunk_addr_resets: assert (chunk_addr == 12'd0)`.

An assertion is a claim *about* the design, not a mechanism that constrains it.
Reading one as a bound inverts the gate's entire purpose — and it did so on
**three of the four LOCAL verdicts in the whole bundle**. With formal regions
excluded, `chunk_addr` and `act_wr_word` become FREE and `fv_next_act_addr`
disappears from the audit altogether, correctly: it is a formal-only tracker,
not design logic. **One genuine LOCAL bound remains** in the entire emitted
design, `activation_requant.trit_count`.

**95c. Which surfaced a real unstated contract.** `chunk_addr` advances once per
`layer_valid` and resets only on `layer_start`, so between layer starts it
counts `num_neurons × num_chunks`. It drives `weight_bram.rd_addr`, depth
**4096** — exactly what its 12 bits hold. `num_neurons` is a 16-bit port and
`num_chunks` an 8-bit one, so their product can reach 16.7M. Nothing in the
engine, and nothing in the sequencer, prevents the wrap; the contract lives
entirely in whoever programs the CSRs, and it was written nowhere. Prop. 83's
shape again, and it had been **masked by an assertion's spelling** for four
waves.

`act_wr_word` turned out safe for a different reason worth recording: 65535/27 =
2427 < 4096, so it is bounded **by port width** rather than by contract — a
margin narrower than it looks, and one that widening `num_neurons` past 17 bits
would erase.

**95d. What both defects have in common.** Each gate was reading *text that
looked like the thing it was checking for*. The probe injector matched the last
`endmodule` rather than the right one; the bound scanner matched a comparison
without asking whether it was in the design or in a claim about the design.
Neither failed loudly, and one of them had been red for three waves without the
message pointing anywhere near the cause.

---

### Prop. 96 — `-set-init-zero` is not the reset state — `MEASURED`

**Gate:** `formal-yosys.yml` → *Registers that do not reset to zero are listed*

Every module suite here is proved with `-set-init-zero`, and this campaign has
described that choice, since Prop. 8c, as **"starting from a reachable state"** —
the reason it was preferred over `-tempinduct`, whose unconstrained start refutes
properties that hold everywhere reachable.

It starts from the **zero state**. That equals the reset state only where every
register resets to zero, and **nine registers here do not**:

| module | register | resets to |
|---|---|---|
| `dma_controller` | `state` | `IDLE` |
| `layer_sequencer` | `state` | `IDLE` |
| `multilayer_sequencer` | `state` | `IDLE` |
| `weight_prefetch_ctrl` | `state` | `IDLE` |
| `double_buffer_ctrl` | `use_buffer_a` | `1'b1` |
| `axi_lite_slave` | `s_axi_awready`, `s_axi_wready`, `s_axi_arready` | `1'b1` |
| `activation_requant` | `trit` | `TRIT_Z` |

**96a. This is not an unsoundness, and the distinction matters.** Starting from a
superset of the reachable states can only produce spurious **refutations**, never
spurious proofs: anything that proves under `-set-init-zero` proves for every
reachable state too. **Nothing verified in this campaign is weakened.** The four
FSM rows are additionally harmless in fact, because `IDLE` is encoded `0` in all
four — the zero state and the reset state coincide by *coincidence of the
encoding*, not by construction.

**96b. It is a fragility, and an invisible one.** Renumber an FSM so that any
**decoded** state lands on code 0 — a pure relabelling, since every reference to
`state` in these modules is by name — and the zero state decodes as *active*.
Verified:

| relabelling | references by name / by literal | result |
|---|---|---|
| `dma_controller`: `READ_DATA` → `3'd0`, `IDLE` → `3'd2` | 16 / 0 | `a_rready_implies_burst` **refutes**, alone in its suite |
| `weight_prefetch_ctrl`: swap `IDLE` and `FETCH` | 9 / 0 | `a_rready_implies_active` **refutes**, alone in its suite |

Both are the same shape: `rready` is a combinational decode of the state, so at
the zero state it is high with no burst owed. Two properties, two suites, from a
change that alters nothing in silicon — and the resulting failure would read as
a design defect rather than as a modelling artifact.

**96c. A local instance of this was found years of waves ago and not
generalised.** `double_buffer_props` carries an `fv_started` register whose
comment reads: *"Under `-set-init-zero` every register begins at 0 … the
property refutes on the REAL design."* That is exactly this problem, for exactly
this reason, fixed for exactly one property — and nobody asked how many other
registers reset to something other than zero. The answer was nine.

**96d. A fix that did not work, recorded because the reasoning is the point.**
The obvious repair is to copy `fv_started` to the two affected properties so
they do not assert over the zero state. It **does not help**: with
`-set-init-zero` and `rst_n` never asserted low, the design sits in the decoded
state *indefinitely*, not merely at time zero, so a one-cycle guard changes
nothing. The tested repair was reverted. What is needed is either an explicit
reset assumption or the encoding invariant this gate now records — and the honest
answer is that the encoding invariant is what the design actually relies on.

**96e. What the gate does.** It does not forbid non-zero resets, which would be
absurd — an AXI slave that comes up not-ready is worse than one that does. It
requires each to be **listed** with a reason, so the gap between "the zero state"
and "the reset state" is written down rather than rediscovered by a refutation
in some later wave. Nine registers, nine notes, all in the emitters.

---

### Prop. 97 — a guard whose members had different preconditions — `FIXED`

**Gate:** `formal-yosys.yml` → *The DMA drain is never consumed after it wraps (bounded)*

Prop. 94d named Prop. 85d's **1.58×** the most load-bearing unreproduced number
in the campaign: it moved code behind a separate guard, it is quoted in the
README, and its expensive endpoint had never been reproduced. Attempting the
re-measurement did not produce a number. It produced a defect.

**97a. The `with_drain` arm refutes.** Compiling `-DT27_FORMAL_DRAIN` into the
engine — the exact configuration Prop. 85d measured — now **fails in 11 s**. The
harness declined to report a timing for a command that exited nonzero, which is
the second of its three rules doing precisely its job.

**97b. One property is responsible, and it is the one Prop. 88b predicted.**
Isolating each of the four drain properties in the engine:

| property | engine verdict |
|---|---|
| `a_drain_sane_where_consumed` (DMA) | **REFUTES** |
| `a_drain_residue_nonzero` (DMA) | proves |
| `a_drain_never_underflows` (prefetch) | proves |
| `a_drain_within_request` (prefetch) | proves |

Prop. 88b states it outright: the DMA drain claim is **false in isolation** — an
extra beat past `rlast` wraps the counter — and true only under the AXI
read-slave model that supplies `arlen` compliance. The engine has no slave
model; its `rvalid`/`rlast` are free. The refutation is correct behaviour.

**97c. The guard conflated two kinds of property.** `T27_FORMAL_DRAIN` was
created in Wave 633 for properties that hold unconditionally, and Wave 634 added
an **environment-dependent** property to it without anyone noticing the
categories differed. A define is read as a category by everyone who uses it, and
this one silently meant "drain properties, one of which is false unless you also
supply an AXI slave model". The DMA properties now sit behind
`T27_FORMAL_DRAIN_AXI`, whose name states the precondition. Verified: the DMA
step still proves at `seq 24`, and the engine under `-DT27_FORMAL_DRAIN` proves
again.

**97d. And the number is permanently unreproducible.** Prop. 85d compared the
engine with and without the Wave-633 drain set. That set no longer exists — it
gained a property in Wave 634 that cannot be compiled into the engine at all.
The 1.58× therefore joins Prop. 91c's category: not shown to be wrong, but
**incapable of being checked**, because the configuration it describes is gone.
The design decision it justified is independently sound — the properties are
proved unbounded at module level, so re-proving them in the engine to depth 40
buys strictly less — and that argument never needed the timing.

**97e. The measurement still has not been made.** The post-split re-run showed
both arms proving, but the harness **refused to print a ratio**: load 8.4 on 8
cores with a competing prover, because a gate-audit workflow was running
concurrently. Recorded as unmeasured rather than reported with a caveat. That is
the third refusal this harness has produced in three waves — a failing command,
moved inputs, and now a busy machine — and in each case the number it declined
to print would have been wrong.

---

### Prop. 98 — four defects in two gates, found by attacking them — `FIXED`

**Gate:** `formal-yosys.yml` → *No declaration is narrower than the range it carries*

An adversarial audit of the campaign's remaining gates — one agent per gate,
every finding then independently reproduced by a second — confirmed **four**
defects. All four are the same family as Prop. 95: a gate matching text that
merely *looks like* what it means to check.

**98a. `phantom_scan` missed every multi-bit undriven wire.** The gate exists for
exactly one defect — Prop. 62, where a property proved against an undriven wire
for four waves. Yosys words that warning differently by width:

```
1-bit    Warning: Wire zz.\fv_ghost is used but has no driver.
n-bit    Warning: Wire zz.\fv_ghost [3] is used but has no driver.
```

The pattern ran `([\w.\\]+) is used`, and that character class cannot cross the
space or the brackets. **Every undriven wire wider than one bit went unmatched.**

The self-test could not see the hole because it never opened one: all four of its
injections — a hierarchical reference, a misspelled name, a renamed port — are
identifiers yosys implicitly declares as a *single bit*. The gate was tested
only in the form that worked. Prop. 62's own case was one bit, which is why it
looked correct. Two width cases are now permanent self-tests.

**98b. `width_scan` deduped reductions by target name, dropping 40% of its
subject.** `assign l2[0]`, `l2[1]`, `l2[2]` all yield target `l2`, and the
dedup set was consulted *before* the check ran — so only the first was ever
examined. **2 of the 5 checkable reductions in the bundle were never looked at,
both inside `adder_tree_27`, the module the gate was written for.** Worse, that
same set was the coverage counter, so the summary reported distinct *names* and
read as full coverage. Every reduction is now checked (3 → **5**); only the
error message is deduped.

**98c. A same-line range comment deleted its own declaration from the gate's
view.** `parse()` tested for a range comment first and `continue`d, so a line
carrying *both* a comment and a declaration was consumed as a comment only. The
name entered neither dictionary and was invisible to every check. Moving an
existing comment to **trail** its declaration — a formatting change — took a
provably broken adder tree from exit 1 to exit 0.

**98d. And the unannotated-operand fallback was the unsound rule the file's own
docstring forbids.** For an operand with no documented range, the gate fell back
to its declared *width* — precisely the worst-case-by-width reasoning Prop. 82b
established is wrong for ternary, since `val` is `signed [1:0]` but holds only
{−1,0,+1}. Deleting one of three range comments therefore produced a **false
finding against correct RTL**. An unannotated operand now makes a reduction
*uncheckable* rather than checkable-by-a-wrong-rule, and the count of such
skips is printed, because that is a real loss of coverage and absorbing it
silently is how the first three defects survived.

**98e. What the two gates have in common with Prop. 95's two.** Four gates, four
instances of matching a *form* rather than a *fact*: a warning's phrasing, an
identifier's name, a comment's position, a width standing in for a range. Every
one passed its own self-test, because a self-test written by the author of a
gate exercises the cases the author had in mind.

---

### Prop. 99 — the drain properties make the engine proof *faster* — `MEASURED`

**Gate:** `formal-yosys.yml` → *Benchmark harness self-test*

With the guard split of Prop. 97 in place, the question Prop. 85d asked can be
put again — not as a reproduction, which Prop. 97d showed is impossible, but as
a fresh measurement of the configuration that exists today.

Three runs per arm, alternating, input fingerprint identical, one prover (the
command under test):

| arm | median s | observed range |
|---|---|---|
| engine without the drain properties | 146.6 | [146.5, 148.7] |
| engine **with** them | **120.4** | [117.9, 120.8] |

**0.82×, 26 s faster.** Ranges disjoint. Adding two assertions makes the proof
cheaper — unsurprising once stated: an assertion that is easy to discharge acts
as a lemma and prunes the solver's search.

**99a. This is not a reproduction of Prop. 85d, and must not be read as one.**
That comparison included the DMA drain property, which cannot be compiled into
the engine at all (Prop. 97b). The configurations differ, so the 1.58× remains
what Prop. 97d called it: uncheckable, not refuted.

**99b. It does remove the stated reason for the guard split.** Wave 633 moved
these properties behind their own define *because they were measured as costly*.
On today's configuration they are not costly; they are free, and slightly
better than free. The split remains correct for the **other** reason given at
the time — induction proves them for every request length while the engine
re-proves them only to depth 40, so including them buys strictly less — and that
argument never depended on a timing. A decision whose stated justification has
evaporated but which is still right is worth noticing, because next time the
justification might have been the only one.

**99c. An uncomfortable footnote.** Prop. 87c recorded an *implausible* 0.88×
and rejected it, correctly, because the RTL had been regenerated mid-run. The
clean figure is **0.82×** — the rejected number was, in direction and roughly in
magnitude, right. That does not make rejecting it wrong: a measurement whose
inputs moved underneath it is unusable as evidence *whatever value it lands on*.
Being accidentally right is not a form of being right.

---

### Prop. 100 — the audit found six; I had fixed four — `FIXED`

**Gate:** `formal-yosys.yml` → *No declaration is narrower than the range it carries*

Prop. 98 recorded four confirmed defects and fixed them. The full audit report
arrived afterwards and contained **six** for `width_scan` alone. The two I had
not seen were both verified, and both **survived the Wave 637c fixes** — checked
before assuming otherwise:

| defect | after Prop. 98 |
|---|---|
| a constant addend makes the check decline silently | **still missed** |
| subtraction is never checked at all | **still missed** |

**100a. Two whole expression forms declined in silence.**
`assign l2[0] = l1[0] + l1[1] + l1[2] + 5'sd9;` overflows the declared
[−16, +15] and the gate printed *"0 carrying less"*, exit 0 — because a literal
is not an identifier in `rng`, the operand count mismatched, and the loop
`continue`d. `top_level_plus` counted only `+`, so **every subtraction** was
likewise outside the matcher: `l1[0] - l1[1] - … - l1[5]` reaches [−18, +18]
against the same declaration, silently.

Both are ordinary Verilog. Declining them is defensible; declining them without
saying so is the failure this campaign is about, hiding inside the gate written
to catch arithmetic that does not fit. The reduction loop now splits an
expression into **signed terms** at bracket depth zero, resolves each as an
operand *or* a sized literal, negates ranges after `-`, and counts anything
unresolvable as **uncheckable** rather than skipping it.

**100b. And the guard tripped only at exactly zero.** That is why three separate
defects could hide behind it: losing one of three annotations, or dropping a
declaration out of the parser's view, left a summary line indistinguishable from
a healthy one. It is now a **floor** — 16 declarations, 3 annotated, 5
reductions, the shipped tree's actual numbers — so a drop is loud and must be
raised deliberately if the emitters change.

**100c. The lesson is about my own reporting, not the gate.** I read four
findings off a truncated notification, fixed them, wrote a proposition, filed an
issue and pushed — while two more sat in the untruncated result on disk. The
diagnostics line said exactly where the full text was. **A summary of an
adversarial review is not the review**, and the failure mode is precisely the one
the review exists to prevent: acting on the part that was easy to see.

Sequence, recorded plainly: Prop. 98 claims four defects fixed. That claim was
true and incomplete. Six were found; six are now fixed.

---

### Prop. 101 — every decline, counted — `FIXED`

**Gate:** `formal-yosys.yml` → *Every proposition carries the gate that keeps it true*

Prop. 100's mechanism is generic: a matcher `continue`s on a form it does not
handle, while the coverage figure still reads full. Two whole expression classes
hid in `width_scan` that way. So the pattern was swept across all ten gates —
every bare `continue`, asked whether it means *"not my subject"* or *"my subject,
which I could not check"*. The second kind must be counted.

**101a. Eight of the ten were clean.** `bound_scan`'s three continues are
control flow and precedence, and its classification is *total* — every
self-incrementing register receives exactly one of LOCAL / CONTRACT / FREE /
DRAIN. `phantom_scan` and `init_zero_scan` have none. A negative result, recorded
so the sweep is not repeated.

**101b. `doc_gate` silently skipped any fence containing `<foo>`.** The rule is
that every ```bash fence must run something, with a template exempted as
*"not a command"*. That exemption was invisible: a reproduce command which
happened to contain angle-bracketed lowercase text left the check entirely with
nothing in the summary to say so. There is **1** today — the `<bundle>` /
`<harness>` template at `FORMAL_FOUNDATIONS.md:443`, legitimately a template —
and now it is named, so the count cannot grow quietly.

**101c. `absence_sweep` silently dropped its `BUILDERS` steps.** The sweep
reports steps, exemptions and its own recursion, but the builder exclusions were
uncounted: **6** of them. A checking step named like a builder would have
vanished from the sweep with the summary unchanged. The same lie the file's own
comment at line 141 warns against — *"reporting the list size says '1 exempt' on
a run where nothing was exempted"* — committed one exclusion class over.

**101d. Changing the signature caught a coupling.** `collect()` returning a
third value broke `claims_check`, which imports it to derive a gated README
number. That import was deliberate (Prop. 84's "derive, never re-count"), and it
means a change to the sweep's interface propagates into the gate that polices
prose. Both callers updated; noted because the coupling is invisible from either
file alone.

**The rule, stated for reuse:** a gate's summary must report what it *did not*
check as prominently as what it did. "0 problems" over an unstated number of
declines is the same sentence as "0 problems" over none, and this campaign has
now found four defects that lived in exactly that gap.

---

### Prop. 102 — three defects in code written two days earlier — `FIXED`

**Gate:** `formal-yosys.yml` → *No declaration is narrower than the range it carries*

The round-two audit — `orphan_scan`, never reviewed, plus the four gates changed
in Waves 637–638 — returned **25 verified findings**. Three are confirmed here,
all in code written within the previous 48 hours, and all found by asking the
gate the question it was built to ask about the design.

**102a. `4'b101` was read as one hundred and one.** `term_range()`'s literal
pattern captured the digits and evaluated every sized literal as **decimal**,
ignoring the base that is the entire point of the notation. Five became 101 — a
20× error, in the direction of a *false finding* — while `8'hff` and `3'o7`
matched nothing at all. Verified across all four Verilog bases; fixed and kept
as a self-test row. This is "match a form, not a fact" (Prop. 98e) committed
inside the gate written to catch arithmetic that does not fit.

**102b. `strip_formal` deleted real design.** Wave 636b added it so `bound_scan`
would stop crediting assertions as bounds. It removed *regions* rather than
resolving guards, so two constructs vanished: the body of `` `ifndef
T27_FORMAL `` — which is precisely the code that compiles when the define is
absent — and the `` `else `` branch of any `` `ifdef T27_FORMAL ``. Both are
design. It now resolves each guard as T27_FORMAL-undefined: drop the `ifdef`
branch, keep the `else`; keep the `ifndef` branch, drop its `else`. Guards on
other symbols are untouched.

The direction of the error was safe — deleting design can only push a register
toward FREE, which *demands a note* rather than hiding anything — and the
shipped tree's 16 verdicts are unchanged. It was still wrong, and it hid
whatever bound lived in those branches.

**102c. `orphan_scan` counted assertions inside comments.** A comment
*discussing* an assertion made a module look DIRECT. This is the **identical**
defect fixed in `claims_check` one wave earlier (Prop. 95), with the identical
regex, in a sibling file — and nobody checked whether the same pattern elsewhere
had the same problem. It did.

**102d. One claim did not reproduce, recorded as such.** The audit reported that
`term_range()` matches a *prefix* identifier and discards the rest of the token.
It does not: `l1x[0]` against a range table containing `l1` returns `None`
correctly, because the captured name is `l1x` and no such key exists. Reported
as verified; not reproducible. Negative results are recorded here so the next
wave does not re-litigate them.

---

### Prop. 103 — the defect taxonomy, with counts — `MEASURED`

**Gate:** `formal-yosys.yml` → *Every proposition carries the gate that keeps it true*

Twenty-odd waves of instrument auditing have produced enough instances to state
the finding as a class rather than a list. Every confirmed defect in this
campaign's *gates* — not in the design, in the things that check the design —
falls into one of five shapes. The counts are of confirmed, independently
reproduced instances.

| # | shape | instances | example |
|---|---|---|---|
| 1 | **Matching a form, not a fact** | 9 | a warning's phrasing (1-bit vs n-bit "no driver"); an identifier's *name* where its *value* was meant; a literal's digits without its base |
| 2 | **A decline that is not counted** | 4 | subtraction and constant addends silently skipped while coverage read full |
| 3 | **Reading a claim as the design** | 3 | assertions inside `` `ifdef T27_FORMAL `` credited as bounds; assertion labels counted inside comments |
| 4 | **Targeting by position, not by name** | 2 | a probe injected before the *last* `endmodule`; a range comment bound to the *next* declaration |
| 5 | **A guard that trips only at zero** | 3 | losing 1 of 3 annotations leaves a summary identical to a healthy one |

**103a. Three structural regularities.** First, **the self-test never catches
these**, in any instance. A self-test is written by the gate's author from the
same mental model that produced the defect, so its cases sit at one point on
whichever axis matters — every `phantom_scan` injection was a one-bit signal,
every `width_scan` reduction a bare identifier sum.

Second, **defects cluster in the newest code**. Of the 25 findings in the
round-two audit, the three confirmed were all in code less than 48 hours old.
The gates that had survived twenty waves were comparatively clean; `bound_scan`
proved *total* over its subject.

Third, and least comfortable: **the same defect recurs in sibling files**.
Prop. 95 fixed comment-counting in `claims_check`; the identical regex in
`orphan_scan` had the identical defect and went another wave undetected because
fixing an instance was not followed by grepping for the pattern.

> **FALSIFIED by Prop. 106.** The prediction below did not hold. An audit of
> the five never-reviewed gates produced defects fitting **no** shape in the
> table, and the table is corrected there rather than defended here.

**103b. A testable prediction, stated before the next audit.** If shapes 1–5 are
exhaustive, a further audit should find defects only in these categories, and
predominantly in gates modified since the last audit. If it finds a *sixth*
shape, the taxonomy is incomplete and this proposition is the thing to correct.
That is falsifiable, and it is recorded here so the next wave can check it
rather than re-derive it.

**103c. The methodological result, which is the transferable part.** A gate that
checks a property of a codebase is itself a program with no gate above it. This
campaign's answer — adversarial review by an agent instructed to *refute* rather
than confirm, with every finding independently reproduced before it is believed
— found **10 defects in six gates in two days**, after those gates had passed
their own self-tests for up to twenty waves. The self-tests were not bad. They
were written by the person who wrote the gate.

---

### Prop. 104 — the orphan check never checked that anything runs — `FIXED`

**Gate:** `formal-yosys.yml` → *Every property file is run by some workflow*

`orphan_scan` was written in Wave 618 after an accident revealed eight property
files that CI never ran (Prop. 69). Its stated purpose is to *"cross-reference
every property file against every workflow and error if nothing runs it"*. It
asked a different question: **does this filename appear anywhere in the workflow
file?**

**104a. Four ways to be "referenced" without being run.** Each verified by
injection, each diff-confirmed before the run:

| the only mention of the file | old verdict |
|---|---|
| a `#` comment: *"TODO: we should someday prove formal/ghost_props.sv"* | referenced |
| a step carrying `if: false` | referenced |
| `grep -c assert formal/ghost_props.sv` — runs, reads it, proves nothing | referenced |
| a workflow triggered `on: [release]`, which no push or PR fires | referenced, and **not even a weekly warning** |

The injected file's property was `assert (1'b0)` — provably false, confirmed
refuting under yosys — and the gate reported a clean tree in all four cases.

**104b. The hazard was live, on the file the gate exists because of.**
`formal-yosys.yml` already carries **two retrospective `#` comments naming
`zero_size_props.sv`** — the very file whose ungated properties Prop. 69 was
about. Deleting only its two *executable* references leaves the summary
**byte-identical to a healthy tree**: `15 property files, 26 workflows, 0
orphaned`, exit 0, no errors, no warnings. The comments narrating Wave 617's
defect would have concealed its recurrence. One deletion deep, and the gate is
the thing that hides it.

**104c. The fix asks the intended question.** The searched text is now the
`run:` bodies of *reachable* steps — `if: false` skipped at job and step level,
`#` comments stripped from inside the bodies — and a body counts only if it also
invokes something that could prove or load the file (`yosys`, `read_verilog`, or
`open(`, since several steps hand a property file to a python harness first).
All four injections are now caught and kept as self-tests.

**104d. A filename is a name, not a substring.** `f.name in text` credited a
hypothetical `formal/props.sv` to **eight** unrelated suites, because the
convention in `formal/` is `<thing>_props.sv` and every one of them contains the
substring. Now matched with delimiters on both sides.

**104e. The fix failed loudly first, which is the right way round.** The initial
delimiter excluded `/` — but references are written `formal/<name>.sv`, so every
reference stopped matching and the gate reported **all 15 files orphaned at
once**. A gate whose repair is wrong should say so at the top of its voice
rather than drift by one.

**104f. Prop. 103's third regularity, again.** The comment-counting defect this
gate also had is the *same defect, on the same regex*, that `claims_check` was
fixed for one wave earlier (Prop. 95). It was found here only because the audit
looked. Fixing an instance is still not fixing the pattern.

---

### Prop. 105 — grepping for the shape, not waiting for the audit — `FIXED`

**Gate:** `formal-yosys.yml` → *No property references a signal that does not exist*

Prop. 103's third regularity says the same defect recurs in sibling files, and it
had been demonstrated twice by an *audit noticing*. That is the slow way. Each of
the five shapes has a textual signature, so the tree was swept for them directly.

**105a. A third instance of the comment-counting defect.** `scale_probe.py`
enumerates assertion labels with `a_[a-z0-9_]+: assert \(` over **raw** source,
and the file it reads is `build/rtl/bitnet_engine_top.sv` — the exact file
carrying the Wave-636b comment that *quotes* an assertion by name. A comment
naming a label with no corresponding assertion puts a phantom property into the
probe list and produces a timing for nothing.

This is the same defect, on the same regex, that `claims_check` was fixed for in
Wave 636b and `orphan_scan` in Wave 639b. **Three files, three waves, one
pattern** — and this one was found in seconds by grep rather than by a
multi-agent audit.

**105b. A latent instance of the position-targeting defect.**
`phantom_scan`'s self-test injected before `src.rindex("endmodule")` — precisely
the construct that redirected four liveness probes into the wrong module in
Prop. 95a, once a wave appended modules to a property file. Its victim,
`dma_controller_props.sv`, has one module today, so it worked. That is exactly
how the same defect stayed live in a sibling twice: *it works until a file grows*.
A self-test that silently begins injecting into the wrong module stops testing
without saying so, and no gate stands above a self-test. Now targeted by name,
with a missing module reported as an error.

**105c. The sweep's yield, honestly.** Six signatures across 15 gate files
produced 33 candidate hits, of which **two** were real. Most "guard trips only at
zero" matches are ordinary `if not x:` idioms, and most "tool output matched by
one phrasing" hits are legitimate single-purpose patterns. A grep for a defect
shape is a *lead generator*, not a verdict — every candidate still needed reading
and, for these two, tracing to the file each actually consumes.

**105d. What this suggests about method.** The audits are expensive: two of them
cost roughly four million subagent tokens and five hours, and produced 33
findings of which a dozen were confirmed. The grep cost a minute and found two
the audits had not reached. They are complementary — an audit discovers *new*
shapes, a grep propagates *known* ones — and the cheap one should run first,
immediately after any fix, rather than waiting for the next review.

---

### Prop. 106 — the taxonomy was falsified, as designed — `MEASURED`

**Gate:** `formal-yosys.yml` → *Benchmark harness self-test*

Prop. 103b staked a prediction: a further audit would find defects only in
shapes 1–5, and *"a sixth shape means the taxonomy is incomplete and this
proposition is the thing to correct."* Five never-audited gates were then
attacked, with agents told explicitly that finding a sixth shape was **more
valuable** than confirming the five.

**They found two, and the prediction is withdrawn.** This is the result the
prediction was for: it was written to be falsifiable, and it was falsified in
one round.

**106a. Shape 6 — sampling a time-varying property at its boundaries.**
Verified by construction in `bench.py`, twice over:

- `run_once` sampled competing provers and load average **once before** the
  subprocess and **once after**. A prover that started *and finished* inside the
  run was invisible to both. The contention guard — the entire reason the
  harness exists — was a boundary check on a continuous quantity.
- The input fingerprint was taken once before *all* repeats and once after. A
  file changed between repeat 1 and repeat 2 and **reverted** before the end
  yields identical digests. That is precisely the contamination Prop. 87c added
  the guard for, undetectable whenever it reverts.

Neither fits shapes 1–5. The check observes the right thing, in the right units,
with a correct threshold; it is blind only to the *interval between its
observations*. Fixed: a background sampler polls every 250 ms and reports the
peak, and the fingerprint is taken around every run rather than around the
sequence. Both kept as self-tests, including a file that flips and reverts.

**106b. Shape 7 — over-detection.** All of shapes 1–6 describe a gate that
**fails to fire when it should**. The audit reported a gate failing a *correct*
artifact, which is the mirror image and cannot be an instance of any of them.

This shape already had an instance and it was mis-filed: Prop. 98d recorded
`width_scan` producing a **false finding against correct RTL** when one comment
was reworded, and it was classified under shape 1. The behaviour is real, the
classification was wrong, and the taxonomy had no box for it because every box
was about silence.

**106c. The corrected table.**

| # | shape | what fails |
|---|---|---|
| 1 | matching a form, not a fact | detection |
| 2 | a decline that is not counted | detection |
| 3 | reading a claim as the design | detection |
| 4 | targeting by position, not by name | detection |
| 5 | a guard that trips only at zero | detection |
| **6** | **sampling a time-varying property at its boundaries** | **detection, in the interval between observations** |
| **7** | **over-detection: failing a correct artifact** | **the opposite of detection** |

**106d. Five other claimed new shapes were not new.** Two described a
perturbation that fails to reach every declaration and a theorem absent from an
expected-results table — both already recorded as Prop. 93b, and both instances
of shape 2. One described a hardcoded port list, which is shape 2's enumeration
form. Being strict about this matters: a taxonomy that absorbs every finding
predicts nothing, and one that admits every claimed novelty is not a taxonomy.

**106e. What the falsification is worth.** A prediction that survives tells you
little; the five shapes had already been fitted to the data they came from.
Stating the boundary *before* looking, then having it broken in the first round,
is the only part of this that was ever evidence. The corrected table now carries
the same obligation: if an eighth shape appears, **this** proposition is what to
correct.

---

### Prop. 107 — a quarter of the gate citations named steps that did not exist — `FIXED`

**Gate:** `formal-yosys.yml` → *Every proposition carries the gate that keeps it true*

The rule this file is built on is that every proposition names the CI step that
keeps it true. `doc_gate` enforced it as: **is a `**Gate:**` line present?** It
never opened `.github/`.

**107a. 33 of 106 named a step no workflow defines.** Independently verified by
extracting every `- name:` from every workflow and resolving each citation
against it. The clusters:

| count | cited step | reality |
|---|---|---|
| 16 | *Prove bitnet_engine_top integration properties* | removed two waves earlier; `git log -S` dates it |
| 7 | *Prop. 39e is still open (must refute)* | the step is *No property is gated as an expected refutation* |
| 2 | *Zero-sized requests complete without pretending* | the step is *Prove zero-size properties* |
| 1 | *Prove integration properties (core 22, deep bound)* | pre-split name; it is core **24** |
| 1 | *Every proposition names its gate, every block runs* | **doc_gate's own proposition**, citing a step that does not exist |

Twenty-five lines repointed. The last one is the sharpest: the proposition
asserting that every proposition names a real gate named a gate that was not
real, and the gate enforcing it could not tell.

**107b. The check now resolves.** Every `**Gate:**` line's italicised step name
is matched against the set of `- name:` entries across all workflows, and an
unresolvable name fails the build. The count is printed — *"106/106 named steps
exist, 8 in a format this check cannot resolve"* — so the unresolvable
remainder is visible rather than absorbed, per Prop. 101.

**107c. Adding a shape-3 check committed a shape-7 defect, twice.** The first
version read `**prove**` out of a *bold* span and reported a correct line as
citing a nonexistent step; the second treated parentheticals, `none.` and
ellipses as step names. Both are over-detection — failing a correct artifact —
the shape named one proposition earlier in Prop. 106b. Writing a check for one
failure mode is an excellent way to commit its opposite.

**107d. And a third, in the guard on the guard.** The "resolution check would
pass on nothing" guard resolved workflows relative to the *document*, so the
self-test — which copies the doc to a temp directory — found no workflows and
correctly failed its own unmutated case. Anchored to the gate file instead. The
guard was right; its notion of "where the repository is" was not.

---

### Prop. 108 — the mirror compared nothing, if you asked it positionally — `FIXED`

**Gate:** `formal-yosys.yml` → *Prove the trit algebra (exhaustive)*

`mirror_check` holds Prop. 92's composition proof to the real circuit: the
abstraction duplicates `trit3_add`'s wiring, so something mechanical must pin the
copy to the original. Two of the three criticals reported against it reproduce.

**108a. Positional instantiation yields zero extracted connections.** `CONN`
matches only `.name(net)` form. `trit_full_adder fa0 (a[1:0], b[1:0], TRIT_Z,
sum[1:0], c0);` — perfectly legal Verilog — produces an **empty** connection map,
and an empty map compares equal to any other empty map. The gate would print
*"3 concrete stages vs 3 abstract, 0 disagreements"* while having read nothing
from either.

Shape 2, a decline that is not counted, inside the gate that exists to stop a
proof drifting from its subject. A stage with no named connections is now an
error naming the stages involved, because the honest report is *"I cannot
compare these"*, not *"these agree"*.

**108b. Localparam chains resolved exactly one level.** The resolver added in
Wave 636b — itself the fix for comparing names instead of values — turned
`localparam TRIT_Z = ZZ;` into the string `"ZZ"`. A name standing in for a value,
which is the same shape the resolver was written to eliminate, one indirection
further out. Now a bounded fixed point, so a cyclic definition terminates rather
than hanging.

**108c. One reported critical did not reproduce.** Instances inside `/* */`
block comments were said to be counted. They are not: the shipped parser found
exactly the one live instance in a two-instance test, the commented one
excluded. Recorded so it is not re-litigated.

**108d. Both fixes were verified against the resolver, not only end-to-end.**
The chained-localparam regression was first written as a full RTL injection and
failed for a reason that was the *injection's*, not the gate's. Testing
`params()` directly proves the property the fix is about; an end-to-end
injection would have proved something about anchor text. When a fix is to one
function, test that function.

---

### Prop. 109 — the sweep was starving the instruments, not the subjects — `FIXED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

`absence_sweep` is the load-bearing gate: it certifies that all forty CI checking
steps **fail when starved**, which is what licenses reading any of their greens
as evidence. It was the only gate never audited — an agent assigned to it stalled
in two consecutive rounds.

**109a. It moved the gate scripts aside along with the design.** The sweep
relocated `build/rtl/` *and the whole of* `formal/` — which holds all ten gate
**scripts**. Every python step then failed with
`No such file or directory: formal/<gate>.py`, and the sweep recorded *"fails,
correct"*.

For roughly a quarter of the swept steps, the only thing established was that
**deleting a script breaks the step that runs it**. That is circular, and it
proves nothing whatever about whether the gate reads its subject. The claim
*"0 passing on nothing"* was, for those steps, not evidence.

Now `build/rtl/` goes entirely and only the non-`.py` files leave `formal/`, so
the instruments survive while their subjects are gone.

**109b. The fix immediately exposed two steps that pass on nothing.** With the
scripts present, *Benchmark harness self-test* and *Every proposition carries the
gate that keeps it true* both exit 0 on an empty tree — because their subjects
are not the RTL. `bench --self-test` exercises its own guards with synthetic
commands; `doc_gate` reads markdown and workflow step names.

Both are now EXEMPT **with a written reason and an internal absence case of
their own**, because demanding that a documentation gate fail when the RTL is
missing would be shape 7 — failing a correct artifact. The sweep had silently
assumed every step's subject is the design.

**109c. What this costs and what it buys.** The swept count drops from 39 to 37
and the exemptions rise from 1 to 3. That is a smaller number describing a real
guarantee, replacing a larger one describing a circular test — the trade this
campaign has made repeatedly, and the reason Prop. 101 requires exemptions to be
counted out loud.

**109d. The audit reported this as a new shape; it is not.** It was filed as
*"the sweep runs only the negative arm of a two-arm control"* — true, and a real
observation — but the mechanism is shape 2: the sweep declined to distinguish
*"failed because the subject is missing"* from *"failed because the script is
missing"*, and counted both as success without saying so. Prop. 106's table
stands.

---

### Prop. 110 — three orthogonal ways a gate is wrong — `MEASURED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 103 listed five defect shapes; Prop. 106 added a sixth column for
over-detection and left it as "shape 7". With roughly thirty-five confirmed
instances the list resolves into something with structure, and the structure
makes a prediction the list does not.

**110a. The framing.** A gate `G` is a decision procedure over artifacts. Let
`P` be the property it is *documented* to enforce. Two standard notions apply,
and one that is not standard:

> **Sound** — `G(a) = pass ⟹ P(a)`. No artifact violating `P` gets through.
>
> **Complete** — `P(a) ⟹ G(a) = pass`. No artifact satisfying `P` is failed.
>
> **Faithful** — the property `G` actually decides *is* `P`, rather than some
> `P′` that resembles it.

Every confirmed defect in this campaign is a failure of exactly one of the three,
and the three are independent: a gate can be sound and complete for `P′` while
`P′` has nothing to do with what its docstring claims.

**110b. The census, by category.**

| category | what fails | instances | shapes |
|---|---|---|---|
| **Unsound** | passes an artifact violating `P` | ~28 | 1–5 |
| **Incomplete** | fails an artifact satisfying `P` | 3 | 7 |
| **Unfaithful** | decides `P′`, claims `P` | 4 | — |

The five shapes of Prop. 103 are *all* mechanisms of unsoundness. That is not a
property of gates; it is a property of **how this campaign has been looking**.
Every audit so far was instructed to find gates that pass when they should fail,
so the taxonomy it produced enumerates the ways that happens and nothing else.

**110c. The unfaithful category is the one adversarial testing cannot find.**
Props. 73, 85f, 91c and the Wave-643 sweep are its four members. In each, the
instrument was *correct* — it decided its `P′` soundly and completely — and the
sentence describing it named a different `P`:

- Prop. 73: the matrix measured gaps against **one wrapper**; the caption said
  gaps in **the module**.
- Prop. 85f: the timings were real; they were measured under **contention** and
  described as a clean comparison.
- Prop. 91c: an inference whose endpoint described a **22-property**
  configuration that no longer existed.
- Prop. 109: the sweep soundly decided *"does this step fail when its script is
  deleted"* while claiming *"does this step fail when the design is absent"*.

No amount of injecting defects into the subject finds any of these, because the
gate answers correctly every time. **Only reading the claim against the
implementation finds them** — which is why Prop. 73's error stood for twelve
waves while the harness ran green throughout.

**110d. What this predicts.** Three things, all falsifiable:

1. An audit instructed to hunt **over-detection** will find shapes that are not
   1–5, because 1–5 are unsoundness mechanisms by construction. Finding them
   does *not* falsify Prop. 103; it confirms that Prop. 103 described one
   category rather than the field.
2. The unfaithful category will keep appearing at roughly its historic rate
   (~1 per 8 waves) and will keep being found by *reading*, not by testing.
3. If a future defect fits none of the three categories, this proposition is
   wrong. A gate is a decision procedure; sound, complete and faithful are
   exhaustive over "the answer is wrong" and "the question is wrong". A
   counterexample would have to be a fourth thing to be wrong about.

**110e. The methodological consequence.** Adversarial agent review — the
technique that found ~28 defects in ten days — is a **soundness** instrument. It
is nearly blind to the other two categories, and the campaign's own record shows
it: every unfaithful defect was caught by a human or a model *re-reading a
claim*, never by an injection. A verification effort that runs only adversarial
review will drive unsoundness toward zero and leave its captions untouched.

---

### Prop. 111 — the first instrument for the unfaithful category, and what it cannot do — `MEASURED`

**Gate:** `formal-yosys.yml` → *No gate mutates a path its docstring never names*

Prop. 110 sorted every confirmed defect into three independent categories and
observed that **unfaithful** — soundly deciding some `P′` while claiming `P` —
has no instrument, and cannot be found by adversarial testing because the gate
answers correctly every time. `formal/faith_check.py` is a first attempt, and
the interesting part of this proposition is the limit rather than the check.

**111a. What it does.** Faithfulness in general is undecidable; a projection of
it is not. The check compares one concrete thing the docstring claims against
one concrete thing the code does: **which paths the gate MUTATES**. Every path
passed to a filesystem-changing call — `move`, `rmtree`, `unlink`, `write_text`,
`open(…, "w")` — must be named in the module docstring. 17 gates, 10 mutated
paths resolved, 0 undeclared.

**111b. Reads are deliberately excluded, and the first version proved why.** It
demanded that every path a gate *reads* appear verbatim in its docstring, and
produced **24 findings on a clean tree** — because a docstring legitimately says
"reads the emitted RTL" where the code says `build/rtl`. Prose is not a path
literal. That is over-detection (shape 7) in the instrument built to find
unfaithfulness, one hour after the category was defined. What is surprising is
not what a gate reads but what it *changes*.

**111c. The check would NOT have caught Prop. 109, and the first draft said it
would.** A retroactive test was written to demonstrate the opposite. It briefly
appeared to pass — because the reconstruction of the pre-fix file had mangled
the docstring it was supposed to preserve. Repairing the reconstruction turned
the result negative and it stayed negative.

The reason is instructive. Prop. 109's defect was `absence_sweep` moving the
whole of `formal/` aside, gate scripts included. But its docstring **did** say it
empties `formal/`. The path was declared; what went unnoticed was the
*consequence* — that emptying the directory also removes the instruments. **No
path-level check can see that.** The surviving claim is narrower than the one
first written: this catches an *undeclared* path, not a *misunderstood* one.

**111d. Three over-detections in one file in one wave.** The reads version (24
findings), the function-scope widening (11 findings, all self-tests writing to
temp trees), and a docstring naming `build/rtl` failing to cover a mutation
reported as `build`. Each was fixed by narrowing: mutations only, self-test
functions exempt, paths closed under parent prefixes. Prop. 110's prediction
that an instrument pointed at a new category would meet shape 7 repeatedly held
within a single file.

**111e. Its own absence case, since the sweep cannot supply one.**
`faith_check`'s subject is `formal/*.py` — the gate scripts — which the sweep now
deliberately *preserves* (Prop. 109). Starving `build/rtl` cannot make it fail,
and making it fail would mean deleting the instruments the sweep was just fixed
to keep. It is therefore EXEMPT with a written reason, and carries a floor on
resolved mutated paths so an extractor that stops seeing anything fails loudly
rather than reporting a clean tree.

**111f. What remains unmeasured.** The unfaithful category has four recorded
members; this instrument addresses the *path* projection of one of them. The
other three — a caption naming a module where the data described a wrapper, a
timing measured under contention, an inference whose endpoint no longer existed
— are not path-shaped, and nothing here would find them. The category is now
instrumented, not covered.

---

### Prop. 112 — a gated claim and its ungated synonym, in the same document — `FIXED`

**Gate:** `formal-yosys.yml` → *Numbers in the documentation match the tree*

Prop. 111 instrumented the *path* projection of the unfaithful category and
noted the other three recorded members are not path-shaped. This is the second
projection, and it found a live instance.

**112a. The finding.** README stated, four hundred words apart:

> "runs **all 37 checking steps** of both formal workflows"  — gated, and correct
>
> "certifying that all **forty** CI steps fail when starved"  — ungated, and wrong

Both describe the same sweep, which walks 41 steps and checks 37. The first is
matched by a `claims_check` pattern and has been kept correct through four
count changes. The second was invisible, because a gate that matches one
phrasing sees only that phrasing. This is Prop. 73's shape at its smallest: the
data was right and a caption elsewhere described a different set.

**112b. Registering the synonym is the wrong fix, and the gate said so.** A
`CLAIMS` entry demands its pattern *match* — so registering `"all N CI steps"`
would forbid ever rephrasing the sentence. Removing the numeric phrasing
immediately tripped the UNMET guard added in Wave 631: *"the pattern matches
nothing — the claim is unchecked, not clean"*. Correct behaviour, and it ruled
out the obvious design.

**112c. The check is the inverse.** For a quantity the tree already knows, **no
*other* numeric claim about it may appear unregistered**. The registered
spelling is blanked out of the text and anything numeric left over is a finding.
That permits rephrasing, forbids drift, and needs no second pattern to maintain.

**112d. It over-detected on its first run, for the third wave running.** The
first pattern matched any `N steps` and fired on *"explanations ≤ 10 steps"* —
the CLARA pipeline, a different subject in the same file. Narrowed to require an
explicit qualifier (`CI`, `checking`, `swept`, `sweep`). Prop. 110's prediction
that an instrument aimed at a new category meets shape 7 first has now held in
three consecutive waves, on three different checks.

**112e. Verified by injection.** Re-inserting *"all forty CI steps"* into a
temporary copy produces exactly one finding naming the gated quantity and the
tree's value. Kept as the third `claims_check` self-test case.

---

### Prop. 113 — the third projection, and a superseded figure stated as live — `FIXED`

**Gate:** `formal-yosys.yml` → *Numbers in the documentation match the tree*

Prop. 110's unfaithful category has four members. Prop. 111 instrumented the
**path** projection of one, Prop. 112 the **scope** projection of another. The
remaining two are timing claims whose captions outlived the conditions they were
measured under — Prop. 85f's contention, Prop. 91c's 22-property configuration.
This is the **provenance** projection, and auditing for it found a live defect
before the gate was written.

**113a. The campaign's own convention makes this decidable.**
FORMAL_FOUNDATIONS propositions are *dated records*, so a duration there is
historical by construction. README is the *current-state* document, so a
duration there is a live claim. A live timing must therefore be **traceable**:
it carries either a provenance marker (the conditions it was measured under) or
a proposition citation the reader can follow to find them. 15 durations in
README, 0 untraceable; an injected *"the whole suite now runs in 47 seconds"*,
300 characters from any citation, is caught.

**113b. The audit found a superseded figure stated as live.** README asserted:

> "took its cheapest step from **153 s to 241 s** — 1.58×, +88 s from two
> properties"

and, three thousand words later:

> "the drain properties turn out to make the engine proof **0.82× — 26 s
> faster** (three paired runs, disjoint ranges, stable inputs)"

The second is provenanced, supersedes the first, and even says so — *"that
removes the stated reason for the Wave-633 split"*. But a reader meeting the
first sentence gets a retracted number with nothing to warn them. This is
Prop. 81d's shape exactly: a withdrawal recorded far from the claim it
withdraws. The first figure now carries an inline forward pointer.

**113c. The gate fired on the documentation of its own predecessor.** Prop. 112's
check flagged *"all forty CI steps"* — appearing in README only because the
Prop. 112 narrative **quotes it as the example of the defect it fixed**. Same
shape as Prop. 95, where a counter read an assertion quoted inside a comment: a
document that discusses a bad claim must contain it. Prose here marks a quoted
string as `*"…"*`, so those are removed before matching.

**113d. Where the three projections leave the category.** Path (Prop. 111),
scope (Prop. 112) and provenance (Prop. 113) each address one recorded member,
and the fourth — Prop. 73's caption naming a *module* where the data described a
*wrapper* — remains uninstrumented. It is a noun-phrase mismatch with no
countable projection, which is why it survived twelve waves. Three of four
members now have a mechanical check; the category is **not** closed.

---

### Prop. 114 — two CI steps were already broken, and the sweep read it as correct — `FIXED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 109 fixed the absence sweep so it starves subjects rather than
instruments. That made the negative control sound. It did not make it
sufficient, because **a step that is already broken also fails when starved**,
and the sweep records that as *"fails, correct"*. Two live instances:

**114a. `Prove zero-size properties` raised `ValueError` after two suites.**
Its `SUITES` list carried a stray third element —
`("weight_prefetch_ctrl", "zs_prefetch", "")` — while the loop unpacks two.
Python raised `too many values to unpack (expected 2, got 3)` after the first
two wrappers, so **`zs_prefetch` and `zs_layer` — four of the eight zero-size
properties — were never proved**, and the step exited 1 in normal operation.

The irony is exact: this is the suite of Prop. 69, whose whole finding was that
eight zero-size properties were counted while no job ran them. Half of them went
back to being unrun, by a one-character defect, and the gate built to detect
that class could not see it.

**114b. `Baseline, control, and mutation` mutated text that no longer exists.**
Its target was the engine emitter's pre-2026-08-09 wording,
`wire start = reg_ctrl[0] && !dma_busy;`. The emitter now writes the declaration
and the assignment on separate lines (`bitnet_engine_top.sv:122` and `:372`), so
the target appeared **zero times**, the mutation was never applied, and the
mutation suite **silently tested 7 of 8 mutants** while the step exited 1. The
mutation guarded Prop. 24's liveness result.

**114c. The missing arm.** A negative control licenses nothing on its own:
"fails when starved" and "works when fed" are two claims, and the sweep only
ever asked the first. `absence_sweep --positive` now runs every step with the
tree **intact** and fails on any that does not pass. Verified both ways — the
fixed step passes, and the re-injected stray tuple is caught with its exact
`ValueError` in the error line.

It is opt-in because it executes the real proofs and costs what CI costs, where
the starved sweep is minutes. That is the honest trade: the guarantee is worth
an hour, and pretending otherwise would put a second unrun check in the tree.

---

### Prop. 115 — over-detection is not rare; it is universal — `MEASURED`

**Gate:** `formal-yosys.yml` → *Numbers in the documentation match the tree*

Prop. 110 recorded three incomplete (shape 7) instances and called the frequency
unmeasured, predicting that an audit which hunted over-detection would find
shapes outside 1–5. A census was run against all ten gates, each probed with
semantics-preserving changes to its subject.

**Every one of the ten over-detects.** Reported instances include:
`identity_scan` splicing `//` comments into an assertion body; `encoding_gate`
failing on an equivalent literal spelling (`2'd0` for `2'b00`); `init_zero_scan`
not recognising a signed zero literal; `guard_scan` firing on a comment that
quotes the guard it looks for; `width_scan`'s floor turning any reformatting
into a failure; `doc_gate` requiring `**Gate:**` at column 0; `orphan_scan`
globbing only `*.yml` so a `.yaml` workflow is invisible.

**115a. The rate.** 10 of 10. Unsoundness was found in 6 of 10 gates across ten
days of adversarial review; incompleteness is in all of them, and was found in
one pass, because nobody had asked. Prop. 110's framing predicted this
asymmetry precisely: the five catalogued shapes are unsoundness mechanisms
*because every audit was instructed to look for unsoundness*.

**115b. Why it is not simply the more dangerous number.** An unsound gate passes
bad artifacts silently. An incomplete gate fails good ones loudly — and a gate
that cries wolf gets disabled, which converts it into an unsound one with extra
steps. The two failure modes are not symmetric in consequence, but they are
symmetric in ending with a gate that does not do its job.

**115c. What this campaign's own record now shows.** Three consecutive waves
each shipped an instrument that over-detected on its first run (Props. 111,
112, 113), and each was fixed by *narrowing the question* rather than adding
exceptions. That is not carelessness three times; it is the default behaviour of
a new check, and the census says the older ones are no different — they have
simply never been probed from this direction.

---

### Prop. 116 — the sweep's verdict was the sign of an exit code — `MEASURED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 114 gave the sweep its positive arm. This addresses the negative arm's own
weakness, and the measurement is worse than the defect.

**116a. Every non-zero exit read as "fails, correct".** A missing binary
(`rc 127`), an unrelated crash, and a hang (`rc -1`) were all printed as a
healthy gate. Verified with synthetic one-step workflows through the shipped
sweep: `yosys_that_does_not_exist` → *"fails, correct"*; `exit 42` →
*"fails, correct"*; `sleep 60` against a 5-second timeout → *"fails, correct"*.

**116b. On the real swept set, most of them are crashes.** Classifying the
captured output of every step, with `diagnosed` requiring positive evidence that
the step *noticed* its subject was gone:

| verdict | count |
|---|---|
| **diagnosed** — produced an `::error::` or a named-absence message | **9** |
| **INDETERMINATE** — exited non-zero saying nothing about the absence | **28** |

So *"0 passing on nothing"* was true and nearly vacuous. Nine steps demonstrate
they read their subject; twenty-eight demonstrate only that they fell over, and
a step that dies with a bare traceback would fail just as readily if it were
simply broken — which is exactly how Prop. 114's two defects hid.

**116c. A ratchet, not a wall.** Failing all 28 today would take the gate out of
service, and a gate that fails correct work gets disabled — Prop. 115b's
mechanism, by which an incomplete gate becomes an unsound one. The count is
published in the summary and capped at its current value: it may fall, never
rise.

**116d. `N exempt` counted membership, not use.** A step in `EXEMPT` that
*failed* was still tallied as exempt, so the summary read identically whether
the exemption suppressed anything. Now counted only when it actually suppressed
a green verdict — which is what the Wave 643 comment already claimed it did.

**116e. What the number means going forward.** 9 of 37 is not a failure of the
suite; it is the first honest measurement of how many CI steps can tell you
*why* they failed. Every step moved from INDETERMINATE to diagnosed is a step
whose green is worth something, and the ceiling makes that progress visible
instead of assumed.

---

### Prop. 117 — the 9-of-37 figure was my classifier, not the suite — `FIXED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 116b reported **9 diagnosed against 28 indeterminate** and concluded that
*"0 passing on nothing"* was "true and nearly vacuous". That number was wrong,
and this corrects it in the manner Prop. 85f established.

**117a. What the 28 actually printed.** Every one names the exact missing file,
in the tool's own words:

```
ERROR: File `build/rtl/interrupt_controller.sv' not found or is a directory
FileNotFoundError: [Errno 2] No such file or directory: 'formal/zero_size_props.sv'
```

Those *are* diagnoses of the absence. The classifier looked only for this
repository's own `::error::` convention plus three house phrases, so it scored
yosys's and Python's perfectly clear messages as silence. Corrected figure:
**37 diagnosed, 0 indeterminate.**

**117b. The right criterion is Prop. 114's question, not a house style.** The
distinction that matters is whether a failure can be told apart from *"the step
was already broken"*. A message naming a **starved path** can. `ValueError: too
many values to unpack`, `command not found`, and a timeout cannot — and that
ValueError is exactly how one of Prop. 114's two defects hid. The classifier now
tests for a named subject under `build/rtl` or `formal/`, and is checked against
seven cases including both Prop. 114 defects: 7/7.

**117c. The ceiling becomes a wall.** Prop. 116c set a ratchet at 28 on the bad
measurement, reasoning that failing 28 steps would disable the gate. With the
true count at zero, the ratchet has nothing to ratchet: any step that fails
without naming what it was missing is indistinguishable from one already broken,
so the ceiling is 0 and enforced.

**117d. The self-test was modelling something the tree does not contain.** Its
"honest step" was a bare `test -f`, which fails silently — no real step in this
repository does that. Made realistic, and given a counterpart that fails *without*
naming its subject and must be caught. Both directions now covered.

**117e. Fourth consecutive wave, and this one in the measurement itself.**
Props. 111, 112 and 113 each shipped an instrument that over-detected on its
first run. This one over-detected in a *number that was then published*, which is
worse: a wrong gate fails loudly, a wrong measurement propagates. Prop. 115's
finding that over-detection is universal applies to classifiers as much as to
gates, and the corrective is the same — check the instrument against cases whose
answer you already know before believing its output.

---

### Prop. 118 — six of the ten over-detections, fixed — `FIXED`

**Gate:** `formal-mutation.yml` → *No gate passes when its subject is absent*

Prop. 115 found all ten gates fail some semantics-preserving change. Each census
entry carried a proved equivalence — a yosys `miter -equiv` for the RTL cases, a
CommonMark render for the documentation one — so each is a gate rejecting work
that is genuinely correct. Six were one-line defects:

| gate | what it rejected | cause |
|---|---|---|
| `init_zero_scan` | `16'sd0` as a reset value | the base class omitted SystemVerilog's signed marker `s`, though this codebase writes signed literals everywhere |
| `claims_check` | a re-aligned column in a shell script | `^ +probe '` demanded exactly one space |
| `doc_gate` | a `**Gate:**` line indented two spaces | `startswith` at column 0; the indented form renders to byte-identical HTML |
| `bound_scan` | `` // BOUND: `accumulator` `` | `(\w+)` cannot match a backticked name — the quoting style the gate's own errors use |
| `identity_scan` | a comment *inside* an assertion body | bodies were normalised without stripping comments, so a comment explaining why a property is **not** a self-comparison made it read as one |
| `guard_scan` | a comment saying an open-guard "has been removed" | matched inside `//` comments |

**118a. Three of the six were the same mistake.** `identity_scan`, `guard_scan`
and (in Props. 95 and 102c) `claims_check` and `orphan_scan` all matched text
inside comments. That is now five instances of one shape across four files, and
each was found separately rather than by grepping after the first. Prop. 103's
third regularity — the same defect recurs in sibling files — has cost more here
than any other single lesson.

**118b. Every one rejected this repository's own conventions.** A signed literal,
a backticked identifier, an indented markdown line, a retrospective comment
about a removed guard: not exotic inputs, but the house style. A gate written
from an author's mental model of the code encodes that model's blind spots, and
the author's own idioms are exactly what it fails to anticipate — because they
were invisible while writing it.

**118c. Four remain.** `encoding_gate` matches exact literal text so `2'd0` for
`2'b00` reads as a defect; `mirror_check` compares net names so a consistent
internal rename fails; `width_scan`'s floor turns a line-wrap into a CI failure;
`orphan_scan` globs only `*.yml` so a `.yaml` workflow is invisible. Each needs
more than a character, and each is recorded rather than quietly deferred.

Verified: 10 injections from the census, all now quiet, with the discriminating
cases checked in both directions — `16'sd0` reads as zero and `16'sd1` does not.

---

### Prop. 119 — closing a class that cost five waves — `FIXED`

**Gate:** `formal-yosys.yml` → *No gate reads Verilog without stripping comments or saying why*

One defect shape has now taken five separate fixes across four files:

| prop | gate | what a comment did |
|---|---|---|
| 95 | `claims_check` | invented a 29th integration property out of a comment quoting an assertion |
| 102c | `orphan_scan` | identical defect, identical regex, sibling file — survived a wave because nobody grepped |
| 118 | `identity_scan` | a comment explaining why a property is **not** a self-comparison made it read as one |
| 118 | `guard_scan` | a note saying a guard "has been removed" reported it as present |
| 118 | `bound_scan` | rejected the repository's own backtick style in a note it parses |

Every one was found on its own, wave after wave. `formal/comment_scan.py` closes
the class: any gate that reads Verilog and applies a regex must strip `//`
comments first, or declare in writing why it does not.

**119a. Declaring is the interesting half.** Four gates read comments *on
purpose* and now say so — `width_scan` parses `range [-N, +M]` annotations and
would have nothing left if they were stripped; `phantom_scan` matches yosys
warning output, where `//` cannot occur; `faith_check` reads `formal/*.py`, and
the `build/rtl` strings that put it in scope are path literals it compares
against docstrings; `encoding_gate` permutes declarations in a copy fed only to
yosys, which ignores comments anyway. The marker forces that question to be
answered once, in writing, where a reader can check it — rather than being
rediscovered by a defect.

**119b. It over-detected on its first run, for the fifth wave running.**
`mutate.py` does strip comments, in a helper called `code_mask` that also masks
nested `` `ifdef T27_FORMAL `` regions and labelled assertion lines. The
recognised-stripper list did not know that name, so a gate doing exactly the
right thing was reported as one that was not — over-detection inside the gate
written to close an over-detection class. Fixed by recognising the name.

**119c. What this is worth.** Not the five fixes — those were already made. It
is that a sixth instance now fails the build instead of being discovered by
whatever it breaks. The pattern had a textual signature the whole time, and five
waves went by without anyone searching for it.

### Prop. 120 — the requantizer never flushes, and a property asserts that it doesn't — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove activation_requant properties*

The first adversarial pass at the **design** since Prop. 80 — twelve days of work
having gone entirely into instruments — returned six yosys-verified findings.
This is the one independently reproduced and judged reachable in the assembled
engine.

**120a. The defect.** `activation_requant` packs 27 neuron results into one
54-bit word and raises `word_valid` **only** at `trit_count == 26`. There is no
flush: no `layer_done` input, no partial-word emission. A layer whose neuron
count is not a multiple of 27 leaves its last `num_neurons mod 27` results in a
partial word that is never emitted, and **nothing constrains `num_neurons` to a
multiple of 27** — it is a free 16-bit port.

**120b. A property asserts the gap.** The module carries
`a_word_only_on_full: assert ($past(trit_count) == 5'd26 && $past(valid_in))`,
which *proves the module never emits a partial word*. The behaviour is encoded
as intended. This is precisely the Wave 628 shape: a defect that was not merely
untested but **protected** by something asserting it — there, a unit test pinning
the wrong width; here, a formal property pinning the missing flush.

**120c. The annotation says the opposite of the design.** Props. 84/95b annotate
`act_wr_word` as advancing **`ceil(num_neurons / 27)`** times per layer. The RTL
advances **`floor`**. Two readings, one file apart, disagreeing about the design
— and the `ceil` reading is what the design was *intended* to do.

The bound argument built on it is unaffected, since `floor ≤ ceil` and the
16-bit port keeps the count under 4096 either way. So this is not a safety defect
in the bound; it is a **false statement about the design**, whose falsity points
exactly at the functional gap. The annotation now states `floor`, names the
consequence, and cites this proposition.

**120d. Why no property caught it.** Prop. 81b established that the integration
suite constrains **control** — handshakes, phase, contiguity, readiness — and a
genuine arithmetic defect in a module it depends on disturbed none of them. This
is the same boundary from the other side: dropping the last 26 neurons of a layer
is a **data** loss that leaves every handshake correct. The engine runs, the
buffers fill, the phase alternates, and the answer is wrong.

**120e. What is not yet decided.** Whether the fix is a flush path in
`activation_requant`, a contract requiring `num_neurons ≡ 0 (mod 27)` enforced at
the sequencer, or a reader that tolerates a partial final word, is a **design
decision** and not one to take unilaterally — it changes emitted hardware. What
this proposition fixes is the documentation that was false and the record that
was missing. The remaining five findings from the hunt are pending independent
refutation.

---

### Prop. 121 — five confirmed design defects, and multi-layer inference does not run — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Prop. 120 recorded the first of the hunt's findings. The refutation phase has now
independently reproduced **five**, every one judged reachable in the assembled
engine. This is the largest single result of the campaign, and it is about the
**design** rather than the tooling.

| # | defect | reproduced by |
|---|---|---|
| 1 | `activation_requant` has no flush — a layer's trailing `N mod 27` results are never emitted | module-level harness (Prop. 120) |
| 2 | those trailing trits **leak into the next layer's word**, carrying stale values across a layer boundary | concrete trace: 2 beats `acc=+1`, idle, 25 beats `acc=−1`, and an emitted word still carries `TRIT_P` |
| 3 | the activation buffer is indexed by **neuron**, not by **chunk** | formal, on repo RTL |
| 4 | **multi-layer inference deadlocks** | independent Icarus testbench driving the assembled engine *only through its AXI4-Lite CSR aperture*, with a compliant AXI4 read-slave |
| 5 | the ping-pong flips **two cycles before** the requantizer emits a layer's final word | timing derived from the RTL, then proved |

**121a. Two of them share one line.** `double_buffer_ctrl.sv:35` reads
`assign read_addr = neuron_id;`, and that drives `rd_addr` on both activation
memories. A neuron's input vector spans `num_chunks` words and **every neuron
must see the same vector**; addressing by neuron gives each neuron a different
word instead. That is defect 3, and it is also the root of the deadlock in
defect 4 — the layer-start gate compares packed-word *slots* against a *neuron*
count, two quantities that only coincide when `num_chunks == 1` and
`num_neurons ≤ 27`.

**121b. What the engine's 28 integration properties say about this.** They all
still prove. Prop. 81b named the boundary and this is the demonstration at
scale: handshakes, buffer phase, address contiguity and readiness are all
correct while the machine computes the wrong answer and, for more than one
layer, does not terminate. A control suite reports perfect health straight
through five data defects.

**121c. Why the design-level pass found in one day what twelve days of
instrument work did not.** Nothing was wrong with the instruments — Props.
111–119 fixed real defects in them. But every one of those waves asked *"is this
gate sound?"* and none asked *"is the design correct?"*. The taxonomy of
Prop. 103 predicted exactly this: a catalogue of failure shapes is a catalogue of
the questions asked. Twelve days of asking one question produced no answers to
the other.

**121d. Not fixed here, and that is deliberate.** Each defect has a proposed fix
in the emitters — a flush port on the packer, a chunk counter for the buffer
index, a drain interlock before the ping-pong flip. All change emitted hardware
and interact with each other; defect 3's fix likely subsumes defect 4's. Choosing
among them is a design decision, and the campaign's own discipline says a
verification pass records what it found rather than rewriting the subject.

**121e. The properties that assert the defects must go with the fixes.**
`a_word_only_on_full` proves the missing flush is intended (Prop. 120b). Any
flush implementation fails it. Whoever takes the fix must retire that property in
the same change, or the suite will reject the repair — which is what "protected
by an assertion" costs.

---

### Prop. 122 — a sixth defect, what was proved clean, and a claim I over-stated — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

The design hunt completed: 11 agents, no errors. Three things it produced that
Prop. 121 did not record.

**122a. A units mismatch that may block layer 0 as well.**
`bitnet_engine_top.sv:351` passes `.length(reg_neurons)` to `dma_controller`,
whose header states **"One beat = 8 bytes (64-bit). length is byte-count"** and
which writes one local address per beat. The same register is read at line 124
as `neurons_per_layer`, a *neuron* count.

So for N neurons the input DMA moves N **bytes** — ⌈N/8⌉ words — while the
readiness gate demands `filled >= neurons_per_layer` = N. For N ≥ 2 the gate can
never be satisfied, so the deadlock of Prop. 121's defect 4 may reach **layer 0**
and not only layer boundaries. Confirmed by reading the contract against the
call; not yet reproduced in simulation, and recorded at that strength.

**122b. What was proved CLEAN, which is worth as much as the defects.** The
quantiser was checked against an independent 17-bit reference over **all**
inputs and is correct — including the `TRIT_Z` fall-through that no inline
property asserts, and including `threshold = 16'sh8000`, where the 16-bit
negation overflows but the priority chain masks it, so the 16- and 17-bit
results are observationally equal. The packing order matches its documentation
exactly (trit *i* at bits `[2i+1:2i]`), proved against an index-addressed
reference. `2'b11` is unreachable in **all 27 fields** of `word`, not merely in
the scalar `trit` that `a_trit_never_invalid` guards. And the reset value
decodes as 27×`TRIT_N` rather than 27×`TRIT_Z` — but is never observable,
because `word_valid` gates it and 27 beats flush the shifter first.

Five defects sit next to four proved-correct behaviours in the same module. A
report that lists only the failures misrepresents the design.

**122c. I over-stated the root cause in Prop. 121a.** That proposition asserts
`read_addr = neuron_id` "is defect 3, and it is also the root of the deadlock in
defect 4". The **refuting** agent concluded that. The **hunting** agent explicitly
declined to: *"I did not adjudicate which side of Finding 2 is wrong … if that
reader addressing is itself the defect, the fix moves to the reader and the
requantizer's 27:1 packing stands."*

Two readings remain open — either the reader should address by chunk, or the
packer should not pack 27:1 — and 121a presented one agent's judgement as
settled. The finding stands; the attribution of the root does not. Prop. 122a
adds a third possibility, that the DMA length is the primary error and both
downstream readings are consequences.

---

### Prop. 123 — a gate that could not see the defect it was written for — `FIXED`

**Gate:** `formal-yosys.yml` → *No port is driven by a signal naming a different quantity*

Prop. 122a found a defect no property covers, because each side of it is
internally consistent: `dma_controller` is right that `length` counts bytes, the
engine is right that `reg_neurons` counts neurons, and **nothing looks at what
joins them**. `formal/units_scan.py` reads names across module boundaries and
reports a port and its driver falling in different quantity families.

**123a. A general units system would be the right answer, and this is not one.**
Exactly one module in the bundle documents its units at all, so a gate requiring
declared units would check one port and pass — the failure of Wave 646's
abandoned design, repeated. This reads the *names*, which the emitters write
consistently: `.length(reg_neurons)` is a mismatch visible without annotation.

**123b. It could not see the connection it was built for.** The first version
captured an instantiation body with a non-greedy `(.*?)\);`, which stops at the
first `);` and cannot survive a nested parenthesis. The engine's DMA
instantiation opens with
`.start(reg_ctrl[1] && !reg_ctrl[0] && …)`, so the **one connection the gate
exists to catch was never parsed**. Eleven instantiations were read;
`dma_controller` was not among them; the tree reported clean.

And the floor did not help. `compared > 0` passed because twenty *other*
connections were compared — **a floor on a total says nothing about coverage of
the thing you care about**. Fixed by matching the body with paren depth.

**123c. It over-detected first, for the sixth consecutive wave.** The initial
vocabulary put `chunk` and `word` in different families, so
`.input_chunk(activation_word)` and `.weight_chunk(weight_word)` were reported —
but a chunk *is* a 54-bit packed word here. The vocabulary encoded a distinction
the design does not make. Merged, and the self-test case that asserted the
distinction is kept **inverted**, as the regression that holds them together.

A control keyword was also parsed as a module name: `else if (length == …)` read
as an instantiation called `else`, which produced the *original* "real" finding
at a line where no instantiation exists. That coincidence — a false positive
naming exactly the right two signals — is why the parse defect went unnoticed
through a full self-test run.

**123d. The known defect is declared, not silenced.** Prop. 122a is real and
unfixed, because the repair is a design decision. Following Prop. 26's
expected-refutation convention, it is listed in `KNOWN_OPEN` with its reason and
issue, reported as a warning, and **anything not on that list fails the build**.
Removing the entry without fixing the defect turns the gate red, which is the
point.

---

### Prop. 124 — name the subject a gate exists for, and two negative results — `FIXED`

**Gate:** `formal-yosys.yml` → *No port is driven by a signal naming a different quantity*

Prop. 123 showed a gate passing its coverage floor while missing exactly the
artifact it was written for: `compared > 0` held on twenty other connections
while `dma_controller` went unparsed. **A floor on a total says nothing about
coverage of the thing you care about.** Three gates now name their subject.

| gate | witness | why that one |
|---|---|---|
| `units_scan` | the `dma_controller` instantiation is parsed | Prop. 122a's connection |
| `width_scan` | `l2` is among the declarations examined | Prop. 80's defect site |
| `bound_scan` | `accumulator` is among the registers classified | Prop. 83's register |

All three verified by renaming the subject in a scratch copy: **3/3 fire**.

**124a. Widening the units vocabulary was mostly a negative result.** Enumerating
the 141 skipped connections was supposed to reveal unchecked quantities. It
revealed `clk`, `rst_n`, `rd_data`, `wr_en`, `a`, `b`, `sum`, `cin`, `cout` and
AXI handshakes — **not quantities at all**. My previous report framed this as
"the vocabulary covers 14%", implying 86% of quantities were unchecked; the truth
is that most connections are not quantities and are correctly skipped. One family
was genuinely missing — addresses — and adding it took the compared count from 23
to 42 with **0 new disagreements**.

**124b. Two of my own tests were wrong before either gate was.** The
`width_scan` witness appeared not to fire because its *reduction floor* caught
the mutation first — the gate failed correctly, by a different guard, and the
test demanded one specific message. And the `bound_scan` witness appeared silent
because I renamed only the register's **declaration**, while that gate identifies
registers from **assignments**; the mutation never removed the subject from its
view. In both cases the instrument was right and the check of it was wrong, which
is the mirror of Prop. 89b.

**124c. And an edit that silently did nothing.** The `width_scan` witness was
first inserted with `str.replace()` on an anchor that did not match, with no
assertion on the count — so `names_seen` stayed empty and the witness fired
against the shipped tree. This campaign has written down "assert your injection
landed" three times (Props. 82d, 98, 111) and it was violated in the wave that
cites it. Re-applied with `assert s.count(old) == 1`.

---

### Prop. 125 — one units confusion with four faces, and a root cause refuted — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

The adjudication swept the assembled engine through its CSR aperture with
Icarus: N = 0…80 at C=1,L=1 plus the full grid N ∈ {1,2,8,26,27,28,54} × C ∈
{1,2} × L ∈ {1,2}. It settles the question Props. 121–122 left open, and the
answer is not on the candidate list.

**125a. Exactly one configuration in 81 works.** No configuration with
`num_neurons ≥ 2` starts layer 0. Measured mechanism: `dma_controller` writes
`ceil(N/8)` words — observed 1,1,1,4,4,4,7 for N = 1,2,8,26,27,28,54 — while the
gate requires `filled >= neurons_per_layer` = N. `ceil(N/8) ≥ N` holds only for
N ≤ 1. `layer_start_g` never fires, `buffer_unwritten` pulses, the error IRQ
latches, and `busy` sticks at 1 forever.

The harness is not the cause: the same testbench terminates cleanly with the
done IRQ on N=1,L=1 and on 20 of 28 configurations of the repaired variant.

**125b. The three candidates, each tested alone.**

| candidate | change | result |
|---|---|---|
| (b) reader index | `read_addr = neuron_id` → `neuron_id/27` | **byte-identical to stock across all 28 configurations** |
| (c) packer ratio | 27:1 → 1:1 | identical to stock for every N ≥ 2 |
| (a) DMA length | `.length(reg_neurons)` → `.length(8·reg_neurons)` | **the only single change that unblocks layer 0** |

**Candidate (b) changes nothing whatsoever — it is neither necessary nor
sufficient for any observed behaviour.** Prop. 121a published it as the root of
two defects. Prop. 122c already recorded that as one agent's judgement rather
than a finding; it is now **refuted outright**.

**125c. And (a) is not primary either.** With (a) fixed, layer 1 still never
starts for *any* N. With (a)+(c) together, likewise. Multi-layer inference is
blocked by something none of the three candidates touches.

**125d. The primary error is a fourth reading the list did not contain.** The
activation buffer must be indexed by **chunk**, not by neuron.
`trit27_dot_product` consumes 54 bits of activation against 54 of weight — 27
inputs per cycle — and `chunk_addr` walks `neuron·C + chunk` over the weight
memory, so the weight store is an N×C matrix of 27-input chunks. The input
*vector* is therefore C words of 27 trits, and **every one of the N neurons reads
the same C words**.

Under that reading the whole picture changes: **(c) is correct and not a defect
at all**; (b) is wrong but `neuron_id/27` is also wrong — the right value is
`chunk_id`; (a) is wrong because the length should be `chunks_per_neuron·8`
bytes; and the gate comparing *words* against *neurons* is a fourth error nobody
had listed. **All four are faces of one units confusion: neurons versus 27-trit
chunks** — the same confusion `units_scan` was built for in Prop. 123, one level
up.

**125e. Confirmed by construction.** Five coherent changes — read address =
`chunk_id`, DMA length = `chunks_per_neuron·8`, gate against
`chunks_per_neuron`, an end-of-layer flush, and the ping-pong flip delayed 5
cycles — make two-layer inference run to completion with the done IRQ and no
error for **every** configuration where `ceil(N/27) ≥ C`. Layer-0 `act_words` is
then `ceil(N/27)` exactly. The configurations that still refuse are precisely
those asking layer 1 to consume more chunks than layer 0 can produce, and they
report the error IRQ rather than computing garbage. Predicted and measured
patterns match with no exceptions.

**125f. Two earlier defects now quantified in the assembled engine.** The
no-flush defect (Prop. 120) emits `act_words = floor(N/27)` exactly — 0,0,0,0,1,
1,2 for the seven N values, so **N=26 produces zero activation words for
twenty-six computed neurons**. The ping-pong defect loses exactly 2 words at C=1
and 1 word at C=2.

**125g. What this says about the method.** Three candidates were enumerated from
static reading and adversarial proof, and the true root was none of them. It took
*sweeping the assembled machine* to find that the three were faces of a fourth.
A defect list assembled from module-level analysis can be complete about symptoms
and wrong about causes, and only the whole system can adjudicate.

---

### Prop. 126 — the fix, applied and verified both ways — `FIXED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Prop. 125 identified one units confusion with four faces and demonstrated a
repair. This applies it to the emitters and closes the verification.

**126a. Fifteen edits, all asserted.** `activation_requant` gains a `flush_in`
port and an end-of-layer branch that right-aligns a partial word.
`bitnet_engine_top` addresses the activation buffers by **chunk**
(`act_rd_addr = chunk_id`), takes the DMA length as `chunks_per_neuron·8` bytes,
compares the readiness gate words-to-chunks, and routes `layer_done` through a
five-stage delay so the requantizer drains before the ping-pong flips. Each edit
was asserted against its anchor, and the regenerated bundle was then checked to
carry all fifteen changes the verified variant had: **15/15**.

**126b. Simulation.** Driving the assembled engine only through its AXI4-Lite CSR
aperture: layer 0 starts and completes for **every** configuration in the sweep,
against exactly one of eighty-one before. Two-layer inference completes with the
done IRQ for every configuration where `ceil(N/27) ≥ C`; those asking layer 1 for
more chunks than layer 0 can produce raise the error IRQ instead of computing
garbage.

**126c. Four properties encoded the defect, not the contract.** The integration
suite refuted at first. A before/after control against the pre-fix tree showed
all three engine properties **proved before and refuted after**, so the fix
changed them — the question was whether they described the design or the bug.

- `a_buffer_alternates` hung on `$past(layer_done_pulse)`, asserting the flip
  happens one cycle after the strobe. That *is* Prop. 121's defect 5. Re-pointed
  at `layer_done_dly`.
- `a_read_slot_written` and `a_read_within_written` tracked `buf_read_addr`,
  which the repair disconnected from the activation memories. They were
  asserting about a wire that no longer reaches the thing they describe.
  Re-pointed at `act_rd_addr`, in both the A and B arms.
- `a_word_only_on_full` (Prop. 120b) proved the packer never emits a partial
  word. Retired for `a_word_on_full_or_flush`.

None was weakened: each keeps its claim and names the signal that now carries the
meaning it was written about. **All 28 integration properties then prove at
`seq 40`**, and the simulation sweep is unchanged.

**126d. The count of defect-asserting properties is now four.** Props. 80 and 120
each found one assertion pinning a defect; this wave found three more in a single
repair. A verification suite that has grown alongside a defect will contain
properties that *are* the defect, and the repair must retire them in the same
change or read as a regression. Budget for that when estimating a fix.

**126e. What is still not established.** The simulation covers the configurations
swept, not all of them; the formal proofs are bounded at `seq 40`. The engine now
runs and its properties hold — neither statement is "the design is correct".

---

### Prop. 127 — the first end-to-end value check, and what it found — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Every property in this campaign constrains **control**. Prop. 81b named that
boundary; Prop. 121 demonstrated it at scale, with 28 properties proving while
the machine computed the wrong answer and deadlocked. **Nothing had ever compared
an engine output against a reference.** `sim/tb_data_check.v` is the first.

**127a. How it is possible.** The engine's two memory ports are separate: the DMA
reads input activations over `m_axi_*`, the prefetcher reads weights over
`mem_rd_*`. So the testbench can serve a known input vector on one and known
weights on the other, compute the expected result itself, and compare. The
existing sweep harness fed a constant to both, which is why it could only ever
check control flow.

The vector is 9×(+1), 9×(0), 9×(−1) against all-(+1) weights, so the reference
accumulator is exactly 0 and the reference trit is `TRIT_Z` at threshold 3 — a
value that is wrong under almost any indexing error, since a mis-addressed read
would pick up a different mixture.

**127b. WITHDRAWN — the MAC agreement was my variable's initial value.** This
proposition first reported "engine `acc = 0`, reference `acc = 0`" as the first
end-to-end numerical agreement in the campaign. It is not one.

`acc_seen` is initialised to 0 in the testbench and assigned only under
`mac_valid_q`. I never established that assignment ever fired, and a trace of the
requantizer's inputs shows `acc = xxxx` arriving at the moment `valid_in` is
asserted — which is inconsistent with a measured zero. The reported "agreement"
is a variable that was never written matching a reference that happens to be
zero.

The vector was chosen so the reference would be 0 *because* that value is wrong
under most indexing errors (127a). That choice made the reference collide with
the one value an uninitialised counter would show, and the collision was not
noticed. **A reference value chosen to be discriminating against the design can
still be indiscriminate against the harness.**

**127c. The X is traced to an unwritten weight memory, and the harness is why.**
A cycle-level trace of the requantizer's inputs settles it: `acc` is already
`xxxx` when `valid_in` is asserted, so the X does not originate in the requant
path. `shift_word` is reset to `54'd0`, and the undefined value arrives from
upstream.

The cause is that the weight BRAM is read before anything writes it. Two harness
errors compounded, both mine: the testbench started inference on a fixed 200-cycle
delay rather than on any observable condition, and a subsequent attempt to wait
for `prefetch_done` **deadlocked**, because the weight prefetch is triggered *by*
the inference start and therefore cannot complete before it.

So the X is a harness artifact so far as it has been established, and the value
check does not yet demonstrate anything about the design. That is the honest
status: an instrument built this wave, not yet a measurement.

**127d. Why this belongs in the repository rather than a scratch directory.** The
harness is the only artifact in the campaign that can answer "does the engine
compute the right number". Its absence is what let six defects through. It is
checked in at `sim/tb_data_check.v` with the Icarus declare-before-use adapter
beside it, so the next wave extends it rather than rebuilding it.

---

### Prop. 128 — the value check now fails for a named reason — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Prop. 127b withdrew a false agreement. This wave makes the harness capable of
producing a real one, and it now fails in a way that says why.

**128a. A reference no uninitialised variable can produce.** The old vector was
9×(+1), 9×(0), 9×(−1), chosen so the reference accumulator would be exactly 0 —
wrong under most indexing errors, and *also* what an unwritten counter reads. It
could not distinguish a working engine from a silent harness. The vector is now
27×(+1) against all-(+1) weights: reference `acc = 27`, reference trit
`TRIT_P`. Neither value is reachable by an uninitialised register.

**128b. A flag proving the capture fired.** `acc_seen` is assigned only under
`mac_valid_q`, so it now carries a companion `saw_mac`, and the harness reports
*"the MAC never produced a result — nothing was measured"* rather than comparing
an initial value. Comparing an unassigned variable against a reference is not a
measurement, and it looks exactly like a passing test.

**128c. The measurement, and why it still says nothing about the design.**
`saw_mac = 1`, one MAC result, engine `acc = 0` against reference 27. That is a
genuine measurement — and a probe on the weight path explains it:
**`weight_bram writes = 0`.** The prefetcher never writes a single word, so the
MAC computes against an unwritten memory. The mismatch is explained by absent
weights, not by a defect in the datapath.

**128d. What has actually been gained.** Three waves ago the harness reported a
false agreement. Two waves ago it reported an unexplained `X`. It now reports a
specific, localised failure — *no weight ever reaches the memory the MAC reads* —
which is a statement that can be acted on. The progression is from a wrong answer
to no answer to a **named missing precondition**, and only the last of those is
a foundation.

**128e. Still not established.** Whether the prefetch fails because the harness
does not drive it correctly or because the design does not start it is open. The
sweep harness of Prop. 125 did raise a prefetch IRQ in some configurations, which
suggests the path can work and points at this harness first. That is a lead, not
a conclusion.

---

### Prop. 129 — nothing loads layer 0's weights — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 24, deep bound)*

Prop. 128 localised the value check's failure to `weight_bram writes = 0` and
recorded two hypotheses: the harness does not drive the prefetch, or the design
does not start it. **It is the design.**

**129a. The probe.** `start_prefetch = 0`, `mem_rd_en = 0`, `mem_rd_valid = 0`,
`prefetch_done = 0`, `bram_we = 0`. Nothing about the weight path is exercised
at all — this is not a stalled handshake or a wrong parameter, it is a signal
that never asserts.

**129b. Why.** `multilayer_sequencer` asserts `start_prefetch` in exactly one
place: in `LAYER_RUN`, when `layer_done` fires **and** `current_layer` is not the
last. It exists to fetch the weights for the **next** layer. There is no
corresponding load before the first, and the engine has no other weight-write
path — its only weight interface is the prefetcher's read port
(`mem_addr`/`mem_rd_en`/`mem_rd_data`). So **layer 0 always computes against an
unwritten weight memory.**

**129c. Corroborated by the earlier sweep — with a correction to how I read it.**
Prop. 125's harness recorded a prefetch-IRQ column, and I first reported it as
**0 for every configuration, L=1 and L=2 alike**. That was column 15. The header
puts `irq_pf` at column **16**, and column 15 is `irq_done`.

The correct column says something sharper: `irq_pf` is **0 for every
single-layer run and 1 where a second layer exists**. That is exactly what the
mechanism predicts — the prefetch fires between layers and never before the
first — so it is stronger evidence than the flat zero I claimed, and the claim I
published was false about the data while right about the conclusion.

The table has been in the tree since Wave 658. The finding was in a column read
as "nothing interesting", and then briefly in the wrong column.

**129d. Why no property caught it.** The same boundary as Props. 81b and 121: the
28 integration properties constrain handshakes, phase and readiness, and an
engine reading an unwritten memory violates none of them. It runs, it completes,
it raises done. The weight memory's *contents* are not something any property
mentions.

**129e. What this does not settle.** Whether layer-0 weights are meant to arrive
by a route that was never built, or whether the sequencer should prefetch before
the first layer as well, is a design question. The measurement is that no route
exists today and every layer-0 result is computed against undefined data.

---

### Prop. 130 — a property about memory contents, gated as the defect it is — `MEASURED`

**Gate:** `formal-yosys.yml` → *No property is gated as an expected refutation*

Three defects now share one shape: **control properties cannot see what is in a
memory.** Props. 81b, 121 and 129 each passed a suite that constrains handshakes,
phase and readiness while the engine read data nothing had written.

**130a. The pattern already existed, for the other memory.** The activation
buffer has carried `a_read_slot_written` since Prop. 33 — a per-slot written
bitmap, asserting the MAC never reads a slot nothing wrote. It was never applied
to the **weight** memory, and Prop. 129 is what that omission cost.

`a_weight_read_was_written` is the same construction: a bitmap set by
`pf_bram_we`, asserted against `chunk_addr` whenever `layer_valid`.

**130b. It refutes, and that is the point.** Verified both ways: with
`T27_FORMAL_OPEN` defined the engine suite exits 1; in CI's configuration it
exits 0. The property is gated as an **expected refutation** (Prop. 26), so the
gap is stated inside the suite rather than only in prose, and a layer-0 weight
load cannot land without someone moving this property out of the guard.

**130c. Why encode a defect you are not fixing.** The alternative is a
proposition describing it, which is what Props. 25b and 39e already are — and
Prop. 129 was found by probing, not by reading them. A refuting property is
checked on every run; a paragraph is checked when someone happens to read it.

**130d. What it does not do.** It bounds the check at four tracked words
(`chunk_addr[1:0]`), matching the existing activation-side pattern and the
`DEPTH 4` CI parameter. A wider memory needs a wider bitmap, and this says
nothing about weights beyond the fourth address.

---

### Prop. 131 — layer 0 now loads its weights, and the property still refutes — `MEASURED`

**Gate:** `formal-yosys.yml` → *No property is gated as an expected refutation*

Prop. 129 found that nothing loads layer 0's weights: `start_prefetch` is
asserted only when a layer finishes and another follows, making the prefetcher a
between-layers mechanism with no initial load. This completes it.

**131a. The change reuses existing machinery.** The prefetcher already does the
right thing. `IDLE` now routes `start` through the existing `PREFETCH` state
rather than straight to `LAYER_RUN`, and a `first_load` flag suppresses that
state's `current_layer` increment — which is correct when fetching for the *next*
layer and wrong for an initial load. That keeps the two-bit state encoding rather
than adding a fifth state.

**131b. Simulation: the weight memory is written for the first time.** Before,
every signal on the weight path read zero. After: `start_prefetch = 1`,
`mem_rd_en = 3`, `mem_rd_valid = 3`, **`bram_we = 1`**, `prefetch_done` asserted.
And the emitted activation trit is now `2'b10` — **`TRIT_P`, matching the
reference** — where it was `X` two waves ago and `xx` before that.

**131c. The formal property still refutes, and that is not a detail.**
`a_weight_read_was_written` was added in Prop. 130 as an expected refutation
encoding this exact gap. It **still refutes at `seq 40`** on the repaired design.

Two readings, and this proposition does not choose between them: the bound may be
too short for the prefetch to complete from reset in the formal model, or a
reachable case remains where the MAC reads an unwritten address. The simulation
result and the formal result disagree, and the honest position is that the defect
is **not closed** until they agree.

**131d. What is established.** The weight path is exercised where it was
completely idle, and one end-to-end trit matches its reference. That is the first
agreement between an engine output and a computed expectation in this campaign —
and it is one value, in one configuration, against a property that still fails.

**131e. The MAC accumulator still reads 0 against a reference of 27.** The
harness captures `mac_result` under `mac_valid_q`, which fired once. A design
producing `TRIT_P` cannot have an accumulator of 0 at threshold 3, so the capture
and the requantizer are almost certainly sampling different cycles — a harness
question, unestablished, and not to be reported as a design result either way.

---

### Prop. 132 — the engine trusted a CSR that resets to zero — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (all 28, tracker-backed included)*

**132a. The activation path had the interlock; the weight path did not.**
`start` has been held behind `input_loaded` for many waves — the DMA must have
written the buffer before inference begins. `weight_words` comes from
`reg_chunks[31:16]`, and nothing checked it at all.

**132b. `weight_prefetch_ctrl` reports success for a fetch it never performs.**
Line 92 reads `if (num_words != 16'd0)`; the zero case falls through to `DONE_ST`
and asserts `prefetch_done`. Simulation at `weight_words = 0` — **the reset value
of `reg_chunks`** — gives `bram_we = 0`, `mem_rd_en = 0`, `prefetch_done` high,
the MAC running, and an activation word of `X` emitted. A host that starts
inference without writing `0x18` gets exactly this, silently.

**132c. Non-zero is not the contract.** The `!= 0` guard alone did not make the
weight-contents property provable. The prefetcher writes `weight_words`
addresses while the MAC walks `neurons × chunks_per_neuron`; declaring too few
weights reads unwritten memory as surely as declaring none. The guard is
`weight_words != 0 && weight_words >= neurons × chunks_per_neuron`, widened to
24 bits rather than truncated.

**132d. A refusal the host cannot see is a hang.** `cfg_err` is sticky, exposed
as `reg_status[2]`, and carries its own property — `a_refused_start_is_reported`
— which is deliberately *not* a restatement of the guard: the guard suppresses
the start, the property requires the suppression be observable.

**132e. What this did not fix.** The weight-contents property still refutes,
with a counterexample requiring between 15 and 18 cycles. The configuration
guard was necessary and is not sufficient.

---

### Prop. 133 — a proof under assumptions is worthless without an emptiness check — `PROVEN`

**Gate:** `formal-yosys.yml` → *Vacuity gate — no property may pass on an empty trace set*

This is the most consequential result in the campaign so far, and it is about the
method rather than the design.

**133a. Two experiments, both vacuous, both initially read as findings.** To test
whether a degenerate `weight_words` explained the refutation, this wave assumed
`weight_words != 0` and the property proved. That was written down as "root cause
confirmed". An aliveness probe then showed `layer_valid` was *unreachable* under
the assumption. The same happened for `num_layers == 1`.

**133b. The mechanism, and it is general.** Under `-set-init-zero` every register
is zero at `t = 0`. An assumption of the form

```
always @(posedge clk) if (rst_n) assume (R == k);   // k != 0, R resets to 0
```

is contradicted at the first cycle where `rst_n` holds. No trace satisfies it.
Yosys then reports **"proof succeeded"** for every property in the run — with no
diagnostic, no warning, and exit code 0.

**133c. The decisive test, with a control.** `assert (1'b0)` **proves** under the
assumption and **refutes** without it. A literally false assertion passing is
proof that the trace set is empty and every result from that configuration is
meaningless.

**133d. Theorem (vacuity).** *Let `A` be an assumption set and `P` any property.
If `A` is unsatisfiable over traces of length ≤ n, then `A ⊨ P` holds for every
`P`, including `P = false`. Therefore the verdict "`P` proved under `A`" carries
information about `P` only if `A` is satisfiable.* The contrapositive is the
gate: refuting `assert(false)` witnesses a satisfying trace, which is exactly
satisfiability of `A`. One extra solver call decides it.

**133e. The production suite is live — now established rather than assumed.**
The emptiness probe refutes under all three configurations the suite is run
in (`T27_FORMAL`, `+DEEP`, `+OPEN`) at `seq 40`. The 30 integration properties
are non-vacuous. Before this wave that was an assumption about assumptions, held
for 130 propositions.

**133f. Gate 15 is the first gate here that runs the solver.** The other fourteen
read text, because the defects they catch are visible in source. This one cannot
be: it injects `assert (1'b0)` and fails the build if it proves. Verified against
all three bars — it passes clean, its probe anchor is asserted to match exactly
once, and it bites a planted unsatisfiable assumption in every configuration.

**133g. OPEN — the gate does not yet satisfy the starvation contract.**
`absence_sweep.py` reports it as a step that *crashes* when starved rather than
diagnosing the absence, which Prop. 116 set a ceiling of zero for. It is the only
gate here that invokes the solver, so it is also the only one whose starved run
is slow enough to be killed rather than answered. Recorded as failing, not
excluded from the sweep and not given a raised ceiling: a gate that cannot say
why it declined is exactly what Props. 103 and 116 were written about, and this
one is not exempt because I wrote it this wave.

**133h. This is Prop. 110's `unfaithful` category turned on the method.** A gate
can soundly decide `P′` while claiming `P`. Here the solver soundly decided
"`P` holds on the empty set" while I read "`P` holds". The taxonomy was written
for the artifact under test; it applies just as well to the instrument.

---

### Prop. 134 — the generated top has never been simulable, and Wave 665's measurement is not reproducible — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (all 28, tracker-backed included)*

**134a. Yosys resolves declare-after-use; Icarus rejects it.** `bitnet_engine_top`
reads `irq_status_w`, `cycles`, `layer_done_dly`, `layer_start_g`, and
`pf_overflow` in instantiations hundreds of lines above their declarations.
Fixing them one at a time produced a cascade rather than convergence.

**134b. The consequence is not cosmetic.** Every property in this campaign is
proved by yosys, which tolerates the ordering. Every *value* ever measured comes
from Icarus, which does not. A design that can be proved but not simulated is a
design whose control has been checked and whose arithmetic never has.

**134c. Wave 665's simulation result must be withdrawn as unreproduced.** That
wave reported `bram_we = 1` and an emitted trit of `TRIT_P` matching the
reference — described as the campaign's first agreement between an engine output
and a computed expectation. The harness copies from `build/rtl`, and `build/rtl`
as generated today does not compile under Icarus. The result is not refuted; it
is **unreproducible with the current tree**, which for a published measurement is
the same obligation. It is withdrawn pending a build that compiles.

**134d. The capture bug found while investigating stands on its own.** The
harness sampled `mac_result` under `mac_valid_q`, but `mac_valid_q` is the
compute stage's *input* valid — `.valid_in(mac_valid_q)` — while the requantizer
is fed `.valid_in(mac_valid_out), .acc(mac_result)`. It read the result one stage
before it existed. That fully explains the impossible pair reported last wave:
`acc = 0` beside an emitted `TRIT_P` at threshold 3. Corrected in
`sim/tb_data_check.v`; not yet exercised, because of 134a.

**134e. Two emitters produce the same module.** `gen-bitnet-bundle` and
`gen-bitnet-engine-top` both write `bitnet_engine_top.sv`, with different
declaration ordering. Whichever ran last decides what is verified.

---

### Prop. 135 — the first validated end-to-end value measurement — `MEASURED`

**Gate:** `formal-yosys.yml` → *Prove integration properties (all 28, tracker-backed included)*

**135a. Four nets stood between provable and simulable.** `pf_overflow`,
`ld_pipe`, `layer_start_g` and `mac_valid_q` were read by instantiations above
their declarations. Yosys resolves declare-after-use; Icarus rejects it. Hoisting
all four — splitting declaration from assignment where one carried an assignment —
made the generated top compile for the first time.

**135b. The measurement.** Reference: 27 lanes of `+1` against all-`+1` weights,
`acc = 27`, requantized at threshold 3 to `TRIT_P`. Engine: **`acc = 27`,
`trit = 2'b10`, `RESULT: MATCH`.** The weight path shows `bram_we = 1`,
`mem_rd_en = 3`, `prefetch_done` asserted.

**135c. Three bars, because a value check that cannot fail is worth nothing.**

- **TRUE** — engine and reference agree exactly.
- **ALIVE** — the MAC fired (`saw_mac`), weights were written, and the expected
  value is 27, which is neither zero nor any uninitialised register's contents.
  Prop. 127b was withdrawn precisely for reporting an initial value as agreement.
- **BITING** — perturbing the *reference* by `+1` while leaving the engine
  untouched yields `MAC MISMATCH engine=27 reference=28`. The check can fail.

A weaker control was run first and rejected: setting the weights to zero moved
engine and reference *together* (both to `acc = 0`, `TRIT_Z`), which demonstrates
responsiveness but not detection. Only the desynchronised control establishes
that the harness compares.

**135d. Wave 665's withdrawal is now resolved, and upward.** That result is
reproduced and extended: the accumulator agrees too, which it did not then,
because the harness had been sampling `mac_result` under the compute stage's
*input* valid. A withdrawal is not a retraction of the underlying fact — it is a
statement that the evidence was not reproducible. It now is.

**135e. Theorem (three-bar adequacy for a value check).** *A comparison of a
design output `d` against a reference `r` licenses the conclusion "the design
computes `r`" only if (i) `d = r`, (ii) the capture condition for `d` was
observed to fire and `r` is distinguishable from every default the capture could
hold, and (iii) there exists a perturbation of `r` alone for which the comparison
reports failure.* Dropping (ii) admits an unfired capture reading its
initialiser — Prop. 127b. Dropping (iii) admits a comparison that is not wired to
the design at all — Props. 121, 128. Conditions (ii) and (iii) are each one extra
simulation.

---

### Prop. 136 — the retroactive vacuity audit is clean, and it found twelve of its own false positives first — `MEASURED`

**Gate:** `formal-yosys.yml` → *Vacuity sweep — every proof step must refute assert(false)*

**136a. Every proof in this campaign predates the vacuity check.** Prop. 133 built
a gate for the engine suite. Twelve of the fifteen property wrappers use `assume`,
and none of them had ever been asked whether any trace satisfies its assumptions.

**136b. Gate 16 asks it once per step.** `formal/vacuity_sweep.py` parses every
`sat -verify -prove-asserts` invocation out of both formal workflows, re-runs it
verbatim with `assert (1'b0)` injected into its `-top` module, and requires a
refutation.

**136c. Result: 12 live, 0 vacuous, 6 not audited, of 18 proof steps.** No proof
in this campaign has been passing vacuously. The six are reported rather than
absorbed: four have a shell-variable `-top` this parser does not expand, one is
combinational and has no posedge clock to hang a probe on, and one is a template
naming `${mod}.sv`.

**136d. Its first run reported twelve vacuous steps, and every one was false.**
The workflow writes relative paths; the substitution was keyed on absolute ones,
so the probed copies were written and then not read. An unprobed suite proves —
so a probe that fails to land reports *the exact opposite of the truth*. The
contradiction that exposed it was `bitnet_engine_top` appearing as vacuous
minutes after `vacuity_gate.py` measured it live.

**136e. The shape was already in the catalogue, and in the sibling tool's own
assertions.** `vacuity_gate.py` asserts its probe anchor matches exactly once,
with the comment *"a probe that does not land tests nothing"*. The sweep, written
one wave later by the same hand, omitted that check. Prop. 103's first shape —
matching a form rather than a fact — does not stop applying because you wrote it
down.

**136f. Corollary (asymmetry of probe failure).** *An injected-probe audit whose
probe silently fails to land does not degrade to "no information": it inverts.
The unprobed artifact satisfies the probe's negation, so every step reports the
failing verdict.* Hence such an audit must assert delivery of the probe, not
merely its construction — an unverifiable step in a diagnostic tool is worse than
its absence, because it is read as data.

**136g. And once more, immediately.** Adding the comment stripper that
Prop. 136e's sibling gate demanded introduced a fresh false positive within one
command: detection moved to the stripped text while the insertion offset still
indexed the original, so the probe landed outside the module. Stripping shifts
every index after the first comment. Detection and offsets must not come from
different strings — a general hazard for any tool that sanitises text and then
edits by position.

**136h. Verified biting.** Planting `assume (1'b0)` in `double_buffer_props.sv`
moves the sweep to 11 live / 1 vacuous and names that step exactly; removing it
restores 12 / 0.

---

### Prop. 137 — the value check now sweeps, and one observable did all the work — `MEASURED`

**Gate:** `formal-yosys.yml` → *Value sweep — engine arithmetic against a reference, across configurations*

**137a. One measurement was one point.** Prop. 135 matched engine against
reference at `N=1, C=1, L=1` — which Prop. 125 had already identified as the
single configuration out of 81 that completes. The campaign's only arithmetic
result sat on the point least likely to be representative.

**137b. Gate 17 sweeps it.** `formal/value_sweep.py` parameterises the testbench
and runs the grid. Accumulators track the chunk count exactly: **27, 54, 81** for
`C = 1, 2, 3`, matching the reference in every configuration.

**137c. Eighteen configurations were three measurements.** The accumulator
observable depends only on `C`. Across the whole grid, `N` and `L` never moved
it — so a sweep that looked eighteen-wide was three distinct facts wearing
eighteen hats. **Breadth in a sweep is not independence**, and a grid that varies
parameters the observable cannot see reports coverage it does not have.

**137d. The fix was a second observable, not a bigger grid.** The requantizer
packs 27 trits per word, so a run of `L` layers of `N` neurons owes
`L × ceil(N/27)` activation words — a quantity that depends on both axes the
accumulator ignores. Adding it changed the sweep from three facts to a genuine
two-dimensional measurement, and it fired immediately.

**137e. The gate does not go red on known-open configurations.** It records a
baseline and fails only when a configuration that used to MATCH stops matching.
Prop. 26's expected-refutation convention exists because a gate that is red for a
known reason gets disabled rather than fixed.

---

### Prop. 138 — the six failures were an ill-posed question, and the check on the question was one line — `MEASURED`

**Gate:** `formal-yosys.yml` → *Value sweep — engine arithmetic against a reference, across configurations*

**138a. The finding, as it first appeared.** With the word-count observable,
six of eighteen configurations emitted one activation word where the reference
owed two. The pattern was sharp: every `C ≥ 2, L = 2` point failed and every
`C = 1` point passed. Both layers demonstrably ran — `start_prefetch = 2`,
`mac = 2` — so compute happened and only emission was lost. That is the exact
signature of the layer-end flush defects of Prop. 125, and it would have been
written up as the eighth design defect.

**138b. It was not a defect. The networks were ill-posed.** A multi-layer network
is only defined when layer 0 produces what layer 1 consumes: `N` neurons emit `N`
trits, and the next layer reads `C` chunks of 27, so `L > 1` requires
`N = C × 27`. The grid crossed `N ∈ {1,2,3}` with `C ∈ {1,2,3}` freely — asking
layer 1 to read 27 to 81 trits from a layer that produced 1 to 3.

**138c. The decisive test cost one command.** At `N = C × 27` the same
configurations emit **2, 4 and 6** words and all MATCH. The design was right; the
experiment was malformed.

**138d. Why `C = 1` masked it.** Those rows are equally ill-posed — layer 1 wants
27 trits from a layer producing 1 — yet they emitted the expected count. So the
apparent `C`-dependence was not a clue about the design at all; it was an
artefact of which malformed configurations happen to produce the right number of
words anyway. **A pattern in results from invalid inputs is still a pattern, and
it will look like a mechanism.**

**138e. Theorem (well-formedness precedes measurement).** *Let `M` be a
measurement defined on configurations satisfying a well-formedness predicate `W`.
For `x` with `¬W(x)`, `M(x)` is not evidence about the artifact — neither for nor
against — regardless of how systematically `M(x)` varies with `x`.* The
corollary is the working rule: **before reporting that a sweep found a defect,
verify that the failing configurations are ones the artifact was ever obliged to
handle.** Systematic variation across an invalid region is the most convincing
possible presentation of nothing.

**138f. What this cost and what it bought.** It cost one command. It bought not
publishing a fabricated defect — the fourth time in this campaign a new check
has fired on correct behaviour (Prop. 115), and the first time the false alarm
was in the *question* rather than the instrument.

---

### Prop. 139 — every proof step is now audited, and two properties were only repeating themselves — `MEASURED`

**Gate:** `formal-yosys.yml` → *Restatement scan — no property may merely repeat its RTL line*

**139a. The audit went from 12 steps to 28.** Prop. 136 left six steps
unaudited. Four drove `-top ${top}` from a shell loop, now expanded one top per
iteration; one is combinational and now gets a clockless `always @(*)` probe;
two were an artefact of a regex that swallowed the `;` separating yosys commands
into the module name. **28 live, 0 vacuous, 1 vacuous by design, 1 not audited,
of 30.**

**139b. The campaign already had a vacuity canary, and it was not enough.**
`assume_liveness_check.sv` assumes something unsatisfiable and asserts something
false, so it proves only when the flow honours assumptions — and it has been in
the workflow for many waves. The sweep flagged it, correctly, as vacuous. It is
now exempted with that argument recorded, and its vacuity is *enforced*: if the
canary ever refutes, the flow has stopped applying assumptions.

The lesson is about scope. That canary asks the question **once, globally, for
one job**. It cannot see a single wrapper whose own assumptions are mutually
contradictory while the flow at large is fine. A global liveness check and a
per-step one are different claims, and having the first is what made the absence
of the second invisible.

**139c. Gate 18 asks whether a property merely repeats its RTL line.**
`a_start_follows_ctrl_unless_interlocked` asserts the exact right-hand side of
the `assign start` above it. In Wave 666 I added a term to that assignment and
edited the property in the same commit — which felt like verification and was
bookkeeping. Such a property is refutable only by an inconsistent edit.

**139d. `mirror_check.py` sounds like this gate and is not.** It compares the
ternary algebra abstraction against `trit_stdlib.sv`. Nothing had ever asked the
restatement question, and the similar name is exactly why: a check whose name
suggests coverage it does not provide is worse than no check, because it stops
the question being asked.

**139e. Two found, both kept, both now annotated.** The scan reports 36 equality
assertions and flags 2. Both are retained with `// restatement: <reason>` stating
what they are for — one guards against a CSR aperture instantiated and then
ignored (how `use_buffer_a` was dead for four waves), the other is the regression
witness for the two-memory-port split. **Acknowledgement is not vindication**: it
makes a deliberate choice countable, so a future wave can ask whether the count
is growing.

---

### Prop. 140 — 139 propositions never touched the boundary — `MEASURED`

**Gate:** `formal-yosys.yml` → *Value sweep — engine arithmetic against a reference, across configurations*

**140a. Shape had been swept; values never had.** Every vector in this campaign
was all-`+1` inputs against all-`+1` weights. The accumulator is then `27C` and
the trit always `TRIT_P`. A sign error, a lane transposition, a wrong trit decode
and an inverted comparison all survive that vector in every configuration —
Prop. 137's twelve-point grid included.

**140b. Randomised trits, drawn reproducibly.** A written-out xorshift behind
`T27_SEED`, with seed 0 preserving the historical vector so nothing already
measured is disturbed. Across twelve seeds the reference reaches `acc ∈ [-3, 27]`
and all three trit values.

**140c. Two seeds disagreed, at exactly one point.** Seeds 5 and 7 both land on
`acc = -3` with `threshold = 3`. The design emits `TRIT_N`; the testbench
reference said `TRIT_Z`. The design's chain is inclusive — `acc >= threshold`,
`acc <= -threshold` — and the reference had been written independently with `>`
and `<`. They agree everywhere except `acc = ±threshold`.

**140d. The design wins, and the reason is not "it is the design".** No `.t27`
spec governs the requantizer's boundary, so there is no authority above the two
implementations. The design's convention is stated twice — as a documented
priority chain in the RTL and as `activation_requant`'s own inline properties —
while the reference agreed with neither. Had the intended semantics been
exclusive, this same evidence would have condemned the design; recording which
way the evidence pointed matters more than which side won.

**140e. Why 139 propositions could not see it.** A boundary disagreement is
visible only from the boundary. The all-`+1` vector produces `acc = 27C ≥ 27`,
never within 24 of the threshold at any swept configuration. Twenty-nine
integration properties constrain control and are blind to it by construction
(Prop. 81b); the value check existed and used the one vector that cannot reach it.

**140f. Theorem (adequacy needs value coverage, not configuration coverage).**
*Let `f` be a design and `r` a reference, and let `D` be the set of inputs a test
suite applies. If every `d ∈ D` maps to the same point of `f`'s output partition,
then the suite distinguishes `f` from `r` only if they differ on that point —
regardless of how many configurations `D` is replicated across.* Sweeping shape
replicates `D`; it does not enlarge it. The corollary is operational: **a suite
whose expected output is constant is testing one thing, and its size is
decoration.**

**140g. Boundary cases now travel with the sweep.** Seeds 5 and 7 are pinned into
the grid at two shapes, so the `±threshold` case is exercised on every run rather
than rediscovered.

---

### Prop. 141 — every proof step is audited, and the last two were exempt for opposite reasons — `PROVEN`

**Gate:** `formal-yosys.yml` → *Vacuity sweep — every proof step must refute assert(false)*

**141a. 30 of 30.** **28 live, 0 vacuous, 1 vacuous by design, 1 immune by
construction, 0 not audited.** Prop. 136 started at 12 audited of 18 known.

**141b. The two non-audited steps are not the same kind of thing, and collapsing
them would have been the error.** `assume_liveness_check` is *designed* to be
vacuous — it assumes something unsatisfiable and asserts something false, so it
proves only when the flow honours assumptions at all. "Properties are non-vacuous
(witnesses must refute)" is *immune*: its pass condition is a refutation, and
vacuity makes refutation impossible, so an empty trace set can only turn it red.
One is an exemption granted by argument; the other needs no exemption because the
hazard cannot reach it.

**141c. Both are enforced rather than merely listed.** If the canary stops being
vacuous the sweep fails, because the flow has stopped applying assumptions. If
the immune step disappears from the workflows the sweep fails, because a stale
exemption reads exactly like coverage.

---

### Prop. 142 — "20 of 28" counted twelve configurations that were never defined — `CORRECTED`

**Gate:** `formal-yosys.yml` → *Value sweep — engine arithmetic against a reference, across configurations*

**142a. The claim, and what was wrong with it.** Prop. 125 reported that the
harness "terminates cleanly with the done IRQ on 20 of 28 configurations of the
repaired variant" — a grid of `N ∈ {1,2,8,26,27,28,54} × C ∈ {1,2} × L ∈ {1,2}`.
Prop. 138 established that `L > 1` requires `N = C × 27`. Of the fourteen `L = 2`
points, exactly **two** satisfy that. **Twelve of the twenty-eight were ill-posed,
and whether they terminated is not evidence about the design.**

**142b. What survives untouched.** Prop. 125's headline — *exactly one
configuration in 81 works* — is unaffected. That sweep is `N = 0…80` at `C = 1`,
`L = 1`, and well-formedness is vacuous at `L = 1`: a single-layer network has no
successor to be consistent with. A retroactive check that clears a prior claim is
worth as much as one that condemns it, and this one does both in the same
paragraph.

**142c. The well-formed subset, re-measured on values.** All sixteen well-formed
configurations of that grid — fourteen at `L = 1`, two at `L = 2` — now **MATCH**
on accumulator, trit, and emitted-word count. `N = 26`, which Prop. 125 recorded
as producing zero activation words for twenty-six computed neurons, emits exactly
one; `N = 28` and `N = 54` emit two; the `L = 2` pair emit two and four.

**142d. This replaces a control-flow figure with an arithmetic one.** "20 of 28"
counted *terminations*. Sixteen of sixteen counts *correct answers*. The first
was contaminated by ill-posed points and measured only that the machine stopped;
the second is smaller, well-posed, and measures what the machine computed.

**142e. Corollary (an ill-posed point is not a conservative one).** *Including
configurations outside a design's contract does not make a pass-rate pessimistic;
it makes it uninterpretable.* Twelve ill-posed points could have inflated or
deflated "20 of 28" depending on which way their undefined behaviour fell — and
in Prop. 138 the analogous points did both, failing at `C ≥ 2` and passing at
`C = 1` for no reason connected to the design.

---

### Prop. 143 — 497 specs parse, and the parser discards 3292 declarations — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**143a. The wave began by writing a spec and finding it empty.** Prop. 140 had
to settle a semantic question with no `.t27` spec to appeal to, so this wave
wrote one. `t27c parse` returned exit 0 and an AST containing **none** of its
identifiers — 16 nodes, an empty `Module`. Exit 0 over an empty set.

**143b. The control located it in the corpus, not the file.** The repository's
flagship spec, `gamma_conjecture.t27`, has 14 constant declarations in source
and **3** in its AST. Across 43 specs: 676 written, 214 captured — **31%**.
Across all 497: **3292 constants never reach any AST** *(WITHDRAWN — Prop. 149: this figure counted function-local bindings and missed every `pub const`)*, and every spec exits 0.

**143c. The mechanism is four lines.** `parse_module_body` recovers from a
failed declaration by calling `skip_to_next_top_level()` and continuing:

```rust
Err(_) => {
    // On parse error, skip to next top-level declaration and continue
    self.skip_to_next_top_level();
}
```

Recovery is correct for a resilient parser. **Discarding the error is not.**
No diagnostic, no counter, no effect on the exit code — so "496/496 specs
parse" has always meant "the parser did not abort", never "the parser read the
spec". This is Prop. 103's second shape, *a decline that is not counted*, in
the compiler for the project's stated source of truth.

**143d. The AST is consumed.** `codegen_python.rs`, `formula_eval.rs` and
`compiler.rs` all read it, so whatever is dropped is dropped for every
downstream consumer, silently.

**143e. The fix is to make the silence audible, not to rewrite the parser.**
`Parser::discarded` records each recovered error; `parse_ast_reporting` returns
them; `t27c parse` prints `recovery-events: N` to stderr with the first five
messages and their line numbers. The first one on `ternary_encoding.t27` is
`Unexpected top-level token: Semicolon (';') at line 16:30` — a precise,
actionable defect that had been invisible for the life of the repository.
Rewriting the parser late in a session, to serve a measurement, is the change
this campaign has twice recorded as the one not to make.

**143f. Theorem (recovery hides loss monotonically).** *Let a parser `P` map
source to an AST with an error-recovery relation that, on failure of
declaration `d`, discards `d` and resumes at some later point. If `P`'s exit
status is a function of only the returned AST's well-formedness, then for any
source `s` and any prefix-preserving corruption `s'` of `s`, `P(s')` and `P(s)`
are indistinguishable by exit status.* Corollary: **an exit code cannot report
recovery, so any parser with silent recovery requires a second channel or it
reports success on arbitrarily corrupted input.**

---

### Prop. 144 — recovery events are not the declarations they discard — `CORRECTED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**144a. I shipped the count under the wrong name.** The stderr line was
`discarded-declarations: N`, and it counts **recovery events**. One call to
`skip_to_next_top_level()` can swallow several declarations, so the two
quantities differ.

**144b. A planted regression exposed it, by not moving.** Three constants added
to a spec left the "discarded-declarations" total at exactly 1741, so the gate
built to catch regressions reported none. The constants-captured measurement
moved 15 → 18 for that file. Had the label been believed, the gate would have
been a ratchet on a number the regression could not touch.

**144c. Both are now ratcheted, and neither is called the other.**
`recovery-events` counts errors recovered; a separate figure counts constants
written minus constants reaching the AST. The gate fails if either rises for
any spec, and reports both totals every run.

**144d. This is the campaign's most-repeated failure, committed again.** Not a
wrong number — an unexamined label. `FAIL: 496` meant "binary not found".
`58 empty/skipped` meant "object-shaped". `20 of 28` meant "including twelve
ill-posed". Now `1741 discarded declarations` meant "1741 recovery events".
Every instance was an accurate count of something other than what its word
said, and every one survived until something forced the two apart.

---

### Prop. 145 — the requantizer boundary is now written down — `RECORDED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**145a.** `specs/numeric/requant_boundary.t27` records the inclusive threshold
convention Prop. 140 had to adjudicate without a specification: the priority
chain, the reason it is a chain rather than parallel comparisons, the
verification record (26 configurations, boundary reached, `acc ∈ [-6, 81]`),
the resolved finding, and the scope limits.

**145b. Its `provenance` is `PRE_RULE`, and that matters.** The value was fixed
before the spec existed, stated in the RTL and asserted by the module's own
inline properties. **Recording it is a record, not independent evidence** — a
spec that agrees with the artifact it was written from confirms nothing about
the artifact.

**145c. The open question is written as open.** Whether *inclusive* is the
intended semantics or merely the implemented one is not settled by anything in
this repository, and is marked `DO_NOT_GUESS` rather than resolved by the fact
that the design does it.

**145d. Its first draft did not parse, and that is how Prop. 143 was found.**
The syntax recorded in the project's own spec-authoring guidance —
`spec Name version X.Y.Z` with `rule { }` blocks — produces an empty AST. The
form the compiler accepts is `module Name { }` with consts and functions. A
documented syntax that the toolchain does not implement is the same shape as a
documented command nobody has run.

---

### Prop. 146 — one unconsumed semicolon cost 1259 recovery events — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**146a. The defect.** `parse_const_decl`'s `= value` branch returns without
consuming the trailing `;`. The stray semicolon becomes the next
`parse_top_level_decl`'s current token, which errors, and recovery discards
whatever follows. The sibling branch for bracket-valued constants consumes it
correctly — which is exactly why the omission survived: **some constants parsed,
so nothing looked broken.**

**146b. The measurement.** Recovery events **1741 → 556**; specs recovering
**427 → 205**; constants never reaching an AST **3292 → 2339** *(WITHDRAWN — Prop. 149)*. One line.

**146c. The ratchet is what made the fix scorable.** Prop. 143 built the counter
before there was anything to count. Without it "I fixed a parser bug" is a claim;
with it the fix has a number, in the direction the gate enforces, measured by the
same instrument that will catch its regression.

**146d. Corollary (a partial-success defect is the hardest to see).** *An error
that destroys some inputs and not others produces a system that appears to work.*
A parser that dropped every constant would have been fixed the day it shipped.
One that drops 69% of them ships for the life of the repository, because every
spot check finds a constant that parsed.

---

### Prop. 147 — a Unicode-to-ASCII commit replaced 162167 characters with a running counter — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**147a. Found by reading a parse error, not by looking for corruption.** With
Prop. 143's counter reporting, the top remaining error on `ternary_encoding.t27`
was `Expected LBrace, got Number ('257')` — on a line reading
`fn bit_to_trit_pair(bit: u8) 257 [2]i32 {`. A bare number where a return arrow
belongs.

**147b. Byte-level attribution.** Commit `fcf80027d`, *"fix(l3-purity): replace
all Unicode with ASCII in 160 .t27 files"*:

```
before:  ) â     Trit      # U+2192 RIGHTWARDS ARROW
after:   )    1   2         Trit      # the character's running index
```

`git log -S` for the ASCII form finds nothing, so the arrow was **never** `->`.
The script enumerated non-ASCII characters and substituted each occurrence's
**index** in place of a transliteration.

**147c. Scale.** 154 `.t27` files, **112 distinct non-ASCII characters**,
**162167 occurrences**. `═` accounts for 155801 (comment banners, which is why
`gamma_conjecture.t27` carries lines reading `123456789101112...` — the digits
are consecutive indices). `→` accounts for 677, and those are the ones inside
function signatures.

**147d. The two defects concealed each other exactly.** The corruption produced
parse errors; the silent recovery of Prop. 143 swallowed them. Neither was
visible while the other stood, and the repository reported 497/497 specs parsing
throughout. **A defect that generates errors is invisible behind a defect that
discards errors** — and the pairing is not a coincidence, because a corpus whose
errors are all swallowed accumulates corruption without resistance.

**147e. Repair, scoped and verified.** 483 arrow sites restored across 37 files,
reconstructed from `fcf80027d^` rather than guessed from patterns: for each line
the pre-image's `→` positions decide where `->` goes, and the result must equal
the pre-image with `→` transliterated or the line is left alone. Recovery events
**556 → 482**. The other 111 characters — Greek letters, box drawing, math
symbols, all in prose — are **reported and not touched**; restoring them is a
transliteration decision about 161490 characters, not a mechanical repair.

**147f. 62 specs could not receive their repair.** They carry uncommitted
working-tree modifications predating this session, so the corruption commit's
pre-image is not an authority on their current content and the repair cannot be
separated from the existing edits. Only the 27 specs whose diff is provably a
pure number→arrow substitution were committed. **The standing dirty-tree
question now has a cost attached to it.**

**147g. Theorem (error-channel occlusion).** *Let `D₁` be a defect whose only
observable is an error on channel `c`, and `D₂` a defect that discards channel
`c`. Then in any system containing both, neither is observable, and removing
`D₂` alone reveals `D₁` at full magnitude while removing `D₁` alone reveals
nothing.* The asymmetry is the actionable part: **fix the discarding defect
first**, because it is the one whose repair converts hidden state into evidence.

---

### Prop. 148 — `#` was documented as a comment and never lexed — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**148a.** The lexer skips `//`, `/* */`, and `;` at column 1. It has never
handled `#`, which the project's own spec-authoring guidance documents alongside
`//`. 199 comment lines across the corpus were parsed as declarations.

**148b. Scoped, because `#` is not only a comment.** It also opens raw string
literals (`r#"..."#`). The rule is line-initial `#` only, matching the existing
`;` precedent — and verified first: **0** line-initial `#` in the corpus look
like syntax rather than prose.

**148c. Measured: 482 → 474 recovery events.** Eight, not the 57 the earlier
clustering suggested — because `t27c parse` prints only the **first five**
messages per file, so every cluster count in Prop. 147's investigation was over a
truncated sample. **A truncated sample of errors is not a census of causes**, and
it was read as one.

---

### Prop. 149 — the constants-lost metric is withdrawn; no regex formulation is sound — `WITHDRAWN`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**149a. What was published.** Prop. 143 reported *3292 constants never reach any
AST* and Prop. 146 reported that falling to *2339*. Both figures came from
`^\s*const\s+\w+` against `kind: ConstDecl`.

**149b. That regex measured the wrong set, in both directions at once.** It
required `const` at line start, so it **missed every `pub const`** — the actual
module-level declarations — and instead matched `const bit = ...`, which are
**function-local bindings** that were never meant to be module-level nodes.

**149c. Three formulations, three different answers.**

| formulation | result |
|---|---|
| `^\s*const\s+\w+` (shipped) | 2339 lost |
| brace-depth ≤ 1 | 2444 lost — but array literals `[32]u16{` open braces, so the depth accounting drifts |
| `^\s*pub\s+const` | ratio **118%** — the AST has more nodes than the marker, because non-`pub` module constants also produce them |

**149d. The reason there is no sound regex.** `const` is legal at module scope
**and** inside a function body, and separating them requires parsing — which is
the thing being measured. **A metric that needs the artifact it is auditing is
not an external check.** Only the parser can report this.

**149e. Withdrawn, not weakened.** The gate now ratchets on recovery events
alone, which the parser itself emits and which is sound. The constants figure is
struck from Props. 143 and 146 rather than restated with a better regex.

**149f. Third instance in the same instrument.** Prop. 144 corrected
`discarded-declarations` (recovery events mislabelled as declarations); this
corrects the other half of the same gate. The gate was built to catch a compiler
that reports success while reading 31% of its input, and it shipped with two
labels that did not describe what they counted. **The failure being audited and
the failure of the audit were the same shape.**

---

### Prop. 150 — 162167 characters transliterated; 61 Coq proofs are deleted and unadjudicated — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**150a. The transliteration Prop. 147 declined to make.** A 148-entry table
covering **162123 of 162167** occurrences (99%) — box drawing to `-`/`=`/`|`,
arrows to `->`, Greek to names, superscripts to `^N`, math operators to ASCII.
1882 lines across 130 files, each reconstructed from `fcf80027d^` and accepted
only if it equals the pre-image transliterated. The remaining 44 occurrences are
Coptic and Cyrillic in prose, where a mapping would be a guess: **17 lines
skipped rather than guessed.**

**150b. The result is legible.** `// Strand I 0 Loop Quantum Gravity` becomes
`// Strand I -- Loop Quantum Gravity`; a comment reading
`// 12345678910111213...` becomes a `-----` rule. The digits were consecutive
indices standing in for `─`.

**150c. 112 files committed, 55 held back.** The held-back set carries
uncommitted edits that make the repair inseparable from them — the same
constraint as Prop. 147f, now quantified against a larger change.

**150d. Classifying the working tree, which the constraint finally justified.**

| count | state | class |
|---|---|---|
| 72 | modified | `.t27` specs — a decision |
| **61** | **deleted** | ~~Coq proofs under `coq/Kernel/`~~ *(WRONG — Prop. 151: they are generated `specs/fpga/*.v`)* |
| 5 | modified | Rust/Zig source — a decision |
| 4 | deleted | `__pycache__` — safe to delete |
| 2 | modified | `Cargo.toml`/`Cargo.lock` — review |
| 1 | deleted | generated Verilog |

**150e. WITHDRAWN — see Prop. 151. The 61 deletions are generated Verilog, not proofs.** *(original text follows)* **The 61 deletions are the item that should not have been sitting in a
working tree.** They are machine-checked proof files, removed but never
committed, so they exist in `HEAD` and not on disk. Nothing in this repository
would notice: no gate reads `coq/`, and the formal workflows are yosys-based.
**A deletion that is never committed is invisible to every check that reads the
committed tree and to every check that reads the working tree, because each sees
a consistent world.**

---

### Prop. 151 — the "61 deleted Coq proofs" were generated Verilog, and the deletion is correct — `CORRECTED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**151a. What was published.** Prop. 150d/e reported 61 deleted files under
`coq/Kernel/` — machine-checked proofs, removed on disk and present in `HEAD`.
It was the headline of that wave's tree classification.

**151b. It was a bug in the classifier, of a shape already in the catalogue.**
The counter was keyed on `(state, class)` while the example dictionary was keyed
on `class` alone, so examples leaked across states: a *modified* `coq/` file was
printed as the exemplar for a *deletion* bucket. `git status` shows `coq/` as
` M`, and `coq/` holds 15 files in `HEAD` against 11 on disk — a discrepancy
worth its own look, and not 61 deletions.

**151c. What the 61 actually are.** `specs/fpga/*.v`, each carrying the header
*"Generated from t27 spec: <Name> / DO NOT EDIT - generated by t27c
gen-verilog"*. Generated output committed into the source-of-truth directory.

**151d. Verified safe before committing the deletion.** All 61 have **both** a
source `.t27` beside them **and** a regenerated copy under `gen/verilog/`.
**Zero would be lost.** Committed, along with 4 `__pycache__` files and one more
generated Verilog file with the same property — 15143 lines out of `specs/`.

**151e. The lesson is narrow and repeats.** Aggregate by a compound key, then
illustrate by a component of it, and the illustration will contradict the
aggregate eventually. Both halves were mine, one wave apart.

---

### Prop. 152 — recovery skipped past every module-level `const`, by design — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**152a. Prop. 149 said only the parser could report this. It now does.**
`skip_to_next_top_level` counts the declaration keywords it passes over. Tokens
inside brace groups are consumed by `skip_brace_body` and never reach that loop,
so **function-local bindings are excluded by construction** rather than by the
heuristic that made the previous metric unsound.

**152b. The first version over-counted, and said so immediately.** It recorded
the keyword the loop *stops* at as swallowed — 28 reported on a file with zero
recovery events. Counting only what is passed over fixed it. A counter placed
before its own exit test measures its terminator.

**152c. The mechanism, stated in the code's own comment.** `is_top_level_start`
**deliberately excludes `const` and `var`**, "which can appear inside
keyword-style test/invariant/bench blocks". Correct for those callers. Shared
with the error-recovery caller it means recovery **skips past every module-level
`const` until it finds a `fn`, `pub`, `struct`, `enum` or `test`.** That is why
`pub const` survived — `KwPub` stops the skip — and bare `const` did not.

**152d. Fixed at the call site, not in the predicate.** `skip_to_next_decl`
stops at `const`/`var`; the keyword-style callers keep the conservative skip.
Recovery events rose **474 → 523**, which is the *correct* direction: recovery
now stops at each declaration instead of one event swallowing a run of them.
ConstDecl nodes across the corpus rose to **1965**.

**152e. The ratchet is now sound in both components.** 523 recovery events and
**788 declarations swallowed**, both emitted by the parser.

**152f. Theorem (shared-predicate scope error).** *Let a predicate `P` be
conservative for caller `A` and exact for caller `B`. Sharing it makes `B`'s
behaviour silently incomplete on exactly the inputs where `A` needs the
conservatism.* The failure is invisible because `P` is correct — for `A`. The
repair is never to change `P`, but to parameterise it at the call site, and the
tell is a caller whose correctness argument differs from the predicate's comment.

**152g. And it vindicated the withdrawal completely.** `gf16.t27` — the file the
withdrawn metric ranked as the corpus's worst, at *640 constants lost* — has 20
`pub const` in source, **20 ConstDecl nodes in its AST, 0 recovery events and 0
declarations swallowed.** It is parsed perfectly. All 669 of its bare `const`
are function-local bindings. The worst-looking file in the corpus was never
losing anything.

---

### Prop. 153 — the blocked specs, classified — `MEASURED`

**Gate:** `formal-yosys.yml` → *Spec parse gate — "parses OK" must mean the parser read the spec*

**153a.** 72 `.t27` files carried uncommitted edits that prevented two waves of
verified repairs from landing. Classified by diff shape: 51 substantive, 13
line-for-line substitutions, 8 comments-only.

**153b. 16 are provably my own repair.** Tested against the same pre-image
oracle used to make them — a line is accepted only if it equals the corruption
commit's pre-image transliterated. Those are committed, leaving **56** that need
a human decision rather than an oracle.

**153c. The number that matters is 56, not 72.** An "unreviewed changes" pile
shrinks when you can prove which parts of it you wrote. The rest is genuinely
someone's unfinished work and no automated test can adjudicate it.

---

### Prop. 154 — 123 `Qed.` are type-checked by nothing — `MEASURED`

**Gate:** `formal-yosys.yml` → *Coq build scan — a Qed in a file nobody compiles is not a proof*

**154a. The README's count is exactly right and is not what it reads as.**
*"546 Qed. across 41 files"* reproduces to the unit. **69 of those 546, in 7
files, appear in no `_CoqProject`** — so `coq_makefile` never generates a rule
for them, `make` never compiles them, and neither Coq CI job type-checks them.
Two of the seven carry headers, uncommitted until this wave, stating in capitals
that they do not compile and are *"research notes, not machine-checked proofs"*.

**154b. Across all three proof trees.** Including `proofs/`, which the README's
command does not scan: **59 `.v` files, 560 `Qed.` inside a build, 123 outside
one.** 18% of the proof terminators in this repository are in files nothing
compiles.

**154c. `grep -c 'Qed\.'` measures terminators in text.** Only membership in a
`_CoqProject` measures proofs. This is the campaign's most-repeated shape — an
accurate count of a different denominator — now found in the README's own
headline evidence, and the fourth instance after Props. 116b, 142 and 149.

**154d. The exclusion itself is correct.** `_CoqProject` lists 9 of `coq/`'s 11
files, and the 2 omitted are exactly the two whose headers say they do not
compile. Nobody made a mistake in the build; the mistake is a published count
that ranges over a wider set than the build does.

**154e. Gate 20 requires each file to be built or to say it is not.** A `.v`
file must appear in its tree's `_CoqProject`, or carry a marker naming itself
unverified. It **ratchets** rather than walling: whether a given unbuilt proof
*should* be added to a build is a mathematical judgement about that file, and a
gate landing red on 17 pre-existing files gets disabled rather than obeyed
(Prop. 26). Verified biting: a planted `.v` file fails it by name, and removing
it restores the hold.

**154f. Comment-stripped, because the shape recurs.** Coq block comments are
`(* ... *)` and a `Qed.` inside one is not a terminator. Five fixes across four
files have gone to unstripped comments in this campaign, so this counter strips
before matching and says why.

**154g. What was NOT committed, and why.** `PhiAttractor.v` also carries
uncommitted changes — but they *remove* four proof-bearing lines and add
`(* TODO: requires domain-specific contraction analysis *)`. That is someone
mid-proof, weakening a file, not an honesty annotation. Committing it would have
silently reduced verified content under cover of a wave about honesty. Left in
the working tree and named here instead.

**154h. Corollary (evidence counted outside its own gate).** *A quantity cited as
evidence for a property `P` must range over exactly the artifacts some check
establishes `P` for. Where the citation's domain strictly contains the check's,
the excess is presented as evidence and supported by nothing.* The excess is
invisible precisely because the count is correct — and the check for it is
mechanical: compare the citation's glob against the gate's input set.

---

### Prop. 155 — `coq/` was never missing anything, and a third filter mismatch — `CORRECTED`

**Gate:** `formal-yosys.yml` → *Coq build scan — a Qed in a file nobody compiles is not a proof*

**155a.** Prop. 151 recorded a residual concern: `coq/` holds "15 files in `HEAD`
against 11 on disk". Enumerated, both are **15**, byte-identical file lists.

**155b. The discrepancy was two filters compared as one.** `git ls-tree -r HEAD
--name-only coq/` counts all 15 entries including `.gitignore`, `README.md`,
`_CoqProject` and `.CoqMakefile.d`; `find coq -name "*.v"` counts the 11 proof
files. Neither number was wrong; comparing them was.

**155c. Third instance of the same error in this campaign, all mine.** Prop. 149
compared a regex over `const` against AST `ConstDecl` nodes; Prop. 151 compared a
counter keyed on `(state, class)` against examples keyed on `class`; this
compared a tree listing against a filename glob. **The recurring form is that
both sides are computed correctly, so nothing looks wrong** — and the only
defence is stating the set each side ranges over before subtracting.

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
