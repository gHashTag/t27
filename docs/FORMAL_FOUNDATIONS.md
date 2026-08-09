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

### Prop. 38 — the MAC is 8× of the solve cost, and it is the one thing that cannot be scaled — `MEASURED`

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

Whether the fault is the engine or the tracking registers is **not established**
— the counterexample has not been read, and two earlier attempts at
counter/address relations in this campaign were wrong in the property rather
than the design. Gated as an expected refutation so closing it turns the build
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
