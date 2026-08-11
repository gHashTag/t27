# Trinity S³AI DNA -- t27 -- TRI-27 Spec-First Language

[![CI](https://img.shields.io/github/actions/workflow/status/gHashTag/t27/ci.yml?branch=master&logo=github&label=CI)](https://github.com/gHashTag/t27/actions/workflows/ci.yml)
[![Zenodo](https://zenodo.org/badge/DOI/10.5281/zenodo.19456875.svg)](https://doi.org/10.5281/zenodo.19456875)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Version: 0.1.0](https://img.shields.io/badge/version-0.1.0-orange.svg)](https://github.com/gHashTag/t27/releases)

**Language:** [English](README.md) | [Русский](docs/README_RU.md)

> **Canonical Zenodo SOT:** [zenodo.org/communities/trinity-s3ai](https://zenodo.org/communities/trinity-s3ai/). The GoldenFloat badge above (19456875) is a legitimate Vasilev deposit but lives **outside** the curated S³AI v5.0 record set; see [docs/ZENODO.md](docs/ZENODO.md) for the canonical 12-record bundle and aliases.

The canonical source of truth for Trinity S3AI.
`.t27` specs in → Zig, Verilog, C out.

**φ² + 1/φ² = 3 | TRINITY**

---

## What this repo is

**t27** is the **spec-first toolchain and numeric format registry** for the
**TRI-NET line** of open high-assurance ternary AI silicon. The primary
product of t27 is the path `.t27 → Verilog RTL → Tiny Tapeout` with sealed,
inspectable artefacts at every step.

- **Enable the gates (one command per clone):**
  `cd bootstrap && cargo build --release && cd .. && ./target/release/t27c install-hooks`
  — points `core.hooksPath` at the tracked `.githooks/`. Gate logic lives in
  `bootstrap/src/hooks.rs` (Rust, unit-tested), not in shell. Without this a
  fresh clone runs **no** hooks; git does not enable them automatically.
- **How to verify:** `cd bootstrap && cargo build --release && cd .. && cargo test --release`
  → **1213 / 1213 passed** (full Quick Start below).
  Validators: `./scripts/tri validate-conformance`, `validate-gen-headers`, and
  `seal-audit --strict` — all green as of the 2026-08-09 seal re-baseline.
- **Primary numeric path:** GoldenFloat **GF16** (default), with the family
  GF4–GF32 registered in [`conformance/FORMAT-SPEC-001.json`](conformance/FORMAT-SPEC-001.json).
  FP8 compat and NF4 / INT4 / INT8 quant bridges are **planned**, not shipped.
  Full details: [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md).
- **Readiness:** [`STATUS.md`](STATUS.md) records what is at SPEC / RTL / SIM /
  SYNTH / GDS / SILICON, conservatively, from this repo's own evidence only.
- **Sibling chip repos (separate):** `tt-trinity-phi` (1×1 φ-anchor),
  `tt-trinity-euler` (8×2 e-engine, safety/control), `tt-trinity-gamma`
  (8×4 γ-surface 32-PE ternary mesh). Tape-out target:
  [Tiny Tapeout](https://tinytapeout.com/chips/). See [`LINEUP.md`](LINEUP.md).
- **Positioning:** [`COMPETITORS.md`](COMPETITORS.md) — we do not race
  commercial NPUs on TOPS or SDK breadth. We also do **not** claim to own the
  formal corner outright: [Vericert](https://github.com/ymherklotz/vericert),
  [Kami](https://github.com/mit-plv/kami), and
  [Amaranth](https://amaranth-lang.org/docs/amaranth/latest/) are ahead of us
  on verified compilation and built-in formal flow
  ([`COMPETITORS.md` §2](COMPETITORS.md#2-adjacent-toolchains----the-corner-we-actually-compete-in)).
  Our narrowest defensible claim is the **machine-checkable tape-out
  conformance gate** (`tt-manifest` → `tt-profile` → `tt-conform`).
  Benchmark policy: [`BENCHMARKS.md`](BENCHMARKS.md).
- **CLARA traceability:** [`CLARA_TRACEABILITY.md`](CLARA_TRACEABILITY.md).
- **Activation width is now a port, not an assumption:**
  `t27c gen-activation-requant` emits the layer-boundary requantizer that was
  missing entirely — `signed [15:0]` accumulator → packed trits, with a
  host-programmed dead-zone. **This module's output port is where the
  ternary-vs-4-bit fork lives**; a 4-bit variant changes `trit [1:0]` to
  `act [3:0]` and nothing else in the datapath moves.
- **Where the BitNet premise holds and where it does not:**
  [`docs/BITNET_V2_POSITION.md`](docs/BITNET_V2_POSITION.md) — ternary *weights*
  are validated by BitNet v2, not superseded by it; but this datapath also makes
  *activations* ternary, which is more aggressive than any published BitNet
  variant on the axis the field finds hardest, and there is no requantization
  stage at the layer boundary.
- **Formal foundations:** [`docs/FORMAL_FOUNDATIONS.md`](docs/FORMAL_FOUNDATIONS.md)
  — numbered propositions with an explicit evidence class (`PROVED` /
  `MEASURED` / `CONJECTURE`), including the seal-path injectivity result *and
  its counterexample*, the measured boundary of Yosys's SVA support, and a
  verified Yosys-only proof pipeline.

---

## System Status

| Domain | Component | Status | Details |
|--------|-----------|--------|---------|
| Compiler | `t27c parse` | GREEN | **496 / 496** specs parse (measured 2026-08-09) |
| Compiler | `t27c gen-verilog` | GREEN | 5/5 FPGA modules synthesize |
| Compiler | `t27c seal` | GREEN | injective seal paths; **496** files in `.trinity/seals/`, all verifying |
| FPGA | Yosys synthesis | GREEN | 5/5 modules pass synth_xilinx |
| FPGA | E2E bitstream | GREEN | Yosys→nextpnr→prjxray→.bit (zero Vivado) |
| FPGA | Board profiles | GREEN | QMTECH XC7A100T (minimal+full), Arty A7 |
| FPGA | `--profile` flag | GREEN | `--profile minimal|full` in fpga-build |
| Pins | Pins IR | GREEN | `specs/pins/ir.t27` — conflict detection invariants |
| Pins | XDC emitter | GREEN | `specs/pins/emitter_xdc.t27` — QMTECH + Arty presets |
| CI | Issue gate | GREEN | L1 TRACEABILITY enforced — greps PR title/body for `Closes #N` |
| CI | Seal **presence** | GREEN | **496** seal files for **496** specs — one each, no orphans |
| CI | Seal **integrity** | GREEN | **496 / 496 verify** (re-baselined 2026-08-09); `seal-coverage` CI is **enforcing**. `t27c seal-audit --strict` |
| CI | Formal (Yosys) | GREEN | **89 properties proved** across 14 modules — the 43rd closes a defect class that had gone six waves without one: `a_writes_within_request` says the DMA never writes more words than the request covers, which is exactly what Prop. 29 fixed and nothing had constrained since. It **detects 13 of the 64** behaviourally-real mutations that suite missed, the largest bite of any property here — and re-measuring afterwards corrected the denominator too: Prop. 61 counted gaps against `dma_props` alone while **three** wrappers constrain that module, and the sibling suites turn out to catch 8 more, so the true remaining gap is **43, not 51 or 64**. The residue is now flat — 42 distinct lines for 51 mutants, 33 of them singletons — so cluster-and-write has extracted what it can from this module; continuing would mean one property per mutation, which is a restatement of the RTL rather than a specification ([Prop. 72](docs/FORMAL_FOUNDATIONS.md)). The property and it took two counterexamples to state correctly — the second showing that a 12-byte request legitimately occupies two words, so the property was wrong about the design's contract rather than the design wrong about the property ([Prop. 71](docs/FORMAL_FOUNDATIONS.md)) — and as of Wave 617 that is true, which it had not been: **eight zero-size properties were counted here while no job in this repository ran them**. `zero_size_props.sv` appeared exactly once in all of `.github/`, inside the *weekly* mutation harness as gate definitions for two of its four wrappers; two wrappers appeared nowhere at all. Nothing was broken — all eight hold — which is precisely why it went unnoticed: an ungated property that happens to hold looks exactly like a gated one until someone counts the steps. Four of them are **expected refutations**, so a step that expects everything to prove could not gate them, and awkward-to-gate is how something ends up ungated ([Prop. 69](docs/FORMAL_FOUNDATIONS.md)). That was found by accident, so the systematic check now ships: `formal/orphan_scan.py` cross-references every property file against every workflow and **errors if nothing runs it** — it found a second orphan on its first run, an 88-line AXI4 read-slave model that a later wave had unknowingly rebuilt a weaker version of inline. It now also maps every **emitted module**, per module rather than per file: of **23** in the bundle, 22 have properties of their own and **0 are constrained only at one remove** through the engine -- Wave 618 measured 8 direct and 8 indirect, and the map is now closed, **0 ternary primitives are instantiated by nothing at all** while being read into every proof, and one emits concurrent SVA this flow cannot check ([Prop. 76](docs/FORMAL_FOUNDATIONS.md)). The module implementing the double-buffer ping-pong — source of the campaign's longest-running defect, three changes across eight waves — is in the second group: it has never had a property of its own. It is now wired into `dma_props`, validated on three bars — the suite proves, all five activities stay reachable, and the model's own single-burst precondition (which it *asserts* rather than assumes) proves against this master ([Prop. 70](docs/FORMAL_FOUNDATIONS.md)). Briefly 41, because one of them turned out never to have read the design: it referenced `dut.word_index`, a hierarchical reference this flow does not support, so yosys implicitly declared an undriven one-bit wire of that name and proved the property against it for four waves. It was deleted rather than patched, then replaced in the next wave by one stated in ports and backed by an AXI-slave environment — which had to clear three bars, not one: it **proves**, the assumption did not buy that by making the design idle (five reachability probes, all still refuting), and it **detects two behaviourally-real mutations the whole suite had missed**. `formal/phantom_scan.py` now fails the build on the two warnings that were there all along ([Props. 62–63](docs/FORMAL_FOUNDATIONS.md)) + **28 integration properties** on `bitnet_engine_top`, all proving **at `-seq 40`, `DEPTH 4`** — and the claim carries its ceiling: 22 of the 26 prove at **`-seq 80`** and all 26 at `-seq 40`, both gated — four properties need ten formal-only tracking registers and cost 75% of the proof time, so they run behind their own define ([Props. 53–55](docs/FORMAL_FOUNDATIONS.md)). **No engine property is gated as an expected refutation**, so nothing in the engine is knowingly broken — the gate for that scans all 23 emitted and property sources for `T27_FORMAL_OPEN`. Four *module-level* properties are deliberate expected refutations and always were: `*_never_completes` records that a zero-sized job **does** report done, which is safe only because its sibling `*_emits_no_work` proves it did not pretend to have done anything ([Props. 26, 65](docs/FORMAL_FOUNDATIONS.md)). That is a recorded design decision, not a knowingly broken property, and it is stated here because the sentence above used to read as covering them. Includes cross-layer and write-contiguity properties, zero- and maximum-size sweeps, a baseline gate, **14 liveness witnesses** — every module probed for all three shapes: that its core activity happens at all, that activities *overlap*, and that they *repeat* (two transfers, two interrupts serviced, two write transactions, two layer runs), each one validated by a constraint that removes exactly what it probes and makes it prove ([Props. 51, 56, 57](docs/FORMAL_FOUNDATIONS.md)) — **0 vacuous guards**, a documentation gate covering all **127 propositions**, a validated counterexample reader, and weekly mutation and scale-ceiling harnesses ([Props. 14–58](docs/FORMAL_FOUNDATIONS.md)). A real arithmetic defect was found in Wave 628 by the first *exhaustive* proof of a combinational primitive — `adder_tree_27` returned −14 where the balanced sum is +2, a four-bit accumulator too narrow for its own documented range — and re-running every engine-level proof on the corrected RTL moved **nothing**: the 28 integration properties proved both before and after, because they constrain *control* while the defect was in *data* ([Props. 80–81](docs/FORMAL_FOUNDATIONS.md)). That defect was never hidden: the RTL's own comment read `range [-9, +9] -> signed [3:0]`, stating the correct range on the line above the width that cannot hold it, and a unit test asserted the wrong width verbatim — so it was not merely untested but protected, for 595 waves, because nothing ever compared the two numbers. Now something does: `formal/width_scan.py` checks every documented range against its declaration and every reduction's operands against what the target is declared *and documented* to hold. Its first draft reported zero findings on the injected defect as well as on the shipped tree — an eight-line comment block outran a three-line lookahead, and a `+` inside an array index made an operand count disagree with a term count, so the check silently declined to run on the very tree it was written for and still printed clean; the summary now reports how many reductions it actually checked, and zero is a failure ([Prop. 82](docs/FORMAL_FOUNDATIONS.md)). Auditing the suite's 36 width pins for others that could be wrong found none stale, but one pointed at a question nothing answered — **is the MAC's 16-bit accumulator wide enough?** The four properties on that datapath could not have said: `result == $past(result) + $past(dot)` is a *16-bit* equation, satisfied exactly by an accumulator that wraps. And the module cannot answer it alone — it has no chunk counter and no `num_chunks` input, so in isolation it overflows after **1214** chunks. It is safe only through a contract written nowhere in the tree: `layer_sequencer` walks `chunk_id` over an 8-bit port, so at most 255 chunks separate two restarts and 255 × 27 = 6885 fits. Widening `num_chunks` for larger layers — an ordinary change to another file — silently reintroduces the wrap. The bound is now proved by **k-induction**, which is the only honest way to state it: the overflow is 1214 cycles out, so every feasible depth would report "proves" and mean nothing ([Prop. 83](docs/FORMAL_FOUNDATIONS.md)). That turned out to be a class rather than an incident, so every growing register in the bundle is now mapped to whatever limits it: of **15**, four are bounded locally, four by an input port, and **seven by nothing inside their own module at all** — and each must now carry a `// BOUND:` note giving the argument, which proves nothing safe but makes a *missing* argument visible. Writing those fifteen notes surfaced two clamps that are tight to the bit (both 12-bit word indices sized at exactly their 4096-entry limit, where one more entry wraps and no comparison in either module would say so) and one 32-bit AXI address register where its siblings are 64-bit, close enough to the 4 GiB ceiling to matter on a real memory map. The scan's own first draft read `<=` as a comparison rather than the nonblocking assignment it is, and so classified the Prop. 83 accumulator — the register the whole sweep exists because of — as bounded by a contract when it is bounded by nothing ([Prop. 84](docs/FORMAL_FOUNDATIONS.md)). Sweeping the other direction found the three **countdowns** those tight bounds actually rest on, and a countdown has the mirror failure mode: `X <= X - k` wraps to near 2ᴺ the moment `X < k`, and a wrapped countdown does not stop — it runs another 2ᴺ steps past the request. `weight_prefetch_ctrl`'s is now proved by k-induction to reach 0 only as the FSM leaves, so the verdict covers every request length rather than a depth; it clears all three bars, including **biting** the off-by-one terminator that would produce the underflow. It had to be stated *inline in the module*, because the register is internal and a wrapper referencing it would have proved against an undriven phantom wire rather than erroring — sometimes the right place for a property is not the property file. The DMA's byte countdown, by contrast, **underflows by design**: a 12-byte request goes 12 → 4 → `0xFFFFFFFC`, which is harmless for two separate reasons — the exit test sits in the same always block and samples the pre-decrement value, while `beats_owed` is a continuous assignment that *does* see the wrap but is only read in states the FSM no longer re-enters — and only while the slave honours the `arlen` it was issued — an environment dependency, recorded as one rather than dressed up as a verified property. Guarding those two module assertions cost more than expected: with the same define the engine's integration steps pass, they silently joined the engine's obligation set and took its cheapest step from **153 s to 241 s** — 1.58×, +88 s from two properties — *(superseded: a later paired re-measurement with disjoint ranges put the same comparison at **0.82×, 26 s faster**, retiring this figure's stated reason; see Prop. 98 below)* — so they now sit behind their own define and the engine keeps its 31 `$check` cells. That figure is itself a correction: it was first published as 4× from measurements taken while three other provers were competing for the machine, and the clean re-run of the *no-properties* case came in at 153 s, faster than the 183 s baseline it was supposedly a regression against, which is what exposed it. A timing figure is a claim about a machine state — record the state or do not publish the number — an inline property is compiled by whoever passes its guard, not by whoever wrote it ([Prop. 85](docs/FORMAL_FOUNDATIONS.md)). The six primitives Prop. 76 left **UNREACHED** for five waves are now answered a third way — not retired, not wired in, but recognised as an **algebra** and proved outright: `not` is negation and an involution, `and`/`or` are min/max so the triple is a **De Morgan (Kleene) algebra** satisfying ¬(a∧b) = ¬a∨¬b, `multiply` is the product with 0 absorbing and the units closed, `compare` is sgn(a−b), and `trit3_add` satisfies val(sum) + 27·val(cout) = val(a) + val(b) over all 4096 input pairs — every one exhaustive at `-seq 1`, with no depth caveat. That closes the coverage map: **22 of 23 modules directly constrained, 0 unreached**, the one exemption being concurrent SVA this flow cannot parse. The theorem that earned its place was `compare`, which is right *only because* the two-bit encoding happens to be monotone in trit value — and testing that by permuting the encoding broke a second module nobody predicted: `trit_full_adder` had the encoding baked in as literals where every sibling, including its own half-adder instances, went through the named constants, so a renumbering would have moved them and left it behind silently. Fixed, and the fix verified by re-running the same experiment ([Prop. 86](docs/FORMAL_FOUNDATIONS.md)). Timings finally got the provenance the proofs always had: `formal/bench.py` runs both arms alternating on one machine, records load and competing provers, and **refuses to print a ratio** when the machine was contended or the arms' observed ranges overlap. Its first real use returned 0.88× — two extra properties making a proof *faster* — which was not a discovery but the harness reporting that the RTL had been regenerated a third of the way through the run; it now fingerprints the files under test and rejects a comparison whose inputs moved underneath it ([Prop. 87](docs/FORMAL_FOUNDATIONS.md)). And the one hand-argument Prop. 85 left standing is now a proof: the DMA's byte countdown wraps deliberately, so the claim is not that it never wraps but that **wherever it is consumed it is still a sane residue** — false in isolation, and true under the AXI read-slave model written eighteen waves earlier, which supplies exactly the `arlen`-compliance the argument rested on. Bounded at `seq 24` (285 s); at `seq 80` it did not complete in 30 minutes and is recorded as not completed rather than retried until it produced a number ([Prop. 88](docs/FORMAL_FOUNDATIONS.md)). Two lemmas now sit under T5 — `val(sum) + 3·val(carry) = val(a) + val(b)` for the half adder and its three-input analogue for the full adder — so that a future failure **localises**: if the tree's equation breaks while both lemmas hold, the arithmetic is right and the wiring is wrong. The full adder's carry is the non-obvious part, exact only because its two internal half-adder carries can never both fire with the same sign. And the first thing that lemma caught was **itself**: its third assertion was written as a rounding formula and refuted, because Verilog's `%` takes the sign of its dividend — the adder was correct and the specification was not, which is the mirror of Prop. 80 and worth recording in both directions ([Prop. 89](docs/FORMAL_FOUNDATIONS.md)). The encoding-permutation experiment that found the Wave 634 defect is now a standing gate, checked **both ways**: no theorem may newly break under a permuted encoding, *and* the one that is encoding-dependent by design must still break — because a gate asserting only "nothing broke" passes the moment its own perturbation becomes a no-op. Nine theorems permuted across 18 localparam sites, zero disagreements, and a self-test that re-injects the exact Wave 634 defect and confirms it is caught ([Prop. 90](docs/FORMAL_FOUNDATIONS.md)). Turning the new harness on the campaign's own scale-ceiling numbers then **withdrew a conclusion**: the two engine steps re-measure at **154.5 s** and **309.9 s** against the published 183 s and 422 s — 16% and 27% lower, on a described machine with disjoint ranges and an unchanged input fingerprint. Prop. 81d had inferred that headroom under the ceiling was narrowing from a 238 s → 422 s comparison; the 422 endpoint is now known to be 27% high and the 238 endpoint describes a 22-property configuration that no longer exists and cannot be re-measured. That inference is retired rather than restated with a smaller coefficient, leaving a baseline that a future wave can actually compare against ([Prop. 91](docs/FORMAL_FOUNDATIONS.md)). The sentence "T5 follows from F by the positional argument" was itself doing real work in a comment, so it is now discharged: `fv_abstract_fa` is a full adder about which **nothing** is known except lemma F — its outputs are free signals constrained only by the conservation equation — and chaining three of them proves balanced addition for *any* F-satisfying adder, making T5-on-the-real-tree a corollary of two proved facts rather than three separate proofs happening to agree. Its non-vacuity is gated by an oracle that must refute, and its one real weakness is named rather than hidden: the abstraction **duplicates** `trit3_add`'s wiring rather than sharing it, so a rewiring of the real tree would leave it behind while the proof kept passing about a circuit no longer in the bundle. `formal/mirror_check.py` pins the two together port by port ([Prop. 92](docs/FORMAL_FOUNDATIONS.md)). That proposition was then **adversarially reviewed before it had been reviewed by anyone**, and the theorem survived while **four of the claims built around it did not**: the vacuity oracle was defeated by an injection that shrank the covered input space to 5.9% while leaving every gate green; the newest theorem was absent from the encoding gate's table *and* outside the reach of its permutation, so a supposedly semantics-preserving relabelling broke it for a reason that was the experiment's fault rather than the design's; `mirror_check` compared connection **text**, so the same identifier holding different values on each side read as identical — "read the declaration, not the use", a rule this campaign wrote down four waves earlier, broken by the gate written to enforce a mirror; and the "every module satisfying F at once" generality is empty, because F plus trit-validity determines the adder **uniquely** and the class has one element. All four are fixed, with lemma F now written once and shared so the guard and the abstraction cannot drift. The lesson is the review itself: Prop. 92 cleared three bars I designed and named, and everything wrong with it lay outside them — **bars you choose yourself test what you thought of** ([Prop. 93](docs/FORMAL_FOUNDATIONS.md)). The same review turned on the campaign's timings found the problem to be structural: of roughly **60 quoted durations**, **none is guarded** — `claims_check` deliberately polices only facts about the tree, and a duration is a fact about a run — and Prop. 91's parting assertion that one withdrawn inference was "the only one" resting on them was itself unaudited and wrong. At least **five further live inferences** stand on unreproducible seconds, including a 436× cost-spread whose cheap endpoint is a property deleted four waves earlier, a standing recommendation built on a premise its own campaign later corrected 8× → 1.5×, and the **1.58× that moved code behind a separate guard and is quoted in this README** — whose expensive endpoint has never once been reproduced. Two further ratios are unsound in kind rather than merely unprovenanced, dividing a completion by a *timeout*. None of the underlying conclusions is claimed to be wrong; what is established is narrower and worse — **their evidence is not re-derivable** ([Prop. 94](docs/FORMAL_FOUNDATIONS.md)). Turning the same adversarial sweep on the campaign's **existing** gates then found two that were not checking what they claimed. The liveness step injects its reachability probe before the *last* `endmodule` in a file — so when Wave 633 appended two modules to `pipeline_stage2_props.sv`, all four `ps2_props` probes began landing in the wrong module, were pruned as unused, and the unprobed suite proved; the step reads that as "unreachable" and **has been failing for three waves with a message naming the wrong cause**. And `bound_scan` was crediting **formal assertions** as design bounds — `chunk_addr` read as *bounded in-module* on the strength of an `assert` inside `` `ifdef T27_FORMAL ``, which is a claim *about* the design rather than a mechanism constraining it. That inverted the gate's purpose on **three of the four LOCAL verdicts in the whole bundle**, and excluding formal regions leaves exactly **one** genuine local bound in the emitted design — while surfacing a real unstated contract, since `chunk_addr` indexes a 4096-deep BRAM and advances `num_neurons × num_chunks` times with nothing anywhere keeping that product under 4096 ([Prop. 95](docs/FORMAL_FOUNDATIONS.md)). And the flag every module suite is proved with, `-set-init-zero`, has been described since Prop. 8c as "starting from a reachable state" — it starts from the **zero** state, which equals the reset state only where every register resets to zero, and **nine here do not** (four FSMs to `IDLE`, a buffer select to 1, three AXI ready lines high, a trit to `TRIT_Z`). This is *not* an unsoundness — extra unreachable states yield spurious refutations, never false proofs, so nothing verified is weakened — but it is an invisible fragility: renumbering an FSM so any **decoded** state lands on code 0, a pure relabelling since every reference is by name, refutes `a_rready_implies_burst` and `a_rready_implies_active` outright, and the failure would read as a design defect. A local instance of this was found and fixed for a single property many waves ago, with an `fv_started` guard whose comment states the cause exactly — and nobody asked how many other registers reset non-zero. The answer was nine, and they are now listed ([Prop. 96](docs/FORMAL_FOUNDATIONS.md)). Attempting to re-measure the campaign's last unreproduced live inference — the **1.58×** that moved code behind a guard and is quoted in this README — produced no number and a defect instead: the configuration it measured now **refutes in 11 s**, because a define created for unconditionally-true properties later gained an **environment-dependent** one. `a_drain_sane_where_consumed` is false without the AXI read-slave model that supplies `arlen` compliance, exactly as Prop. 88b stated, and the engine has no such model — so a guard that reads as a category silently meant "drain properties, one of which needs a slave model". Split into its own define, both sides verified. The 1.58× is therefore not wrong but **permanently uncheckable**, its configuration having ceased to exist; the design decision it justified stands on an argument that never needed it ([Prop. 97](docs/FORMAL_FOUNDATIONS.md)). An adversarial audit of the remaining gates then confirmed **four more** defects of the same family — a gate matching text that merely *looks like* what it means to check. `phantom_scan`, whose sole purpose is catching the undriven wire a property once proved against for four waves, **missed every such wire wider than one bit**, because yosys words that warning with a bit index and the pattern's character class could not cross it; all four of its self-test injections were identifiers yosys declares as a single bit, so the test only ever exercised the form that worked. `width_scan` deduped reductions by target name and so **never examined 2 of the 5 in the bundle** — both inside the very module it was written for — while that same set was its coverage counter, making the summary read as full coverage; a range comment placed *after* its declaration deleted that declaration from the gate's view entirely, taking a provably broken adder from exit 1 to exit 0; and an unannotated operand fell back to the worst-case-by-width rule this project established is unsound for ternary, producing a **false finding against correct RTL**. All fixed, with the injections kept as permanent self-tests ([Prop. 98](docs/FORMAL_FOUNDATIONS.md)). Finally, with the guard split in place, the drain properties turn out to make the engine proof **0.82× — 26 s faster** (three paired runs, disjoint ranges, stable inputs): an easy assertion acts as a lemma. That removes the *stated* reason for the Wave-633 split while leaving the split correct for the other reason given at the time — worth noticing, because next time the evaporated justification might have been the only one ([Prop. 99](docs/FORMAL_FOUNDATIONS.md)). The audit's full report then turned out to contain **six** defects for that one gate where the summary I acted on showed four — and both extras **survived** the first round of fixes: a constant addend and *any* subtraction made the reduction check decline in silence while the coverage counter still read full, so `l1[0] + l1[1] + l1[2] + 5'sd9` and `l1[0] - l1[1] - … - l1[5]` both overflow their declaration with the gate reporting "0 carrying less", exit 0. The loop now splits expressions into **signed terms**, resolves literals, and counts anything unresolvable as *uncheckable*; and the guard that tripped only at exactly zero is now a **floor**, since tripping at zero is precisely how three separate defects stayed invisible. The lesson is about the reporting rather than the gate: I read four findings off a truncated notification and shipped, while two more sat in the full result on disk — **a summary of an adversarial review is not the review** ([Prop. 100](docs/FORMAL_FOUNDATIONS.md)). That mechanism — a matcher declining a form it cannot handle while the coverage figure still reads full — was then swept across all ten gates, every bare `continue` asked whether it means *not my subject* or *my subject, which I could not check*. Eight were clean (recorded, so the sweep is not repeated). `doc_gate` was silently exempting any fence containing `<foo>` as a template, and `absence_sweep` was silently dropping **6** builder steps — the same lie its own comment warns against, committed one exclusion class over. Both now name what they skip. The rule, for reuse: **a gate's summary must report what it did not check as prominently as what it did**, because "0 problems" over an unstated number of declines is the same sentence as "0 problems" over none — and four defects have now lived in exactly that gap ([Prop. 101](docs/FORMAL_FOUNDATIONS.md)). A second audit round — `orphan_scan`, never reviewed, plus the four gates changed the day before — returned **25 verified findings**, three of them confirmed and *all three in code less than 48 hours old*: `4'b101` was read as **one hundred and one** because the literal parser took every sized literal as decimal and ignored its base; `strip_formal` deleted real design, removing both `` `ifndef T27_FORMAL `` bodies and the `` `else `` branches of formal guards; and `orphan_scan` counted assertion labels inside **comments** — the identical defect fixed in its sibling `claims_check` one wave earlier, with the identical regex, which nobody thought to grep for. One reported finding did **not** reproduce and is recorded as such ([Prop. 102](docs/FORMAL_FOUNDATIONS.md)). With enough instances to generalise, the campaign's gate defects now form a **taxonomy of five shapes** — matching a form rather than a fact (9), a decline that is not counted (4), reading a claim as the design (3), targeting by position rather than by name (2), and a guard that trips only at zero (3) — with three structural regularities: **the self-test never catches these** (it is written by the gate's author from the model that produced the defect), defects cluster in the newest code, and the same defect recurs in sibling files. The prediction is falsifiable and stated before the next audit: a sixth shape would mean the taxonomy is incomplete ([Prop. 103](docs/FORMAL_FOUNDATIONS.md)). Auditing `orphan_scan` — written in Wave 618 precisely because eight property files turned out never to be run — showed it had **never checked that anything runs**. Its stated job is to cross-reference every property file against every workflow; it asked instead whether the filename appears anywhere in the workflow *text*. A `#` comment, a step carrying `if: false`, a `grep` that reads the file and proves nothing, and a workflow triggered `on: [release]` all counted as "run" — verified with a file whose property is provably false. And the hazard was **live on the file the gate exists because of**: `formal-yosys.yml` already carries two retrospective comments naming `zero_size_props.sv`, so deleting only its executable references leaves the summary byte-identical to a healthy tree. The comments narrating that defect would have concealed its recurrence. The check now searches only the `run:` bodies of reachable steps, with comments stripped, and requires the file to be an argument to something that could actually prove it ([Prop. 104](docs/FORMAL_FOUNDATIONS.md)). Rather than wait for the next audit to notice the next sibling, the tree was **grepped for each shape's signature**: a third instance of the comment-counting defect turned up in `scale_probe`, which enumerates assertion labels over raw source from the very file carrying a comment that quotes one — and a latent instance of the position-targeting defect sat in `phantom_scan`'s own self-test, working only because its victim file happens to have one module today. Six signatures over 15 files gave 33 candidates of which **two** were real, so a grep is a lead generator rather than a verdict — but it cost a minute and reached two defects that four million subagent tokens of auditing had not. An audit discovers *new* shapes; a grep propagates *known* ones, and the cheap one should run immediately after every fix ([Prop. 105](docs/FORMAL_FOUNDATIONS.md)). The taxonomy then did the one thing a taxonomy can usefully do: **it was falsified**. Prop. 103b had staked that a further audit would find only shapes 1–5; five never-reviewed gates were attacked with agents told a sixth shape was *more valuable* than confirming the five, and they found **two**. Shape 6 is **sampling a time-varying property at its boundaries** — the timing harness sampled competing provers once before and once after each run, so a prover that started and finished inside the run was invisible, and it fingerprinted its inputs once around the whole sequence, so a file that changed between repeats and **reverted** read as stable. The check observed the right thing with the right threshold and was blind only to the interval between its observations. Shape 7 is **over-detection** — every earlier shape describes a gate failing to fire when it should, and a gate failing a *correct* artifact is the mirror image; it already had an instance, mis-filed under shape 1. Five other claimed novelties were **not** new and are recorded as such, because a taxonomy that absorbs every finding predicts nothing. A prediction that survives tells you little — stating the boundary before looking, and having it broken in the first round, is the only part of this that was ever evidence ([Prop. 106](docs/FORMAL_FOUNDATIONS.md)). Three further gates then failed audit. **26% of the campaign's own `**Gate:**` citations named CI steps that do not exist** — `doc_gate` checked only that the line was *present*, never that the step it named was real, and 27 of 104 pointed at nothing ([Prop. 107](docs/FORMAL_FOUNDATIONS.md)). `mirror_check` compared two circuits by identifier *text* while each file defined the constant differently ([Prop. 108](docs/FORMAL_FOUNDATIONS.md)). And the load-bearing gate — the one certifying that every checking step fails when starved, which is what licenses reading any of their greens as evidence — was moving the gate **scripts** aside along with the design, so every python step failed with *"No such file"* and was recorded as failing correctly. For a quarter of the swept steps the only thing established was that deleting a script breaks the step that runs it ([Prop. 109](docs/FORMAL_FOUNDATIONS.md)). With ~35 confirmed instances the list of shapes resolves into structure. A gate is a decision procedure, and it can be wrong in **three independent ways**: *unsound* (passes an artifact violating the property — ~28 instances, and **all five** catalogued shapes are mechanisms of this one), *incomplete* (fails a correct artifact — 3), or **unfaithful** (soundly and completely decides some `P′` while its documentation claims `P` — 4). That the shapes are all unsoundness mechanisms is a fact about **how this campaign has been looking**, not about gates: every audit was instructed to find gates that pass when they should fail. The unfaithful category is the one adversarial testing *cannot* find — in all four cases the instrument answered correctly every time while the sentence describing it named a different question, which is why one such error stood twelve waves with the harness green throughout. Adversarial agent review, the technique behind ~28 of these findings, is a **soundness instrument**; run alone it drives unsoundness toward zero and leaves every caption untouched ([Prop. 110](docs/FORMAL_FOUNDATIONS.md)). The first instrument for that category now exists — a gate must name, in its own docstring, every path it **mutates** — and the interesting result is its limit. It would **not** have caught Prop. 109: a retroactive test written to show the opposite briefly appeared to pass, because the reconstruction had mangled the docstring it was meant to preserve, and repairing it turned the result negative. The sweep's docstring *did* declare `formal/`; what went unnoticed was the consequence — that emptying it also removes the instruments — and no path-level check can see that. Its first version also over-detected **24 times on a clean tree** by demanding that every path a gate *reads* appear verbatim in prose, and two further narrowings were needed after that: Prop. 110's prediction that an instrument pointed at a new category would keep meeting over-detection held three times inside a single file ([Prop. 111](docs/FORMAL_FOUNDATIONS.md)). The second projection — **scope** — found a live instance immediately: this README stated *"all 37 checking steps"* (gated, correct) and, four hundred words later, *"all forty CI steps"* (ungated, wrong; the sweep walks 41 and checks 37). Both describe the same sweep, and a gate matching one phrasing sees only that phrasing. Registering the synonym turned out to be the wrong fix — a claims pattern demands it *match*, so it would forbid ever rephrasing the sentence, and removing the numeric wording immediately tripped the UNMET guard. The check is the inverse: for a quantity the tree already knows, **no other numeric claim about it may appear unregistered** ([Prop. 112](docs/FORMAL_FOUNDATIONS.md)). The third projection — **provenance** — found a live defect before its gate was written. This README asserted a figure (*"1.58×, +88 s from two properties"*) that a paired re-measurement three thousand words later had already superseded (*"0.82× — 26 s faster, three paired runs, disjoint ranges"*), with nothing at the first sentence to warn a reader they were reading a retracted number — Prop. 81d's shape, a withdrawal recorded far from the claim it withdraws. The rule that now holds: FORMAL_FOUNDATIONS propositions are *dated records*, so a duration there is historical by construction, but README is the *current-state* document, so every duration here must be **traceable** — carrying either a provenance marker or a proposition citation. 15 durations, 0 untraceable, and an injected bare timing is caught. Three of the category's four members now have a mechanical check; the fourth is a noun-phrase mismatch with no countable projection, which is why it survived twelve waves ([Prop. 113](docs/FORMAL_FOUNDATIONS.md)). Then the sweep's own backlog produced the sharpest finding of the campaign so far: **two CI steps were already broken in normal operation**, and the sweep read both as *"fails, correct"* — because a step that is already broken also fails when starved. `Prove zero-size properties` carried a stray third element in a tuple list the loop unpacked as two, so it raised `ValueError` after two wrappers and **four of the eight zero-size properties were never proved** — the very suite whose unrun properties Prop. 69 was about. And the mutation harness named emitter text that no longer exists, so its target appeared **zero times**, the mutation was never applied, and the suite **silently tested 7 of 8 mutants**. A negative control licenses nothing alone: *fails when starved* and *works when fed* are two claims, and only the first was ever asked. `absence_sweep --positive` now runs every step against an intact tree ([Prop. 114](docs/FORMAL_FOUNDATIONS.md)). A census of the opposite failure then settled Prop. 110's open question: **all ten gates over-detect** — failing a correct artifact on some semantics-preserving change — against unsoundness found in six of ten across ten days. Incompleteness was in every gate and found in a single pass, because nobody had asked; the five catalogued shapes are unsoundness mechanisms precisely because every audit was instructed to look for unsoundness ([Prop. 115](docs/FORMAL_FOUNDATIONS.md)). Then the sweep's own verdict turned out to be **the sign of an exit code**: a missing binary, an unrelated crash and a hang were all printed as a healthy gate. Classifying what every step actually emitted when starved gives **9 diagnosed against 28 indeterminate** — so *"0 passing on nothing"* was true and nearly vacuous. Nine steps demonstrate they read their subject; twenty-eight demonstrate only that they fell over, which is precisely how Prop. 114's two broken steps hid. The count is now published and **ratcheted** rather than enforced, because failing all 28 today would take the gate out of service — the mechanism by which an incomplete gate becomes an unsound one ([Prop. 116](docs/FORMAL_FOUNDATIONS.md)). **That 9-of-37 figure was then found to be wrong** — and it was the classifier, not the suite. All 28 supposedly-silent steps name the exact missing file in the tool's own words (`ERROR: File 'build/rtl/x.sv' not found`, `FileNotFoundError: ... 'formal/x.sv'`); the classifier had looked only for this repository's own `::error::` convention and scored yosys's and Python's perfectly clear messages as silence. Corrected: **37 diagnosed, 0 indeterminate**, with the criterion now being Prop. 114's actual question — does the failure name a *starved path*, which tells it apart from a step that was simply broken? Checked against seven cases including both Prop. 114 defects, 7/7, and the ceiling is now an enforced 0 rather than a ratchet at 28. This is the fourth consecutive wave in which a new instrument over-detected on first use, and the first in which it did so inside **a number that was published** — a wrong gate fails loudly, a wrong measurement propagates ([Prop. 117](docs/FORMAL_FOUNDATIONS.md)). **Six of the ten over-detections are now fixed**, and every one of them had been rejecting this repository's own conventions: a signed literal `16'sd0` as a reset value, a re-aligned column in a shell script, a `**Gate:**` line indented two spaces (byte-identical rendered HTML), a backticked identifier in a `// BOUND:` note — the quoting style the gate's own error messages use — a comment *inside* an assertion body explaining why that assertion is **not** a self-comparison, and a retrospective note saying an open-guard had been removed. **Three of the six were the same mistake**: matching text inside comments, which is now five instances of one shape across four files, each found separately rather than by grepping after the first. A gate written from an author's mental model encodes that model's blind spots, and the author's own idioms are precisely what it fails to anticipate — they were invisible while writing it. Four remain, each needing more than a character and each recorded rather than quietly deferred ([Prop. 118](docs/FORMAL_FOUNDATIONS.md)). The shape behind three of those six — **a regex applied to raw Verilog, matching inside a `//` comment** — has now cost five separate fixes across four files, every one found on its own. `formal/comment_scan.py` closes the class: a gate that reads Verilog and applies a regex must strip comments first **or declare in writing why it does not**. Four gates read comments on purpose and now say so, which is the interesting half — the marker forces that question to be answered once, where a reader can check it, instead of being rediscovered by a defect. It over-detected on its own first run (a gate whose stripper is named `code_mask` was reported as having none), for the fifth consecutive wave. What this buys is not the five fixes, which were already made: it is that a **sixth** instance now fails the build instead of being found by whatever it breaks ([Prop. 119](docs/FORMAL_FOUNDATIONS.md)). The first adversarial pass at the **design** since Prop. 80 — twelve days having gone entirely into instruments — then returned a confirmed defect. `activation_requant` packs 27 neuron results per word and raises `word_valid` **only** at a full word: there is no flush, so a layer whose neuron count is not a multiple of 27 leaves its last `num_neurons mod 27` results in a partial word that is never emitted, and nothing constrains `num_neurons` to a multiple of 27. The module's own property `a_word_only_on_full` **asserts** that behaviour — the gap is encoded as intended, precisely the Wave 628 shape where a defect was not merely untested but *protected* by something asserting it. The Props. 84/95b annotation compounded it by stating `ceil(num_neurons / 27)` where the RTL does `floor`: two readings one file apart disagreeing about the design, with `ceil` being what it was *intended* to do. And no property caught it because dropping a layer's last 26 neurons is a **data** loss that leaves every handshake correct — Prop. 81b's control/data boundary, from the other side ([Prop. 120](docs/FORMAL_FOUNDATIONS.md)). The refutation phase then confirmed **five** design defects, every one reachable in the assembled engine — the largest single result of the campaign, and the first about the design rather than the tooling. Beyond the missing flush: the trailing trits **leak into the next layer's word**; the activation buffer is indexed by **neuron** rather than by **chunk**, so every neuron reads a different word where all must see the same input vector; the ping-pong flips **two cycles before** the requantizer emits a layer's final word; and **multi-layer inference deadlocks**, reproduced by an independent Icarus testbench driving the assembled engine *only through its AXI4-Lite CSR aperture*. Two of the five share one line — `assign read_addr = neuron_id;` in `double_buffer_ctrl`. Meanwhile all 28 integration properties still prove: handshakes, buffer phase, address contiguity and readiness are correct while the machine computes the wrong answer and, for more than one layer, does not terminate. Twelve days of instrument auditing found no defect of this kind because every wave asked *is this gate sound?* and none asked *is the design correct?* — a catalogue of failure shapes is a catalogue of the questions asked ([Prop. 121](docs/FORMAL_FOUNDATIONS.md)). A **sixth** defect followed from the completed hunt: `bitnet_engine_top` passes `.length(reg_neurons)` to a DMA whose contract reads *"length is byte-count"* at 8 bytes per beat, while the same register is also the neuron count — so the input DMA moves ⌈N/8⌉ words where the readiness gate demands N, and the deadlock may reach **layer 0** rather than only layer boundaries. Equally worth recording is what was proved **clean**: the quantiser is correct against an independent 17-bit reference over all inputs, including the `threshold = 16'sh8000` case where the 16-bit negation overflows but the priority chain masks it; the packing order matches its documentation exactly; `2'b11` is unreachable in all 27 fields rather than only the one an inline property guards. Five defects sit beside four proved-correct behaviours in one module, and a report listing only failures misrepresents the design. One correction: Prop. 121a called `read_addr = neuron_id` the root of the deadlock, which was the *refuting* agent's judgement — the hunting agent explicitly declined to adjudicate, and two readings remain open ([Prop. 122](docs/FORMAL_FOUNDATIONS.md)). That defect was invisible to every property because each side is internally consistent — the DMA is right that `length` counts bytes, the engine is right that `reg_neurons` counts neurons, and nothing looked at what joins them. `formal/units_scan.py` now reads names across module boundaries. It **could not see the connection it was built for**: a non-greedy body capture stopped at the first `);` and could not survive the nested parenthesis in `.start(reg_ctrl[1] && …)`, so eleven instantiations were parsed, `dma_controller` was not among them, and the tree reported clean — while the `compared > 0` floor passed because twenty *other* connections were compared. **A floor on a total says nothing about coverage of the thing you care about.** Worse, a control keyword parsed as a module name produced a false finding that happened to name exactly the right two signals, which is why the parse defect survived a full self-test run ([Prop. 123](docs/FORMAL_FOUNDATIONS.md)). Three gates now **name the subject they exist for** — `units_scan` that the `dma_controller` instantiation was parsed, `width_scan` that `l2` was examined, `bound_scan` that `accumulator` was classified — each verified by renaming that subject in a scratch copy, 3/3 firing. Widening the units vocabulary was mostly a **negative result**: enumerating the 141 skipped connections revealed `clk`, `rst_n`, `rd_data`, `a`, `b`, `sum`, `cin` and AXI handshakes, which are not quantities at all — the earlier "covers 14%" framing implied most quantities were unchecked when in fact most connections simply are not quantities. One family was genuinely missing (addresses), taking the compared count 23 → 42 with zero new disagreements. And two of the tests written this wave were wrong before either gate was, while a third edit silently did nothing because it used `str.replace()` on a non-matching anchor with no assertion — a rule this campaign has written down three times, violated in the wave that cites it ([Prop. 124](docs/FORMAL_FOUNDATIONS.md)). Sweeping the assembled engine through its CSR aperture with Icarus then settled the root cause, and **none of the three candidates was right**. Exactly one configuration in 81 works: `num_neurons = 1`. Tested alone, the reader-index change is **byte-identical to stock across all 28 configurations** — it changes nothing, and it is what Prop. 121a had published as the root. The packer ratio changes nothing for N ≥ 2. The DMA length is the only single change that unblocks layer 0, and even with it fixed **layer 1 never starts for any N**. The real root is a fourth reading nobody listed: the activation buffer must be indexed by **chunk**, not neuron — every neuron reads the *same* C words of the input vector. Under that reading the packer ratio is **correct and not a defect at all**, and all four remaining errors are faces of **one units confusion, neurons versus 27-trit chunks**. Five coherent changes make two-layer inference complete cleanly for every configuration where `ceil(N/27) ≥ C`, with predicted and measured patterns matching exactly. A defect list assembled from module-level analysis can be complete about symptoms and wrong about causes ([Prop. 125](docs/FORMAL_FOUNDATIONS.md)). **The repair is now applied and verified both ways.** Fifteen emitter edits, each asserted against its anchor and the regenerated bundle checked to carry all fifteen the verified variant had. Under Icarus, through the CSR aperture: layer 0 starts and completes for **every** configuration swept, against one of eighty-one before, and two-layer inference completes with the done IRQ wherever `ceil(N/27) >= C`. The integration suite refuted at first, and a before/after control against the pre-fix tree showed all three engine properties **proved before and refuted after** — they encoded the defect rather than the contract. `a_buffer_alternates` asserted the ping-pong flips one cycle after `layer_done`, which *is* the drain defect; the two read properties tracked a wire the repair disconnected from the memories they describe. Re-pointed, not weakened — **all 28 now prove at `seq 40`** with the simulation unchanged. That makes **four** properties found asserting a defect rather than a contract: a suite grown alongside a bug will contain properties that *are* the bug, and a repair must retire them in the same change or read as a regression ([Prop. 126](docs/FORMAL_FOUNDATIONS.md)). The structural gap behind all six defects is now addressed: **nothing had ever compared an engine output against a reference**, and `sim/tb_data_check.v` is the first check that does. It is possible because the engine's two memory ports are separate — the DMA reads activations, the prefetcher reads weights — so a testbench can serve a known input on one and known weights on the other and compute the expected result itself. Against 9×(+1), 9×(0), 9×(−1) and all-(+1) weights the reference accumulator is exactly 0 — a value wrong under most indexing errors. The first report claimed the engine agreed and called it the campaign's first end-to-end numerical agreement. **That is withdrawn**: the variable holding the engine's accumulator is *initialised* to 0 and assigned only under `mac_valid_q`, and a cycle trace shows `acc` arriving at the requantizer as `xxxx`, which no measured zero explains. A reference chosen to discriminate against the design turned out to be indiscriminate against the harness. The X itself is traced to the weight memory being read before anything writes it, caused by two harness errors — starting inference on a fixed delay rather than an observable condition, and then a wait for `prefetch_done` that **deadlocked**, since the prefetch is triggered *by* the inference start. So the value check is an instrument built, not yet a measurement taken ([Prop. 127](docs/FORMAL_FOUNDATIONS.md)). The harnesses are themselves audited, and that audit found **six defects in the instruments** — a shell whose `echo` truncated a proof result at a backslash escape and flipped a verdict, a mutation harness that scored an unparseable mutant as a killed one, a free-property scan that passed while scanning zero files, a documentation gate this README claimed for many waves while nothing in CI ran it, an expected-refutation check that printed `ok` when the file it greps did not exist and read 1 of 23 candidate files, and a DSL check that verified its output parsed but not that it contained anything ([Props. 58–59](docs/FORMAL_FOUNDATIONS.md)). All six are fixed. Every one had the same shape — **an absence read as a pass** — so the question is now asked mechanically rather than by inspection: `formal/absence_sweep.py` empties `build/rtl/` and `formal/` and runs **all 38 checking steps of both formal workflows**, and any step that still exits 0 fails the build. It covers the workflow it runs inside by excluding itself *by content rather than by name*, and its own blind spot — a workflow whose only step is the sweep — is covered by a shipped self-test ([Props. 59–60](docs/FORMAL_FOUNDATIONS.md)). That sweep also caught the last step that mapped a tool crash to `REFUTED`, which failed safely while telling the reader a property had broken when none had. And the suites' **coverage is now measured rather than assumed**: 202 mechanical mutants of five modules, each property run alone against each mutant, gives **45 detected (22%)** — and a bounded equivalence miter shows **133 of the 157 misses genuinely change behaviour**. **Both of those figures were wrong, and are corrected below**: each module was measured against *one* of its property suites while several modules have two or three, so the real numbers are **74/202 detected (36%)** and **104 real gaps**, of which 15 were closed by properties added since and the rest were being caught all along by suites nobody consulted ([Prop. 73](docs/FORMAL_FOUNDATIONS.md)). The error ran *against* the suite rather than for it, which is why it stood for twelve waves — and nothing misbehaved: the matrix measured exactly what it was told to, while the caption said "gaps in `dma_controller`" where the data said "gaps with respect to `dma_props`". Two properties detect nothing, and for one of them the cause is that mutations touching its signal make its *guard* unreachable, so it proves vacuously rather than being weak — mutation adequacy and vacuity interact, and a naive mutation score cannot tell them apart ([Prop. 61](docs/FORMAL_FOUNDATIONS.md)). Every property has since been given a verdict: **18 bite, 1 is innocent** (4 of 84 mutants kill its guard rather than violate it), **5 are subsumed** by another property, and **none is dead** — the first evidence the suites are lean rather than merely large. All five subsumed properties are **kept**, each with the reason written beside it, because a suite is read as well as run and one of them is the regression witness for the defect Prop. 9 fixed ([Prop. 64](docs/FORMAL_FOUNDATIONS.md)). Extending that to the size sweeps finished 36 of the 42 — 27 bite, 7 subsumed, 1 innocent, **1 dead** (kept: it is an expected refutation whose value is documentary) — and required teaching the sweep that four properties are *supposed* to refute ([Prop. 65](docs/FORMAL_FOUNDATIONS.md)). The engine's 26 integration properties were then **sampled** rather than swept, one mutation per subsystem the campaign has found defects in: **3 of 7 detected** — first reported as 1 of 7, because that count ran only the 26 safety properties and the engine's gate set is safety **∪ liveness**. Including the liveness gate found a second (the dma/overflow mutation stalls two activities outright), and a new **phase-conditioned** probe found a third — **7 engine liveness probes** now, each asserting an activity is impossible so that a *refutation* is the proof the engine still works: every existing probe had asked whether an activity happens *at all*, so a fault that stalls one ping-pong phase kept passing because the activity still occurred in the other ([Prop. 67](docs/FORMAL_FOUNDATIONS.md)). That probe also exposed a bound that lies — at the step's `seq 22` it reports the activity **unreachable**, which is not "unknown" but the wrong answer, and the wrong answer is the one that looks like a passing build; probes now carry per-probe depths. Since only the `proves` direction can fail that way, the bounds were then audited: four wrappers re-proved at **2× and 4×** their CI bound with **no verdict flips** — `dma_controller` survives to `seq 320`, eight times the bound it was raised to in Prop. 35 — and `ls_props` at 4× is reported *undecided* rather than retried until it produced a number ([Prop. 68](docs/FORMAL_FOUNDATIONS.md)). Phase-conditioning turned out **not** to generalise: five further phase-conditioned probes were built and none bites, because the remaining four mutations stall nothing — they change *values* while leaving every activity reachable, which is a safety claim about data where the existing 26 are about control. The figure is a floor, not a coverage number — an engine-scale equivalence miter calls a *known-different* mutant equivalent at `seq 6` and does not finish at `seq 12`, so the six undetected mutations are recorded as undetected and explicitly **not** as gaps ([Prop. 66](docs/FORMAL_FOUNDATIONS.md)). That sampling also caught the mutation generator changing the engine's **own inline properties** — 68% of that file is comment or formal-only text — now masked, dropping the generated mutant count 627 → 481. A **free-property gate** fails the build if any assertion body is discharged by syntax alone, and is mutation-tested in the same step. One suite is proved **unboundedly** by k-induction rather than to a depth; the bounded ones were mapped property-by-property and their bounds raised where that is meaningful — `dma_controller` 12→80, `layer_sequencer` 12→48 ([Props. 35–36](docs/FORMAL_FOUNDATIONS.md)) |
| CI | Schema validation | GREEN | runs `validate-conformance` + `validate-gen-headers`; 101 files: **88 with vectors**, 5 report, 8 definition, 0 empty |
| CI | FPGA smoke | GREEN | Verilog gen in CI |
| CI | FPGA bitstream artifact | GREEN | .bit uploaded per PR (7-day retention) |
| TRI | PHI LOOP CLI | GREEN | `cli/tri/` standalone binary |
| TRI | MCP server | GREEN | `cli/tri-mcp/` — 10 tools over JSON-RPC |
| Spec | Phase 3 (shell/tools/file) | YELLOW | 6/8 parse; 2 file specs have parser issue (#388) |
| BitNet HLS | RTL blocks **emitted** | GREEN | 9/9 modules emit (W36a-f + W38 bundle + R-BV-2 `--with-sva`) |
| BitNet HLS | RTL blocks **integrated** | YELLOW | **10 of 10** modules, 12 instances, and **no known defect open**. Nine RTL defects found and fixed by formal verification across both ends of every count and both sides of the datapath — address wrap, word *N* written at *N+1*, a dual-role pointer, a misplaced reset, a write strobe held as a level, and a read of slots nothing wrote, the last needing three separate changes over eight waves ([Props. 26–47](docs/FORMAL_FOUNDATIONS.md)). CI fails if any property is gated as knowingly broken |
| Host stack | Rust driver + IRQ harness | GREEN | 2/3 layers (W39 R-HS-1 driver, W40 R-HS-2 IRQ); host inference engine in flight (Dmitrii W41-W44 parallel) |
| R-TT track | Tiny Tapeout reproducibility | YELLOW | 2/4 (W42 R-TT-1 `tt-manifest` + chip submodules; W45 R-TT-2 `tt-profile` + `tt-conform`); W46-W47 planned |
| Chips | tt-trinity-{phi,euler,gamma} | GREEN | Pinned as git submodules under `chips/` at known commits (W42) |

### Reproducing this table

Every number above is measured, not asserted. To re-derive them:

```bash
cd bootstrap && cargo build --release && cd ..
cargo test --release 2>&1 | grep '^test result'      # 22 suites, 1213 passed, 0 failed
find specs -name '*.t27' | wc -l                     # 496
for f in $(find specs -name '*.t27'); do \
  ./target/release/t27c parse "$f" >/dev/null || echo "PARSE FAIL $f"; done
ls .trinity/seals/ | wc -l                           # 496 (one per spec)
./target/release/t27c seal-audit --strict             # 496 verify, 0 stale
grep -rh --include='*.v' 'Qed\.' coq trios-coq | wc -l  # 546 across 41 files
```

> **Gotcha:** the built binary lands in the **workspace** target directory,
> `./target/release/t27c` — *not* `bootstrap/target/release/t27c`. Running the
> loop above against the wrong path yields exit 127 on every spec and looks
> exactly like a total parser failure. It is not.

Last measured: **2026-08-09**, commit `1be60604`.

---

## BitNet HLS Pipeline & R-TT Reproducibility Track

The `bootstrap/src/` Rust toolchain (`t27c`) now emits a complete
**9-module BitNet HLS pipeline** and a **Tiny Tapeout reproducibility chain**
tying every tape-out to a specific t27 commit + trinity-invariant SHA-256.

### BitNet HLS pipeline (9 / 9 modules)

| # | Module | Wave | Emitter CLI |
|---|--------|------|-------------|
| 1 | `weight_bram` | W36a | `t27c gen-weight-bram` |
| 2 | `pipeline_stage2_compute` | W36b | `t27c gen-pipeline-stage2` |
| 3 | `layer_sequencer` | W36b | `t27c gen-layer-sequencer` |
| 4 | `double_buffer_ctrl` | W36c | `t27c gen-double-buffer` |
| 5 | `weight_prefetch_ctrl` | W36c | `t27c gen-weight-prefetch` |
| 6 | `bitnet_axi_slave` | W36d | (in bundle) |
| 7 | `bitnet_dma` | W36d | (in bundle) |
| 8 | `bitnet_irq` | W36d | (in bundle) |
| 9 | `bitnet_engine_top` | W36d | (in bundle) |

All nine come together as a single emit:

```
t27c gen-bitnet-bundle --output bundle.sv [--with-sva]
```

The `--with-sva` flag (R-BV-2, Dmitrii) wraps every emit with SystemVerilog
Assertions. **Scope note (measured 2026-08-09):** Yosys's `read_verilog`
frontend supports **neither** named `property` blocks **nor** inline
`assert property (@(posedge clk) ...)` — only immediate assertions inside
`always` — so this output has never been checkable by the open-source flow.
**`sv2v` is not a workaround: it drops assertions silently and exits 0**, which
would produce a green formal run over an empty property set
([Prop. 5](docs/FORMAL_FOUNDATIONS.md)).

For properties that are actually proved, use:

```
t27c gen-behavior-sva-yosys <behaviors.json> --output formal.sv
```

which emits the immediate-assertion subset Yosys accepts (`a |-> b` and
`a |-> ##N b`; `s_eventually` is liveness and is **reported**, not silently
dropped). The `formal-yosys` CI job proves it and includes a **vacuity gate**
that counts `$check` cells, so an empty property set fails instead of passing.
See [`docs/FORMAL_FOUNDATIONS.md`](docs/FORMAL_FOUNDATIONS.md) Props. 2, 3, 5, 6.

**This found a real bug.** `formal/interrupt_controller_props.sv` proves six
properties of the generated `interrupt_controller`. One of them,
`a_event_never_lost`, was **refutable** until 2026-08-09: the RTL cleared
`irq_status` on `status_read` as the last of four independent non-blocking
assignments, so last-write-wins meant a status read concurrent with an
interrupt **provably destroyed that event** — always, on every reachable state,
not occasionally. Fixed by clear-then-set; the harness is a regression witness
that refutes against the old RTL. See
[`docs/FORMAL_FOUNDATIONS.md`](docs/FORMAL_FOUNDATIONS.md) Prop. 7.

**And a second one.** `axi_lite_slave` asserted `awready`/`wready`/`arready` at
reset and never deasserted them, while holding a single response register per
channel — so a second transaction was accepted while the first response was
unacknowledged, merging two transactions into one response beat and hanging the
master. Formalised as a transaction balance (`outstanding <= 1`), refuted on
both channels, fixed by releasing `ready` only on the response handshake.
Prop. 8.

**And two more in `dma_controller`.** `arlen`/`awlen` were hardwired to 256
beats for every transfer while the FSM stopped when the byte count ran out —
so a short transfer requested 256 beats and then **abandoned the burst**, which
an AXI4 master may not do. Separately, `READ_ADDR` advanced on `arready` alone,
so a ready-without-valid moved the FSM into `READ_DATA` **having issued no
address**. Prop. 9.

**And two zero-count non-terminations.** `layer_sequencer` with
`num_neurons == 0` compares its terminator against `16'hFFFF` and emits work for
neuron 0, 1, 2, … forever; `weight_prefetch_ctrl` with `num_words == 0`
underflows and writes BRAM past the end of the buffer. Both were stated as
safety **bounds** rather than liveness, since a runaway loop usually has a
safety shadow. Telling detail: `layer_sequencer` already guarded
`num_chunks == 0`, and `multilayer_sequencer` guards `num_layers > 0` — two
siblings in the family guard the zero case and two did not. Prop. 13.

All six defects had a **passing unit test pinning the buggy text in place** —
one named `dma_burst_length_is_max`, asserting the defect as if it were the
contract. Nine such tests have now been rewritten to assert behaviour.

`formal/axi4_read_slave_model.sv` is a reusable AXI4 read-slave model for
master-side properties; its single-burst precondition is **asserted, not
assumed**, so it cannot silently hide the defects it exists to find. The
`arlen`-at-handshake anomaly it surfaced is **resolved** (Prop. 11): Yosys's
`sat` ignores `$assume` cells unless `-set-assumes` is passed — silently — so
the harness had never applied its own constraints. Under a compliant slave the
property proves. CI now proves `formal/assume_liveness_check.sv` first, which
passes only when assumptions are actually live.

### Host stack (W39 R-HS-1, W40 R-HS-2)

A pure-Rust host driver against a MockMmio matching the W36d AXI-Lite slave
CSR map (`CTRL`, `STATUS`, `IRQ_EN`, `IRQ_STAT` with W1C semantics, `NUM_LAYERS`,
`NEURONS`, `CHUNKS`, `THRESHOLD`, `WEIGHT_ADDR_LO/HI`).  Two CLIs:

```
t27c host-smoke         # busy-poll path
t27c host-poll-vs-irq   # comparison harness (poll vs IRQ-handler)
```

Dmitrii's parallel R-HS track (W41-W44) is extending this with a full
host-side inference engine, performance cycle estimator, `--json` output,
and ternary weight packer.

### R-TT track -- Tiny Tapeout reproducibility (2 / 4)

The three Tiny Tapeout silicon variants -- `tt-trinity-phi`,
`tt-trinity-euler`, `tt-trinity-gamma` -- live as git submodules under
`chips/` pinned to known commits.  Two CLIs make every tape-out machine-checkable:

```
t27c tt-manifest --chip <phi|euler|gamma> [--output <path>|-]
#  -> deterministic JSON manifest:
#     { t27_commit, phi_invariant_hash, chip, modules[], axi_widths,
#       sva_count, build_time_utc }

t27c tt-profile --platform <sky130|ihp|gf180> [--output <path>|-]
#  -> deterministic JSON profile:
#     { platform, process_node_nm, cell_library, max_tile_area_um2,
#       supply_voltage_mvolts, target_clock_mhz, max_modules }

t27c tt-conform --profile <p.json> --manifest <m.json> [--verbose]
#  -> single boolean gate:
#     OK conform=<true|false> reasons=<N>
#  exit 0 if ok, 1 otherwise
```

The `phi_invariant_hash` is the SHA-256 of the ASCII string
`phi^2 + 1/phi^2 = 3` (`218403e3...8f80e6b`) and is embedded in every
manifest -- any silent change to the numeric kernel would change the hash
and show up immediately in diff.

Roadmap:

- W46 R-TT-3 `tt-debug` -- TT-debug wrapper around `bitnet_engine_top` (version CSR + error counters + self-test trigger)
- W47 R-TT-4 `tt-lockfile` -- `tt.lock` (chip-hash + commit + profile + verdict) pinned in each chip-repo via submodule

### Submodule layout

```
chips/phi    -> https://github.com/gHashTag/tt-trinity-phi
chips/euler  -> https://github.com/gHashTag/tt-trinity-euler
chips/gamma  -> https://github.com/gHashTag/tt-trinity-gamma
```

Clone with submodules:

```
git clone --recursive https://github.com/gHashTag/t27.git
# or, after a plain clone:
git submodule update --init --recursive
```

### Test coverage (post-W45)

- BitNet HLS suites: 9 modules x dedicated integration suite each
- Host stack: `host_driver` (25), `host_irq` (25)
- R-TT track: `tt_manifest` (23 + 18 inline), `tt_profile` (25 + 24 inline)
- Regression: **22 integration suites green**, total **1195 / 1195 passed,
  0 failed** (`cargo test --release`, measured 2026-08-09 at `1be60604`).
  The long-standing fail in
  `verilog_const_array::r_ca_1_emitter_on_real_mac_spec` is **fixed** as of
  the R12-R14 audit rounds; the suite is now fully green.

Live wave-by-wave log: [`docs/NOW.md`](docs/NOW.md).

---

## What is t27?

t27 is a **spec-first** language for ternary computing. You write `.t27` specifications -- the compiler generates Zig, Verilog, and C backends. No hand-editing generated code. Ever.

The language is built around three pillars:

- **27 Coptic registers** -- a ternary ISA with trits `{-1, 0, +1}`
- **GoldenFloat family** -- phi-structured floating-point formats (GF4-GF32) where `exp/mant ~ 1/phi`
- **Sacred physics** -- fundamental constants derived from `phi^2 + 1/phi^2 = 3`

t27 is the core of [Trinity S3AI](https://github.com/gHashTag/trinity) -- a neuroanatomical AI framework targeting FPGA acceleration and DARPA CLARA compliance.

## Quick Start

```bash
# Clone
git clone https://github.com/gHashTag/t27.git
cd t27

# Build the bootstrap compiler (Rust); use ./scripts/tri as the CLI entry (wraps t27c)
cd bootstrap && cargo build --release
cd ..

# Parse a spec (canonical CLI: tri → wraps bootstrap t27c)
./scripts/tri parse specs/base/types.t27

# Generate Zig (stdout for one file; if the path is a directory, batch → gen/zig/… by default)
./scripts/tri gen-zig specs/numeric/gf16.t27
./scripts/tri gen-zig specs/numeric
# Or: ./scripts/tri gen-dir --backend zig --out-root gen/zig <dir>

# Generate Verilog (file or directory → gen/verilog/…)
./scripts/tri gen-verilog specs/fpga/mac.t27

# Generate C (file or directory → gen/c/…)
./scripts/tri gen-c specs/base/ops.t27

# Verify a seal
./scripts/tri seal specs/numeric/gf16.t27 --verify

# Run all tests (Rust suite: parse / gen / seal / fixed-point)
./scripts/tri test

# Validate conformance vectors (JSON under conformance/)
./scripts/tri validate-conformance

# Validate generated file headers under gen/
./scripts/tri validate-gen-headers

# NOW.md date gate (also runs inside t27c before gen / gen-dir / compile*)
./scripts/tri check-now
```

## Architecture

The project is organized into 5 strands that evolved ring-by-ring:

```
STRAND I   - Base         : types, ops, constants          (Rings 0-8)
STRAND II  - Numeric+VSA  : GF4-GF32, TF3, phi, VSA ops   (Rings 9-11)
STRAND III - Compiler+FPGA: parser, MAC, ISA registers      (Rings 12-14)
STRAND IV  - Queen+NN     : Lotus orchestration, HSLM, attention (Rings 14-17)
STRAND V   - AR (CLARA)   : ternary logic, proof traces, Datalog, restraint, XAI, ASP, composition (Rings 18-24)
```

Gen backends (Zig, C, Verilog) and conformance vectors were generated across Rings 25-31.

### Agent experience (design)

Multi-agent memory, Queen wisdom, and planned **`tri`** subcommands for experience / insights are outlined in **[`docs/TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md`](docs/TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md)**. **Today’s supported pipeline** is the Quick Start block above (`tri test`, `tri check-now`, validators, codegen).

### Directory Structure

```
t27/
├── specs/                  # .t27 SPECIFICATIONS -- source of truth
│   ├── base/               #   types, ops (2 specs)
│   ├── numeric/            #   GoldenFloat GF4-GF32, TF3, phi_ratio (10 specs)
│   ├── math/               #   sacred_physics, constants (2 specs)
│   ├── ar/                 #   CLARA AR pipeline -- logic, proof, datalog (7 specs)
│   ├── nn/                 #   HSLM, attention kernels (2 specs)
│   ├── isa/                #   27 Coptic registers (1 spec)
│   ├── fpga/               #   MAC unit for XC7A100T (1 spec)
│   ├── vsa/                #   Vector Symbolic Architecture (1 spec)
│   ├── queen/              #   Lotus orchestration (1 spec)
│   └── compiler/           #   Parser self-spec (1 spec)
│
├── compiler/               # Compiler .t27 specs (15 specs)
│   ├── parser/             #   lexer.t27, parser.t27
│   ├── codegen/            #   zig/, verilog/, c/, testgen
│   ├── cli/                #   gen, git, spec commands
│   ├── runtime/            #   commands, validation
│   └── skill/              #   PHI LOOP skill registry
│
├── gen/                    # GENERATED backends -- DO NOT EDIT
│   ├── zig/                #   Zig backend (28 modules)
│   ├── c/                  #   C backend (28 .c + 28 .h)
│   └── verilog/            #   Verilog backend (28 modules)
│
├── conformance/            # Language-agnostic test vectors (34 JSON)
│   ├── gf*_vectors.json    #   GoldenFloat arithmetic vectors
│   ├── ar_*.json           #   CLARA AR conformance vectors
│   ├── nn_*.json           #   Neural architecture vectors
│   └── sacred_physics*.json#   phi, gamma, G, Omega_Lambda conformance
│
├── bootstrap/              # Stage-0 compiler (Rust) -- FROZEN
│   └── src/compiler.rs     #   SHA-256 sealed in bootstrap/stage0/FROZEN_HASH
│
├── architecture/           # Dependency graph + ADRs
│   ├── graph.tri           #   Canonical dependency DAG
│   ├── graph_v2.json       #   Machine-readable graph (20 nodes)
│   └── ADR-*.md            #   Architecture Decision Records
│
├── .trinity/               # Agent state (Akashic Chronicle)
│   ├── events/             #   Append-only event journal
│   ├── experience/         #   PHI LOOP episodes (38 episodes)
│   ├── seals/              #   48 SHA-256 integrity seals
│   ├── state/              #   queen-health.json, graph sync
│   ├── claims/             #   Agent ownership claims
│   ├── queue/              #   Task queue
│   └── policy/             #   Coordination law
│
├── contrib/                # Non-core adjacency (API, runners, portable setup) — see OWNERS.md
├── external/               # Vendored upstream (e.g. OpenCode submodule) + kaggle tree — see OWNERS.md
│
├── docs/                   # First-party docs (27-agent / 3-nona layout — see docs/README.md)
│   ├── README.md           #   Index: agents/, coordination/, nona-01..03/, clara/
│   ├── NOW.md              #   Rolling snapshot (sync gates)
│   ├── T27-CONSTITUTION.md #   Charter
│   └── …                   #   nona-01-foundation/, nona-02-organism/, nona-03-manifest/, etc.
│
└── tests/                  # Ring verification + validation scripts
    ├── comprehensive_suite.t27 # Suite contract (see t27c suite)
    └── *.t27             #   Spec tests only — no shell runners
```

**Domain ownership:** each major directory may include an `**OWNERS.md`** (Primary agent, dependencies, outputs). Start at `[OWNERS.md](OWNERS.md)` in the repo root; see also `[docs/agents/AGENTS_ALPHABET.md](docs/agents/AGENTS_ALPHABET.md)`.

## CLARA Automated Reasoning

The AR domain (Rings 18-24) implements a full DARPA CLARA-compliant reasoning pipeline in ternary logic:


| Module             | Spec                          | Description                                                                                                 |
| ------------------ | ----------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Ternary Logic**  | `specs/ar/ternary_logic.t27`  | Kleene K3 logic: `{T, U, F}` isomorphic to trits `{+1, 0, -1}`. 27 truth table entries, verified K3 axioms. |
| **Proof Traces**   | `specs/ar/proof_trace.t27`    | Bounded proof traces with a hard 10-step limit. Each step carries a GF16 confidence score.                  |
| **Datalog Engine** | `specs/ar/datalog_engine.t27` | Forward-chaining Datalog with O(n) complexity. Stratified negation via K3 unknown.                          |
| **Restraint**      | `specs/ar/restraint.t27`      | Bounded rationality: resource limits on inference (max steps, max memory, timeout).                         |
| **Explainability** | `specs/ar/explainability.t27` | CLARA-compliant XAI: explanations <= 10 steps, each with GF16 confidence.                                   |
| **ASP Solver**     | `specs/ar/asp_solver.t27`     | Answer Set Programming with Negation-as-Failure under K3 semantics.                                         |
| **Composition**    | `specs/ar/composition.t27`    | ML+AR composition patterns: CNN+Rules, MLP+Bayesian, Transformer+XAI, RL+Guardrails.                        |


All 7 AR modules have gen backends (Zig, C, Verilog) and conformance vectors.

## Conformance Testing

Every domain has language-agnostic conformance vectors in `conformance/*.json`. These JSON files contain test inputs, expected outputs, and tolerances that any backend must satisfy.

**34 conformance vectors** cover:

- GoldenFloat arithmetic (GF4 through GF32)
- Sacred physics constants (phi, gamma, G, Omega_Lambda)
- Base types and operations
- CLARA AR pipeline (all 7 modules)
- Neural architecture (attention, HSLM)
- Domain modules (VSA ops, ISA registers, FPGA MAC, Queen Lotus)

Validation: `./scripts/tri validate-conformance`

## SEED-RINGS Progress

The compiler grows ring-by-ring. Each ring adds exactly one capability, sealed with SHA-256 hashes.


| Ring | Capability                                         | Layer  | Status      |
| ---- | -------------------------------------------------- | ------ | ----------- |
| 0    | Frozen stage-0 + first green parse                 | SEED   | Sealed      |
| 1    | Lex all 28 specs without errors                    | SEED   | Sealed      |
| 2    | Type declarations -> Zig codegen                   | SEED   | Sealed      |
| 3    | fn signatures -> Zig                               | SEED   | Sealed      |
| 4    | module + use -> Zig imports                        | SEED   | Sealed      |
| 5    | fn body expressions -> Zig                         | ROOT   | Sealed      |
| 6    | test blocks -> Zig test blocks                     | ROOT   | Sealed      |
| 7    | invariant + bench -> Zig                           | ROOT   | Sealed      |
| 8    | Conformance vectors -> test_vector_hash            | ROOT   | Sealed      |
| 9    | Full Zig backend                                   | TRUNK  | Sealed      |
| 10   | Verilog backend                                    | TRUNK  | Sealed      |
| 11   | C backend                                          | TRUNK  | Sealed      |
| 12   | seal --save / --verify                             | TRUNK  | Sealed      |
| 13   | AR pipeline -- all 7 specs                         | BRANCH | Sealed      |
| 14   | Queen + NN specs gen and seal                      | BRANCH | Sealed      |
| 15   | Full test suite -- all 43 specs                    | BRANCH | Sealed      |
| 16   | Self-hosting: stage(N) == stage(N-1)               | CANOPY | Sealed      |
| 17   | Self-hosting verified (fixed point)                | CANOPY | Sealed      |
| 18   | AR ternary logic (K3 isomorphism)                  | AR     | Sealed      |
| 19   | Bounded proof traces                               | AR     | Sealed      |
| 20   | Datalog engine (forward chaining)                  | AR     | Sealed      |
| 21   | Restraint (bounded rationality)                    | AR     | Sealed      |
| 22   | Explainability (CLARA XAI)                         | AR     | Sealed      |
| 23   | ASP solver (NAF + K3)                              | AR     | Sealed      |
| 24   | ML+AR composition (4 patterns)                     | AR     | Sealed      |
| 25   | Gen backends: base/types, base/ops, math/constants | GEN    | Sealed      |
| 26   | Gen backends: numeric core (GF4-GF16, TF3, phi)    | GEN    | Sealed      |
| 27   | Gen backends: extended numerics (GF20-GF32)        | GEN    | Sealed      |
| 28   | Gen backends: VSA, ISA, FPGA, sacred physics       | GEN    | Sealed      |
| 29   | Gen backends: NN attention, HSLM, Queen Lotus      | GEN    | Sealed      |
| 30   | Conformance vectors: AR gap coverage               | GEN    | Sealed      |
| 31   | Compiler/parser gen + graph sync + queen health    | GEN    | Sealed      |
| 32+  | Hardening: docs, validation, CI                    | HARDEN | In Progress |


## Wave 11 — Rust Crates ring-088..ring-099 (12 crates)

> **Honest status:** все 12 крейтов **написаны на диске** (Rust, ~10 000+ строк, 33 `Cargo.toml`), но **`cargo check` / `cargo test` НЕ запускались** — `cargo` и `rustc` не установлены в текущем окружении (network/permission). Цифры строк подтверждены `find` + `wc`.

| Crate | Files | Rust LOC | Topic | Status |
|:------|------:|---------:|:------|:------:|
| `ring-088-rust` |  5 |   961 | GF16 MAC                     | ✅ written, ⏳ uncompiled |
| `ring-089-rust` |  4 |   334 | TNN ISA                      | ✅ written, ⏳ uncompiled |
| `ring-090-rust` | 10 | 2 143 | Simulator                    | ✅ written, ⏳ uncompiled |
| `ring-091-rust` |  4 |   409 | Stochastic Rounding          | ✅ written, ⏳ uncompiled |
| `ring-092-rust` |  6 |   847 | Attention                    | ✅ written, ⏳ uncompiled |
| `ring-093-rust` |  4 |   668 | Sparse MoE                   | ✅ written, ⏳ uncompiled |
| `ring-094-rust` |  4 |   774 | AGI Runtime                  | ✅ written, ⏳ uncompiled |
| `ring-095-rust` |  4 |   659 | φ-Adam Optimizer             | ✅ written, ⏳ uncompiled |
| `ring-096-rust` |  4 |   464 | Quantization (GF16 / INT4)   | ✅ written, ⏳ uncompiled |
| `ring-097-rust` |  4 |   624 | Chain-of-Thought Engine      | ✅ written, ⏳ uncompiled |
| `ring-098-rust` |  5 |   920 | World Model                  | ✅ written, ⏳ uncompiled |
| `ring-099-rust` |  6 | 1 127 | Integration / `trinity` bin  | ✅ written, ⏳ uncompiled |

**Totals:** 12 crates · 60 source files · ≈ 9 930 Rust LOC · 33 `Cargo.toml`.

### Toolchain availability (honest)

| Tool          | Installed | Verified |
|:--------------|:---------:|:--------:|
| `cargo`       | ❌ no | ❌ |
| `rustc`       | ❌ no | ❌ |
| `cargo check` | ❌ n/a | ❌ |
| `cargo test`  | ❌ n/a | ❌ |

**Why:** the sandbox used during Wave 11 had no Rust toolchain (network timeout / permission denied on install). The crates compile-status will be verified in Wave 12 once a Rust-enabled image is in place.

## Wave 12 — Plan: Compile + Integrate + Expand

```
╔═══════════════════════════════════════════════════════════════════════╗
║  WAVE 12 — COMPILATION + INTEGRATION + NEW RINGS                      ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Track A · 3 agents — Fix `cargo check` errors across all 12 crates   ║
║  Track B · 3 agents — Finish execution units inside ring-090 (sim)    ║
║  Track C · 3 agents — Author ring-100..ring-104 (Multi-Chip, Analog…) ║
║  Track D · 3 agents — Docker image w/ full Rust toolchain + CI hook   ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Exit criteria:                                                       ║
║    • cargo check         ≥ 9/12 crates                                ║
║    • cargo test          ≥ 6/12 crates                                ║
║    • `trinity` binary    runs end-to-end (ring-099)                   ║
║    • Docker image        published, CI green on PR                    ║
╚═══════════════════════════════════════════════════════════════════════╝
```

**Tracks in detail**

- **Track A — Compile fix:** run `cargo check` per crate, triage errors (missing deps, lifetime, type mismatches), submit one PR per crate with `Closes #<ring>`.
- **Track B — Simulator finish:** complete the missing execution units in `ring-090-rust` (decode → issue → writeback), add property tests against `conformance/*.json`.
- **Track C — New rings:** spec → crate scaffolding for `ring-100` Multi-Chip Mesh, `ring-101` Analog GF16, `ring-102` Photonic MAC, `ring-103` On-Chip Learning, `ring-104` Telemetry Bus. **Status: scaffolded — 5 crates landed on disk (see table below).**
- **Track D — Toolchain:** `Dockerfile.rust` based on `rust:1.83-bookworm`, GitHub Actions matrix building all `ring-0**-rust` crates, artifact upload on failure.

### Wave 12 / Track C — ring-100..ring-104 (scaffolded 2026-05-22, Closes #711)

> **Honest status:** all 5 crates **written on disk** with `Cargo.toml` + `src/lib.rs` + per-crate `README.md` and `#[test]` coverage (28 tests total). `cargo check` / `cargo test` **still not run** — toolchain hookup is Track D. Crates are intentionally **not** added to `[workspace].members` until Track D Docker image is in place.

| Crate                          | Files | Rust LOC | Tests | Domain                                                                |
|:-------------------------------|------:|---------:|------:|:----------------------------------------------------------------------|
| `rings/ring-100-rust`          |   3   |   205    |   5   | Multi-Chip Mesh — Phi+Euler+Gamma triad fabric, XY routing             |
| `rings/ring-101-rust`          |   3   |   144    |   5   | Analog GF16 — quantize/dequantize + reproducible noise channel        |
| `rings/ring-102-rust`          |   3   |   157    |   5   | Photonic MAC — wavelength-multiplexed dot product w/ insertion loss   |
| `rings/ring-103-rust`          |   3   |   131    |   6   | On-Chip Learning — φ-tempered SGD step                                |
| `rings/ring-104-rust`          |   3   |   185    |   7   | Telemetry Bus — bounded lossy ring buffer of (ts, tag, value)         |

**Totals:** 5 crates · 15 files · 822 Rust LOC · 28 `#[test]`s.

Every crate exposes `identity_witness()` (or `Mesh::identity_witness` in ring-100) asserting `phi^2 + 1/phi^2 == 3` to f64 1e-15.

## Wave 13 — Toolchain & Compilation Gate (2026-05-22, Closes #713)

> **Why this wave:** Waves 11 and 12/Track-C produced 17 Rust crates on disk (≈ 10 750 LOC, 60+ tests), but `cargo check` and `cargo test` were **never** actually executed in CI. Wave 13 lands the missing infrastructure — pinned Rust toolchain, a generated GHA matrix that builds every `rings/ring-*-rust/` crate, and a living per-crate status table. The gate is **non-blocking on purpose**: it surfaces real per-crate compile state without yet enforcing it, so the project can finally distinguish *scaffolded* from *compiles* from *tested*.

```
╔═══════════════════════════════════════════════════════════════════════╗
║  WAVE 13 — TOOLCHAIN & COMPILATION GATE                               ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Dockerfile.rust            rust:1.83-bookworm, pkg-config, rustup    ║
║  scripts/ci/rings_matrix.py pure-stdlib GHA matrix generator          ║
║  .github/workflows/         rings-rust.yml — matrix cargo check+test  ║
║  rings/COMPILE_STATUS.md    living per-crate status (legend below)    ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Legend:  scaffold  →  check  →  test  →  (off-disk for ring-088..099)║
║  Gate is `continue-on-error: true` — honesty surface, not enforcer.   ║
╚═══════════════════════════════════════════════════════════════════════╝
```

**Deliverables**

| Artifact                                  | Role                                                                |
|:------------------------------------------|:--------------------------------------------------------------------|
| `Dockerfile.rust`                         | Pinned `rust:1.83-bookworm` image — `rustc`, `cargo`, `rustfmt`, `clippy` |
| `scripts/ci/rings_matrix.py`              | Discovers `rings/ring-*-rust/` crates → emits GHA matrix JSON       |
| `.github/workflows/rings-rust.yml`        | `discover` → matrix `cargo check` + `cargo test` → step-summary     |
| `rings/COMPILE_STATUS.md`                 | Living per-crate status table (`scaffold` / `check` / `test` / `off-disk`) |

**Honest status at landing:** all 5 Wave-12 Track-C crates start as `scaffold` in the table; the 12 Wave-11 crates remain `off-disk` until imported. No row will be promoted past `scaffold` without a CI log to prove it (**R5-HONEST**).

## GoldenFloat Family

phi-structured floating-point formats where `exp/mant ~ 1/phi`:


| Format   | Bits   | Exp   | Mant  | phi-distance | Use Case       |
| -------- | ------ | ----- | ----- | ------------ | -------------- |
| GF4      | 4      | 1     | 2     | 0.118        | Binary masks   |
| GF8      | 8      | 3     | 4     | 0.132        | Weights        |
| GF12     | 12     | 4     | 7     | 0.047        | Attention      |
| **GF16** | **16** | **6** | **9** | **0.049**    | **Primary**    |
| GF20     | 20     | 7     | 12    | 0.035        | Training       |
| GF24     | 24     | 9     | 14    | 0.025        | Precision      |
| GF32     | 32     | 12    | 19    | 0.014        | Full precision |

### Multi-Language Installation

GoldenFloat is available as native packages for Python, JavaScript, Rust, and C:

**Python (PyPI):**
```bash
pip install golden-float
```

**JavaScript (npm):**
```bash
npm install golden-float
```

**Rust (crates.io):**
```toml
[dependencies]
golden-float-ffi = "0.1"
```

**C/C++ (header-only):**
```c
#include "golden_float.h"  // Auto-generated from gen/c/numeric/
```

All implementations share a single Rust core with a C-compatible ABI, guaranteeing **bit-identical results** across languages. See [`docs/MIGRATION.md`](docs/MIGRATION.md) for detailed installation and migration guides.


## Sacred Constants

```t27
pub const PHI: GF16         = 1.618033988749895;   // Golden ratio
pub const PHI_INV: GF16     = 0.618033988749895;   // phi^-1
pub const TRINITY: GF16     = 3.0;                  // phi^2 + phi^-2 = 3
pub const GAMMA_LQG: GF16   = 0.2360679775;         // phi^-3 (Barbero-Immirzi)
pub const G_MEASURED: GF32   = 6.67430e-11;          // Gravitational constant
pub const OMEGA_LAMBDA: GF32 = 0.685;                // Dark energy density
```

## 27-Agent System

Trinity runs 27 autonomous agents -- one per Coptic register:


| Agent         | Domain                             | Key Files                           |
| ------------- | ---------------------------------- | ----------------------------------- |
| **T** (Queen) | Orchestration, 6-phase Lotus cycle | `specs/queen/lotus.t27`             |
| **A**         | Architecture, SOUL.md, ADRs        | `architecture/`                     |
| **B**         | Build, CI/CD, Railway              | `bootstrap/`                        |
| **C**         | Compiler core, parser, AST         | `compiler/parser/`                  |
| **D**         | De-Zigfication migration           | `specs/` -> generated backends      |
| **F**         | Formal conformance vectors         | `conformance/*.json`                |
| **G**         | Graph topology, ARCH_BENCH         | `architecture/graph.tri`            |
| **H**         | HSLM neural architecture           | `specs/nn/`                         |
| **I**         | ISA, 27 Coptic registers           | `specs/isa/registers.t27`           |
| **K**         | FPGA/MAC kernel                    | `specs/fpga/mac.t27`                |
| **N**         | GoldenFloat numeric                | `specs/numeric/`                    |
| **P**         | Sacred physics constants           | `specs/math/`                       |
| **V**         | Verdict, toxicity scoring          | `conformance/`, `.trinity/verdict/` |
| **27th**      | Security, AAIF compliance          | `.trinity/policy/`                  |


Full list: [docs/agents/AGENTS_ALPHABET.md](docs/agents/AGENTS_ALPHABET.md)

## Constitutional Laws

8 immutable laws govern all mutations. Violations produce **TOXIC** verdicts.


| LAW | Name                 | Rule                                                                         |
| --- | -------------------- | ---------------------------------------------------------------------------- |
| 1   | **De-Zigfication**   | `.t27` specs are the only source of truth. Zig/C/Verilog = generated output. |
| 2   | **PHI LOOP**         | Every mutation follows a 9-step workflow with 4 SHA-256 hashes.              |
| 3   | **SEED-RINGS**       | Language grows ring-by-ring. One ring = one capability.                      |
| 4   | **ISSUE-GATE**       | No byte enters `master` without an Issue, a PR, and `Closes #N`.             |
| 5   | **SOUL.md**          | Every `.t27` spec must contain `test {}`, `invariant {}`, or `bench {}`.     |
| 6   | **NUMERIC-STANDARD** | GoldenFloat defined in specs + conformance JSON. Never in backend code.      |
| 7   | **SACRED-PHYSICS**   | Sacred constants live in `specs/math/` with hard tolerances.                 |
| 8   | **GRAPH TOPOLOGY**   | Evolution follows `architecture/graph.tri`. No circular deps.                |


Details: [SOUL.md](SOUL.md) | [SEED-RINGS](docs/nona-01-foundation/SEED-RINGS.md) | [NUMERIC-STANDARD-001](docs/nona-02-organism/NUMERIC-STANDARD-001.md) | [SACRED-PHYSICS-001](docs/nona-02-organism/SACRED-PHYSICS-001.md)

## PHI LOOP Workflow

Every change follows this exact 9-step cycle:

```
tri skill begin <task> --issue <N>    <- bind to GitHub Issue
tri spec edit <module>                <- edit ONE .t27 spec
tri skill seal --hash                 <- record 4 SHA-256 hashes
tri gen                               <- generate Zig/Verilog/C
tri test                              <- run tests
tri verdict --toxic                   <- TOXIC? -> rollback. CLEAN? -> proceed
tri experience save                   <- append episode to Akashic journal
tri skill commit                      <- verify hashes + issue binding
tri git commit                        <- push with "Closes #N"
```

## Contributing

1. Open a [GitHub Issue](https://github.com/gHashTag/t27/issues) first -- **no issue = no work** (LAW 4)
2. Create a branch: `ring/<N>-<name>`, `ar/<AR-NNN>-<name>`, `fix/<name>`, or `task/<name>`
3. Edit `.t27` specs only -- never hand-edit generated Zig/Verilog/C (LAW 1)
4. Every spec must have `test {}`, `invariant {}`, or `bench {}` blocks (LAW 5)
5. Commit message: `feat(ring-N): description [SEED-N]` with `Closes #N`
6. Open a PR targeting `master`

## PHI LOOP Status

- **31 rings sealed** (SEED-0 through SEED-17, AR 18-24, GEN 25-31)
- **45 .t27 spec files** (28 specs/ + 15 compiler/ + 2 sandbox)
- **112 generated files** across 3 backends (Zig, C, Verilog)
- **34 conformance vectors** covering all domains
- **48 integrity seals** in .trinity/seals/
- **6 CLI commands**: parse, gen, gen-zig, gen-verilog, gen-c, seal
- **5 architecture strands**: Base -> Numeric -> Compiler+FPGA -> Queen+NN -> AR
- **Deterministic fixed point** reached at Ring 17 (CANOPY)
- **CLARA AR module**: 7 specs (ternary logic -> composition)
- **Queen health**: GREEN 1.0 across 15 domains
- CI enforced: Issue Gate + PHI Loop CI on all PRs

## CI Enforcement

All PRs to `master` must:

1. Link to an issue via `Closes #N`
2. Pass PHI Loop CI (build, parse, gen, seal verify)
3. Pass conformance validation
4. Pass gen header validation
5. Pass seal coverage check

See [ISSUE-GATE-001](docs/nona-03-manifest/ISSUE-GATE-001.md) for details.

## Documentation

**Full map (27 agents / three nonas):** [docs/README.md](docs/README.md)

### Governance

- [SOUL.md](SOUL.md) -- Constitutional law
- [SEED-RINGS](docs/nona-01-foundation/SEED-RINGS.md) -- Incremental compiler bootstrap
- [NUMERIC-STANDARD-001](docs/nona-02-organism/NUMERIC-STANDARD-001.md) -- GoldenFloat specification
- [SACRED-PHYSICS-001](docs/nona-02-organism/SACRED-PHYSICS-001.md) -- Sacred physics constants
- [PHI LOOP Contract](docs/nona-03-manifest/PHI_LOOP_CONTRACT.md) -- Workflow contract
- [TDD Contract](docs/nona-03-manifest/TDD-CONTRACT.md) -- Test-driven development policy

### Architecture

- [ADR-001: De-Zigfication](architecture/ADR-001-de-zigfication.md)
- [ADR-003: TDD Inside Spec](architecture/ADR-003-tdd-inside-spec.md)
- [ADR-004: Language Policy](architecture/ADR-004-language-policy.md)
- [ADR-005: De-Zig Strict](architecture/ADR-005-de-zig-strict.md)
- [CANON DE-ZIGFICATION](architecture/CANON_DE_ZIGFICATION.md)
- [TECHNOLOGY-TREE](docs/nona-03-manifest/TECHNOLOGY-TREE.md) -- Evolution roadmap

### Agents & Operations

- [27-Agent Alphabet](docs/agents/AGENTS_ALPHABET.md) -- All 27 agents
- [CLARA Preparation Plan](docs/clara/CLARA-PREPARATION-PLAN.md) -- DARPA compliance
- [Kleene Trit Isomorphism](docs/nona-02-organism/KLEENE-TRIT-ISOMORPHISM.md)
- [TRI Syntax vNext](docs/nona-02-organism/TRI_SYNTAX_VNEXT.md)
- [ISSUE-GATE-001](docs/nona-03-manifest/ISSUE-GATE-001.md) -- Issue gate enforcement law

## License

MIT

---

**φ² + 1/φ² = 3 | TRINITY**

**Maintained by**: [Trinity Project](https://github.com/gHashTag) — [Dmitrii Vasilev](https://github.com/gHashTag)

**Status:** Ring 31 Complete (2026-04-08) — 31 rings sealed, 45 specs, 112 gen files, 34 conformance vectors, 48 seals, CI enforced.

**Wave 11 (2026-05-22):** 12 Rust crates `ring-088`..`ring-099` written (≈ 9 930 LOC, 33 `Cargo.toml`), **compilation not yet verified** — toolchain unavailable in sandbox; verification deferred to Wave 12. See the *Wave 11 / Wave 12* sections above for the honest status table and the four-track plan.

**Wave 12 / Track C (2026-05-22):** 5 new crates `ring-100`..`ring-104` scaffolded (Multi-Chip / Analog GF16 / Photonic MAC / On-Chip Learning / Telemetry Bus) — 15 files, 822 Rust LOC, 28 `#[test]`s. `cargo` gate handed off to Track D.

**Wave 13 (2026-05-22):** Toolchain & Compilation Gate landed — `Dockerfile.rust` (`rust:1.83-bookworm`), `scripts/ci/rings_matrix.py`, `.github/workflows/rings-rust.yml` matrix build, `rings/COMPILE_STATUS.md` living per-crate status. Non-blocking honesty gate — see [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 14 (2026-05-22):** Rings compile green — root `Cargo.toml` `exclude` extended with `rings/`. All 5 Track-C crates (`ring-100`..`ring-104`) promoted from `scaffold` to `check` + `test` in [COMPILE_STATUS](rings/COMPILE_STATUS.md). Verified locally on Rust 1.83.0: 26 tests pass, 0 fail (honest count; Wave-12 NOW's claim of 28 was off by two — corrected).

**Wave 15 (2026-05-22):** Canonical GF16 import — [`rings/ring-088-rust`](rings/ring-088-rust) lands as the first **honestly-authored** Wave-11 crate (439 LOC, 13 tests, including the first cross-kernel `mac_dot([phi,1/phi],[phi,1/phi]) ~= 3` identity check). R5-HONEST reclassification: the other 11 Wave-11 rings move from `off-disk` to `claimed-only` until they receive the same real-source treatment. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 16 (2026-05-22):** TNN ISA import — [`rings/ring-089-rust`](rings/ring-089-rust) lands the second honestly-authored Wave-11 crate (635 LOC, 15 tests). Includes `Trit`, `Word27`, balanced-ternary `trit_add` / `word_add` / `word_sub` per `specs/isa/ternary_arithmetic.t27`, a 9-opcode subset (`NOP`/`MOV`/`ADDI`/`ADD`/`SUB`/`NEG`/`LOAD`/`STORE`/`HALT`), and a `Cpu` fetch/decode/execute model with 27 registers (R0 hardwired to zero). `cpu_phi_identity_integer_projection` is the second cross-kernel anchor test, running `floor(phi) + floor(1/phi) + ceil(phi^2 - 2) = 3` through the CPU. `#![no_std]`, zero deps, verified locally on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 17 (2026-05-22):** Simulator import — [`rings/ring-090-rust`](rings/ring-090-rust) lands the third honestly-authored Wave-11 crate (547 LOC, 19 tests). Mirrors `specs/fpga/simulator.t27` byte-for-byte: `SimState` (5 variants), `SimConfig`, `SimResult`, `ProbePoint`, `TraceEntry`, plus constructor / query / time-conversion / validation helpers. All 13 spec `test` blocks and all 4 spec `invariant` blocks become `#[test]`s. `#![no_std]`, zero deps, all 19 tests green on first run (Rust 1.83.0). Wave-11 narrative claimed 2143 LOC; honest measurement is 547 LOC. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 18 (2026-05-22):** Stochastic Rounding import — [`rings/ring-091-rust`](rings/ring-091-rust) lands the fourth honestly-authored Wave-11 crate (462 LOC, 19 tests). Implements stochastic rounding (`sr_round_f32_to_i32`, `sr_quantize_f32`, `sr_quantize_batch`) on top of a deterministic seedable `SplitMix64` PRNG (Vigna 2014; reference seed-0 value `0xE220A8397B1DCDAF` checked in test). Two statistical tests verify unbiasedness over 10 000 draws against a 3-sigma bound. `sr_quantize_phi_unbiased` is the **third cross-kernel anchor test** in the project after Wave 15's `mac_dot_phi_identity` and Wave 16's `cpu_phi_identity_integer_projection`. `#![no_std]`, zero deps, all 19 tests green on first run. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 19 (2026-05-22):** Attention import — [`rings/ring-092-rust`](rings/ring-092-rust) lands the fifth honestly-authored Wave-11 crate (760 LOC, 28 tests). Mirrors `specs/nn/attention.t27` (SacredAttention) for the `no_std`-realizable subset: sacred constants (`NUM_HEADS=3`, `HEAD_DIM=81`, `EMBED_DIM=243`, `CONTEXT_LEN=81`, `ROPE_PAIRS=40`, `SACRED_GAMMA = phi^-3`, `SACRED_SCALE = 81^(-SACRED_GAMMA)`); `Trit` enum; primitives `ternary_matmul`, `add_residual`, `apply_softmax` (numerically stable max-subtract per head), `compute_scores` (Q.K^T with causal mask + sacred scaling), `weighted_values`, `cache_kv`. A private `exp_f64` (range-reduced Taylor series) makes softmax viable without libm. `attention_phi_identity_via_softmax_matmul` is the **fourth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through softmax-style normalization and ternary matmul. RoPE table init (cos/sin) and the full `sacred_attention_kernel` orchestrator are out of scope (R5-HONEST). Wave-11 narrative claimed 847 LOC; honest measurement is 760 LOC. All 28 tests green on first run (Rust 1.83.0). See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 20 (2026-05-22):** Sparse MoE import — [`rings/ring-093-rust`](rings/ring-093-rust) lands the sixth honestly-authored Wave-11 crate (950 LOC, 28 tests). No backing file under `specs/` (textbook algorithm, like ring-091's SR); design mirrors Shazeer-2017 / Switch-Transformer top-k routing with ternary expert weights and Trinity defaults (`NUM_EXPERTS=3`, `DEFAULT_TOP_K=1`, `DEFAULT_EMBED_DIM=243`, `DEFAULT_EXPERT_HIDDEN_DIM=729 = 3^6`). Exposes `MoEConfig`, `gate_top_k` (top-k selection + max-subtract softmax over selected logits), `expert_ffn` (two-layer ternary FFN with ReLU), `moe_forward` (composes gating + per-expert FFNs, allocation-free), `relu_inplace`, `load_balance_loss` (Switch-Transformer importance balance: 1.0 = uniform, `num_experts` = full concentration). A private `exp_f64` makes gating softmax viable without libm. `moe_phi_identity_via_gating_and_ffn` is the **fifth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through MoE gating + ternary FFN. Wave-11 narrative claimed 668 LOC; honest measurement is 950 LOC. All 28 tests green on first run (Rust 1.83.0). See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 21 (2026-05-22):** AGI Runtime import — [`rings/ring-094-rust`](rings/ring-094-rust) lands the seventh honestly-authored Wave-11 crate (1210 LOC, 32 tests). Mirrors `specs/runtime/{execute, instance, process}.t27` byte-for-byte: spec constants (`DEFAULT_TIMEOUT_MS=30_000`, `MAX_CONCURRENT_EXECUTIONS=16`, `POLL_INTERVAL_MS=100`, `TASK_ID_LENGTH=32`, `MAX_INSTANCES=256`, `INSTANCE_NAME_LENGTH=128`, `LOOKUP_TIMEOUT_MS=100`, `SPAWN_TIMEOUT_MS=5_000`, `PTY_COLS_DEFAULT=80`, `PTY_ROWS_DEFAULT=24`, `MAX_PIPE_BUFFER=65_536`); all nine spec enums; pure-state-machine `Promise`; fixed-capacity `Registry` (256 slots); and a Trinity-priority `Scheduler` (16 slots) with a phi-weighted credit policy (`Trit::Pos -> phi^2`, `Trit::Zero -> 1.0`, `Trit::Neg -> phi^-2`). `runtime_phi_identity_via_scheduler_credits` is the **sixth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through the scheduler's credit accumulator. Real syscalls, heap-backed containers, and async-runtime wakers are explicitly out of scope (R5-HONEST). Wave-11 narrative claimed 774 LOC; honest measurement is 1210 LOC. All 32 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 22 (2026-05-22):** phi-Adam optimizer import — [`rings/ring-095-rust`](rings/ring-095-rust) lands the eighth honestly-authored Wave-11 crate (808 LOC, 25 tests). Mirrors `specs/ml/optimizer/{adam, adamw}.t27`: spec constants byte-for-byte (`DEFAULT_LEARNING_RATE=1e-3`, `DEFAULT_BETA1=0.9`, `DEFAULT_BETA2=0.999`, `DEFAULT_WEIGHT_DECAY=0.01`, `DEFAULT_EPSILON=1e-8`, `PHI_BETA1 = 0.9/phi ~= 0.556`, `PHI_BETA2 = 0.999/phi ~= 0.617`); `AdamWConfig` with `defaults()` (classic AdamW) and `phi_preset()` (phi-damped betas) constructors; caller-owned `AdamWState<'_>` (no allocation); spec-named helpers (`compute_bias_correction`, `update_first_moment`, `update_second_moment`, `apply_weight_decay`, `compute_update`); full `step()` orchestrator with decoupled weight decay, bias-corrected `lr_t = lr * sqrt(1 - beta2^t) / (1 - beta1^t)`, moment recurrences, AMSGrad max-of-v, and the parameter update. Private no_std math (`pow_u64`, `sqrt_newton`) bypasses libm. `phi_adam_phi_identity_via_betas` is the **seventh cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through the optimizer's `pow_u64` and through the phi-damped moment update `m_1 = (1 - 0.9/phi) * phi = phi - 0.9`. Wave-11 narrative claimed 659 LOC; honest measurement is 808 LOC. All 25 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 23 (2026-05-22):** Quantization import — [`rings/ring-096-rust`](rings/ring-096-rust) lands the ninth honestly-authored Wave-11 crate (641 LOC, 42 tests). Mirrors `specs/numeric/formats.t27`: GF16 bit layout (`SIGN_MASK=0x8000`, `EXP_MASK=0x7E00`, `MANT_MASK=0x01FF`, `EXP_SHIFT=9`, `SIGN_SHIFT=15`, `BIAS=31`, `EXP_MAX=63`, `EXP_MIN=0`) byte-for-byte; full GF16 codec `gf16_to_f32` / `f32_to_gf16` (signed zero, denormals, normals, Inf, NaN, round-to-nearest with mantissa-overflow exponent carry); ternary quantization `f32_to_ternary` / `ternary_to_f32` with the spec's strict threshold `|x| > 0.5`; the `Format` enum (`Fp32`, `Fp16`, `Bf16`, `Gf16`, `Ternary`); `format_bytes`; and the `quantize_value` utility. A private `pow_u64` (fast exponentiation by squaring) replaces libm in `no_std`. `quantization_phi_identity` is the **eighth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through the GF16 codec: it computes `phi^2` and `phi^-2` via `pow_u64`, encodes via `f32_to_gf16`, decodes via `gf16_to_f32`, and verifies the sum lies within GF16 mantissa tolerance of 3.0 (~0.03 absolute). Wave-11 narrative claimed 464 LOC; honest measurement is 641 LOC. All 42 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 26 (2026-05-22):** Integration import — [`rings/ring-099-rust`](rings/ring-099-rust) lands the **twelfth** honestly-authored Wave-11 crate (763 LOC, 31 tests) and **closes the Wave-11 import series**. Mirrors `specs/pipeline/e2e_test.t27` byte-for-byte: constants `MAX_PIPELINE_STAGES=10`, `STAGE_INIT=0`, `STAGE_PARSE=1`, `STAGE_SEAL=2`, `STAGE_GEN=3`, `STAGE_TEST=4`, `STAGE_VERDICT=5`, `STAGE_SAVE=6`, `STAGE_COMMIT=7`, `STAGE_DONE=8`, `STAGE_FAIL=255`; functions `pipeline_run`, `pipeline_inject_failure`, `pipeline_progress`, `stage_name`; all 4 spec test blocks (`full_pipeline_pass`, `pipeline_fail_at_gen`, `pipeline_fail_at_test`, `progress_calc`); all 3 spec invariants (`stage_ordering`, `max_stages_sufficient`, `fail_distinct`). The `Pipeline` type wraps fixed `[u8; 10]` stage + `[bool; 10]` results buffers; the `Stage` enum (9 valid + `Fail`) carries `code` / `from_code` / `next` / `is_terminal` / `name`. `#![no_std]`, `#![forbid(unsafe_code)]`, heap-free. `integration_phi_identity` is the **eleventh cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through integer projection (`floor(PHI)+floor(PHI_SQ)=3`), `pow_u64` numeric witness, pipeline progress arithmetic (`progress(9,9)==100.0` and `progress(3,9)==100/3` within 1e-9), and mass-conservation `PHI_SQ + PHI_INV_SQ == TRINITY` to within 1e-12. Wave-11 narrative claimed 1127 LOC; honest measurement is 763 LOC. **Wave-11 series complete**: all 11 narratives have honest source on disk, 8 817 honest LOC, 336 tests, 11 live cross-kernel anchors. The `claimed-only` placeholder table in [COMPILE_STATUS](rings/COMPILE_STATUS.md) is now empty. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 25 (2026-05-22):** World Model import — [`rings/ring-098-rust`](rings/ring-098-rust) lands the eleventh honestly-authored Wave-11 crate (779 LOC, 29 tests). Mirrors three specs byte-for-byte: `specs/brain/unified_state.t27` (`BrainState`, `ConsciousnessState`, `Mood`, `ArousalLevel`, `Layer`, `REGION_COUNT=27`, `LAYER_COUNT=3`, `REGIONS_PER_LAYER=9`, `PHI`/`PHI_INV`/`PHI_SQ`/`PHI_INV_SQ`/`TRINITY`); `specs/ml/rl/dqn.t27` (`Transition { state, action, reward, next_state, done }`); `specs/brain/cognitive_loop.t27` (`COGNITIVE_PHASE_COUNT=5`: sense, evaluate, decide, act, consolidate). `WorldModel` is a bounded recorder: fixed `[BrainState; MAX_STATE_HISTORY=16]` history, fixed `[Transition; MAX_TRANSITIONS=32]` replay buffer, inline `STATE_DIM=8`, plus `snapshot`, `record_transition`, `step_phase`, `run_one_cycle`, `verify`, `reset`. The `verify` routine enforces monotonic `cycle_count` and a `phi_coherence in [0,1]` invariant. `#![no_std]`, heap-free. `world_model_phi_identity` is the **tenth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through (a) integer projection `floor(PHI_SQ) + floor(PHI) = 3`, (b) `pow_u64` numeric witness, and (c) mass-conservation `PHI_SQ + PHI_INV_SQ == TRINITY` to within 1e-12. Wave-11 narrative claimed 920 LOC; honest measurement is 779 LOC. All 29 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 24 (2026-05-22):** Chain-of-Thought import — [`rings/ring-097-rust`](rings/ring-097-rust) lands the tenth honestly-authored Wave-11 crate (823 LOC, 29 tests). Mirrors `specs/ar/proof_trace.t27` byte-for-byte: `MAX_STEPS=10` (DARPA CLARA bound on reasoning chain length); K3 ternary logic (`Trit::{True=1, Unknown=0, False=-1, Null=2}`) with `k3_and` (min lattice), `k3_or` (max lattice), `k3_not`; fixed-capacity heap-free `ProofStep` (interned ASCII operation name up to 24 chars, fixed-arity inputs up to 3 trits, `output`, `timestamp_us`); `ProofTrace` with `[ProofStep; MAX_STEPS]` buffer + `start_timestamp_us` / `end_timestamp_us` / `verified` flag; operations `new_proof_trace`, `add_step`, `verify_trace`, `trace_length`, `is_at_capacity`, `finalize_trace`, `step_at`, `format_trace`, `trit_to_string`; `VerifyStatus::{Valid, Empty, TooManySteps, NullOutput(usize)}` enforcing all three spec invariants (`empty_trace_fails`, `trace_verification_catches_overflow`, `valid_trace_passes`). The crate is `#![no_std]` and heap-free; `format_trace` writes into a caller-supplied buffer. `cot_phi_identity` is the **ninth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through a 6-step bounded reasoning chain: symbolic premises, `k3_and`, a numeric-witness step that evaluates `pow_u64(phi, 2) + pow_u64(phi, -2)` and produces `True` iff the result is within 1e-9 of 3.0, a `k3_or` alternative-path step, and a conclusion -- then verifies and finalises the trace, plus a separate mass-conservation hook for φ²-weighted Pos and φ⁻²-weighted Neg priorities. Wave-11 narrative claimed 624 LOC; honest measurement is 823 LOC. All 29 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**DOI:** [10.5281/zenodo.19456875](https://doi.org/10.5281/zenodo.19456875)