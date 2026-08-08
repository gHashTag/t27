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

---

**φ² + 1/φ² = 3 | TRINITY**
