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

**Gate:** `formal-yosys.yml` → *Prove layer_sequencer properties* and *Zero-sized requests complete without pretending*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prop. 39e is still open (must refute)* and *Baseline - unprobed design must prove*

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

**Gate:** `formal-yosys.yml` → *Zero-sized requests complete without pretending*

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

**Gate:** `formal-yosys.yml` → *Every proposition names its gate, every block runs*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties* and *Oversized requests do not wrap the local address*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*, *No property is gated as an expected refutation*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*, *Prop. 39e is still open (must refute)*

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

**Gate:** `formal-yosys.yml` → *Engine is still alive under its interlocks*, *Prop. 39e is still open (must refute)*

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

**Gate:** `formal-yosys.yml` → *Prop. 39e is still open (must refute)*

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

**Gate:** `formal-yosys.yml` → *Prop. 39e is still open (must refute)*

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

**Gate:** `formal-yosys.yml` → *Prop. 39e is still open (must refute)*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*, *Prop. 39e is still open (must refute)*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*, *No property is gated as an expected refutation*

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

**Gate:** `formal-yosys.yml` → *Prove bitnet_engine_top integration properties*

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

**Gate:** `formal-yosys.yml` → *Prove integration properties (core 22, deep bound)* and *(all 26, tracker-backed included)*

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

**Gate:** `formal-yosys.yml` → *Properties are non-vacuous*; `formal-mutation.yml` → *Baseline, control, and mutation*

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
