# Full status, both tracks — one page for whoever wakes up (W983, 2026-08-22)

> **W983 — THE CONTROL WAS A CONSTANT, AND WHEN IT FINALLY RAN IT REFUTED THE STORY.**
> `tri clauses` reads the *netlist*, not the source, and asks which `c_*` clauses are driven by
> logic: **28 clauses across 7 wrappers, 14 folded to the literal 1**. An unwritten probe collapses
> to its `INIT`; two structurally identical instances are **merged**, so comparing them is a
> tautology. T555 protected the clause **operands** from folding — nobody looked at the
> **comparison**. **This reinterprets published rows:** `gft_sadd`'s *PASS — 4 of 4 on the die* is
> **1 of 1 measured**; `gft_smul`'s `1010` is **2 of 2 measured clauses FAIL**. It does *not*
> reinterpret the MAC's `0011` — that was read against the pre-W978 spec, and the guard added then
> is what makes the clause fold now. **`(* keep *)` does not stop a fold**; two structurally
> distinct counters carrying equal values do. 4 clauses, 0 folded, LUT 452 → **696** — the rise is
> the logic the folding had deleted. **Then the repaired control failed.** One netlist (696 LUT,
> 160 CARRY4), three placements: `1111` / `1001` / `1011`. `c_self` — two instances of one function
> with the **same** operand order — is false at two of three, and at seed 7 it fails while
> commutativity **passes**. **Six waves were framed as "swapped operands disagree". Any two
> instances disagree.** T836–T838, lessons 1504–1506, **410 derived checks**. **Readiness 94 %** —
> down one point, and the drop is the honest signal: the corpus is more correct and now carries a
> named debt.

> **W982 — THE REPRODUCER SHRANK 46 %, AND THE PROOF MOVED TO THE ARTEFACT THAT FAILS.**
> `gft_commmin_jtag`: three `GftSmul` instances instead of five, **430–452 LUT instead of 798**,
> and it reproduces `c_self` TRUE / `c_comm` FALSE exactly. **The verdict tracks the netlist**:
> both 430-LUT builds PASS, both 452-LUT builds FAIL — and the two netlists differ only because
> `t27c` forced `JTAG_CHAIN` to the site nextpnr chose, a parameter that cannot reach the
> arithmetic yet moves 22 LUT and 7 CARRY4. **The seed mapping inverts** against the big design.
> **The mapped netlist commutes — proved.** The miter is synthesised by the *exact* script the die
> runs, Xilinx cells mapped back to logic, `sat -verify -prove neq 1'b0`: `gft_smul` **277 cells /
> 1822 SAT variables**, `gft_sadd` **5020 / 48200**, no counterexample. W977 proved the *module*;
> this proves the two *folded cones that fail*. **The front end is exonerated — the fault is at or
> below place-and-route.** Now `tri miter`. **Third cell-level method, third control-saved
> retraction:** recovering post-PnR LUT functions from nextpnr's `--write` JSON, the control pair
> — two builds that both PASS — disagrees on **591 of 1581** cells. With W981 this is a rule: no
> cell-level comparison decides function preservation across a repacking placer. Own defect:
> `grep -q` under `pipefail` turned the proof into a refutation. T833–T835, lessons 1501–1503,
> **393 derived checks**. **Readiness 95 %.**

> **W981 — NINE DIE READS KILLED A HYPOTHESIS, AND W977's OWN NEXT STEP.** W977's result
> **reproduces on a changed bench** (three cables at 1:4 became one at 1:5): seeds 42/7/31337 →
> `1111`/`1101`/`1111`, same Fmax to two decimals, each bracketed by a wrong-part control.
> **The BSCAN-site hypothesis was 5/5 and is refuted**: holding the seed and moving the site
> gave FAIL at BSCAN3, PASS at BSCAN1, PASS at BSCAN2. The site had moved *with* the placement
> in every sample that suggested it. **Timing tested physically for the first time** — same
> seed, same site, `slowclk` halved: `1101` both times; reported Fmax of the failures
> (17.36/18.14/17.91) sits **inside** the passing range (16.41–18.57). **W977's planned FASM
> diff cannot decide anything**: the logic-LUT multisets of two *passing* builds differ by
> 508 words, because pin permutation rewrites INIT bits without changing the function.
> **Arithmetic exhaustively cleared**: commutativity 0 counterexamples in 2 359 296 pairs;
> identity fails on all 48 128 non-representable words and on **exactly one** representable
> one — negative zero, normalised to `+0`. New instrument `tri domain`: **17 live sources,
> 17 leave the representable set**. **Three repairs, all ours:** `tri audit` had been dying at
> the `info` line W980 added (five gate rows unrun for a wave); the **seven oracles every
> published comparison derives from were never committed** and now live in `conformance/oracles/`;
> `gft_xorpercep`'s clause was fed a word the format cannot express. T829–T832a, lessons
> 1496–1500, **375 derived checks**. **Seed 7 survives everything.** **Readiness 94 %.**

> **W980 — THE DIE ASKS 36 QUESTIONS; THE SUITES ASK SIX BY NAME.** `tri coverage` matches
> wrapper clause names to spec test names: **9 wrappers, 36 clauses, 30 with no same-named test
> — 83 %**. Worst `gft_xorpercep` at **4 clauses / 1 test**; best `gft_signed_mac` at **4 / 4**,
> the suite strengthened by hand in W978 **from the die's own clauses** — the instrument ranks
> the repaired spec top **without knowing that history**, which is the one row whose answer was
> known in advance. **The tool over-reports and says so:** `gft_sadd` lists four uncovered and
> passes **`1111`**. So it is an **`info` line, not a gate** — while `tri rungs` and `tri guards`
> do gate, because they are exact. **A check's precision decides whether it may block**: the
> permanently-red `disk` line lost all meaning over twelve waves and then hid a real ENOSPC.
> **Hard signal:** `gft_smul`'s `comm`+`ind` are listed here **and** false on silicon.
> T828/T828a, lessons 1494/1495, **334 derived checks**. **Readiness 93 %.**

> **W979 — THE CORPUS AUDIT CLOSED THE CLASS AND PREDICTED A 141-WAVE-OLD HARDWARE FAILURE.**
> **134 definitions across 26 specs** scanned; **exactly one** unguarded copy left —
> `gft_signed_dot4.t27 :: smul`. **That design's annihilation clause was measured FALSE on
> hardware in W838** (`0xa5a5a1b4`, clauses `1011`): the static check produced the explanation
> for a failure that had sat unexplained for 141 waves, **without a board**. Fixed; simulation
> 1 → 2 tests, the new one derived from the die clause. **`tri guards` now gates the class** in
> `tri audit` — *51 definitions of smul/sadd across 30 specs, all guarded* — and the counts
> differ from the audit's **on purpose, with the reason recorded**. **The incidental finding is
> larger:** the ladder has **no shared arithmetic module**; every spec re-declares what it
> needs, and 2 of 51 had drifted. **A checker is a tourniquet** — the durable fix is an import
> mechanism, recorded as a standing recommendation. T827/T827a/T827b, lessons 1491–1493,
> **324 derived checks**. **Readiness 92 %.**

> **W978 — FIXED IN THE SPEC, AND THE DEFECT TURNED OUT TO BE EXPENSIVE.** Five guard lines —
> three in `smul`, two in `sadd`, all present verbatim in the primitive specs — take the MAC from
> **6 466 → 5 484 LUT (−15.2 %)**, **1 237 → 961 CARRY4 (−22.3 %)** and **9.14 → 9.85 MHz
> (+7.8 %)**. **A correct early return is cheaper than the path it skips.** Spec tests **4
> PASSED**; the W976 testbench goes from **64 violations to 0**. **Fixing `smul` alone was not
> enough** — the residue became a constant 512 — and the new **`zero_annihilates` test, derived
> from the die's clause, caught the incomplete repair immediately** while the old suite still
> passed 2/2. **This invalidates published numbers:** every cost figure for this operator,
> including the MHz/kLUT curve, priced the **defective** build. **Unverified on silicon:**
> nextpnr hit the 600 s cap and the BSCAN check failed — the chain assignment **moved** (1 → 2),
> so the spec change perturbed the harness that would confirm it. T826/T826a, lessons 1489/1490,
> **316 derived checks**. **Readiness 91 %.**

> **W977 — THE MAC KEEPS A PRIVATE COPY OF THE MULTIPLY, AND IT LOST THE ZERO GUARD.**
> `GftSignedMac` is **flat — zero instances** of `GftSmul`/`GftSadd` — and repeats the identical
> hidden-bit line `prod = __mul_noop((512 + am), (512 + bm))` **without** the guards `GftSmul`
> carries (`if (a==0)` @258, `if (b==0)` @262; "zero" never appears in the MAC). **One missing
> guard explains both die results**: smul's ZERO is **true** on silicon, the MAC's is **false**.
> Three cheaper hypotheses were **refuted first** — `sadd(0,0)=0`, `smul(0,x)=0`, smul
> commutative over 20 pairs — and W976's own measurement was re-verified **gated on `ready`**
> (identical, 16/16). **Seven operators to the die:** `gft_sadd` **1111** and `gft_train1`
> **1111** pass; `gft_smul` **1010** and `gft_signed_mac` **0011** fail; `gft_bitnet_neuron`
> **FAILS timing** at 16.63 vs 70.77 MHz; `xorpercep` and `dot4` are **ABSENT** (600 s PnR cap).
> **Every operator passes its own simulation suite; four of five verdicts are red.** T825/T825a/
> T825b, lessons 1487/1488, **304 derived checks**. **Readiness 90 %.**

> **W976 — ZERO IS NOT AN ANNIHILATOR, AND THE DIE ONLY HAD TO ASK.** The failing word
> `0xa5a5334e` decodes exactly: **c_zero=0, c_comm=0**, c_cancel=1, c_ind=1, beat=1, ok=0.
> **ZERO reproduces in simulation immediately** — driving the *same generated Verilog* with the
> wrapper's operands gives **64 violations in 64 points**: `mac(0,x,0,y)` = **`512 + 4x`**
> instead of 0, in **25 µs**. **Multiplying by zero does not give zero**, and the residue is
> linear in one operand. *The die's whole contribution was to ask a question nobody had asked.*
> **COMM is confirmed on silicon and unexplained off it** after three targeted attempts — a
> 64-point sweep, 32 diverse probes incl. `0x80000000`/`0xFFFFFFFF`, and **960 per-cycle
> samples** testing the sticky-latch/ready-race mechanism: **0 transients, 0 ready skew**.
> Stated as unexplained deliberately. **The suite covered exactly the clauses that pass** —
> ZERO and COMM untested, and the spec's tests assert on **fixed constants** while every die
> clause drives **live operands**. T824/T824a/T824b, lessons 1485/1486, **293 derived checks**.
> **Readiness 89 %.**

> **W975 — THE FORMAT'S SIGNED MAC FAILS ON SILICON WHILE PASSING EVERY TEST IT HAS.** Same
> bench, control satisfied (a foreign bitstream forced **`Done=0`** first). `gft_sadd`: **3/3**
> in simulation, **`clauses=1111`, `ok=1`** on the die. `gft_signed_mac`: **2/2** in simulation,
> **`clauses=0011`, `ok=0`**, **`beat=1`** — loaded, alive, **answering incorrectly**. Timing is
> not the explanation: **9.14 MHz against a 2.21 MHz target, 4.1× margin.** **The sharper
> finding is about the tests:** the MAC ships **two** simulation tests; its die check evaluates
> **four** clauses, and the two that fail are **not covered in simulation**. **Not "the
> simulator lies" — a coverage gap made visible by hardware**, found after synthesis, PnR,
> bitstream and a physical load. **L4 is satisfied by *having* tests, not by their matching the
> project's own stronger oracle.** Remedy is free: derive the sim tests from the die clauses.
> T823/T823a, lessons 1483/1484, **281 derived checks**. **Readiness 88 %.**

> **W974b — THE SILICON ANSWERED.** On `xc7a200tfbg676-1`, IDCODE `0x3636093`, board `1:5`:
> **`Done = 1`** and **`0xa5a5a5a7`** read back through `USER2` — magic `0xA5A5A5A`, **`ok=1`,
> `beat=1`**. Spec → Verilog → yosys → nextpnr → FASM → frames → bitstream → load → read, every
> stage demonstrated, about a minute. **The axis that had no number has one, and it is a pass.**
> **The first attempt failed on the ADDRESS, not the artefact:** the service defaults to the
> three-cable bench (`1:4/1:6/1:8`) while this bench has **one** cable at **`1:5`** — the
> identical bitstream then passed. *"ALL ZERO — dead chain or no BSCANE2"* names the wrong
> suspect; **a read of all zeros must implicate the address first.** **What this does NOT
> establish:** anything about the format. What answered is a **classifier** and its verdict is
> a liveness check; the format's operators are built, timed and **unread**. T822/T822a, lessons
> 1481/1482, **270 derived checks**. **Readiness 86 %.**

> **W974 — THE FORMAT'S OWN OPERATORS ON THE REAL PART.** Spec → bitstream on
> `xc7a200tfbg676-1`, Verilog generated throughout: `gft_sadd` **1 312 LUT / 18.24 MHz**,
> `gft_signed_mac` **6 466 / 9.14**, `gft_signed_dot4` **12 872 / 5.50**, all timing **PASS**;
> the classifier is **123 / 80.35** but constrained on a **different clock**. **MHz/kLUT falls
> 13.9 → 1.41 → 0.43 across the comparable three — a factor of 32** over a 10× growth in area,
> and the shape is not linear: `sadd`→`mac` is 4.9× the LUTs for half the frequency. **A
> MHz/LUT headline must name the operator and the clock**; DUT-equivalents (3.10 vs 1.06) are
> the honest denominator. **`dot4` is incomplete for two unrelated reasons:** nextpnr hit a
> 600 s cap — reported **`ABSENT`, not `FAIL`**, so nothing was proven unroutable — and a real
> **BSCAN chain/site mismatch** (`JTAG_CHAIN(3)` vs `BSCAN4` wired). T821/T821a, lessons
> 1478/1479, **262 derived checks**. **Readiness 83 %.**

> **W973 — FIRST SILICON NUMBERS, AND THE BLOCKER WAS NEVER DOCKER.** Built from the spec via
> `t27c silicon` on **`xc7a200tfbg676-1`**: **123 LUT, 52 CARRY4, 0 DSP48E1, BSCANE2 ×1**,
> **Fmax 80.35 MHz** against a 70.77 MHz target — **PASS, 13.5 % margin** — 1.19 DUT-equivalents
> of arithmetic on the die, bitstream **9 730 834 B** with the sync word at offset 230.
> **Thirty seconds end to end, and no Docker was involved.** **Four preconditions, measured
> separately:** Docker **wedged** (one PID alive 2 d 1 h) not absent; the **native** openxc7
> toolchain present all along and never used; the cable present; **the JTAG chain EMPTY — the
> target does not answer.** The reported blocker was none of them. `docker info` **hangs**,
> killing any command it joins; `--detect` prints **`empty`** for a live cable with a silent
> board. New: **`tri bench`**, four non-blocking probes, one verdict each. **This is synthesis
> and timing, not a die verdict** — the remaining gap is one physical action. T819/T820,
> lessons 1476/1477, **238 derived checks**. **Readiness 81 %.**

> **W972 — ALL THREE DIRECTIONS, AND DOCKER WAS NEVER ABSENT.** **(1) The last convention is
> gone:** against a float with **real subnormals** (normaliser paid for) TNF16 is **450.29 vs
> 501.29 — 10.2 % cheaper**, while that float carries **524 287** values to TNF's **516 097**;
> against the FTZ peer it stays **+2.0 %** on an identical grid. Subnormals cost exactly
> **+60 cells** and buy **8 191** values. **The φ-lattice is a float that declined subnormals,
> priced 9 decoder cells above one that did the same.** **(2) Docker:** twenty-two waves said
> "not running" — it **is** running. `com.docker.backend` under two PIDs, socket present since
> 19 Aug, and `docker version` **hangs**. **Wedged, not absent**: quit and reopen, not start.
> It hid because `docker info` hangs rather than fails, killing any command it was bundled
> with. **(3) master restore authorised, preserved first** (`origin/orphan-master-fead099c2`),
> then **rejected by the repository's protection rules** — not bypassed. T817/T818, lessons
> 1472/1473, **228 derived checks**. **Readiness 80 %.**

> **W971 — `origin/master` WAS REPLACED BY AN ORPHAN COMMIT; NOTHING IS LOST.** `git status`
> moved from *126 / 117* to *2556 / 1* — read as a number that is more divergence; in fact
> **`git merge-base` now exits 1** and the histories share no commit. Master went from
> **2545 commits** (`ca4234e20`, 18:00) to **1** (`fead099c2`, 18:48) — a fresh **orphan
> root**, its own tip and root, which no rebase can produce. **Nothing is lost:** the old
> history is on `origin/fix/coq-phifloat-binary64-name-collision` (2547 commits), and this
> branch is intact at **2556**, 0/0 against its own origin ref. **The loop did not repair it**
> — restoring a shared branch is a force-push, irreversible for anyone who has fetched, and it
> would discard the human's new commit. It **noticed** (merge-base is now gated in `tri
> audit`), **preserved the evidence** in `docs/reports/MASTER-ORPHANED-W971.md` before the
> reflog expires, and **named the remedy** with `--force-with-lease`. T816, lessons
> 1470/1471. **Readiness 79 %** (unchanged — no science this wave).

> **W970 — THE CLASS CLOSES: ONE INVERTED SIGN, TEN ACCURACY CELLS, NONE SIGNIFICANT.**
> Every record from a substituted rig is regenerated on `TNFFormat(3, 4)`. Across
> `activations`, `conv` and `accuracy_seeds` — three rigs, two tasks, five seeds, paired —
> the largest difference is **0.068 pp**, the largest **|t| = 1.48**, signs mixed. An 11-bit
> format spanning **126.91** binades and a 10-bit spanning **30.95** are statistically
> indistinguishable at this quantisation. **Complete assessment:** the substitution inverted
> the **area** sign (W963) and changed **nothing** in accuracy. The asymmetry has a mechanism
> — area depends on decode-table structure (decoder 29 vs 18 cells), accuracy on grid density
> near the working range, which both over-resolve. **"Approximately right" is a claim per
> axis, and must be measured per axis.** The class opened in W954 closes here, sixteen waves
> later, with the damage **quantified rather than assumed**. T815, lesson 1469, **222 derived
> checks**. **Readiness 79 %.**

> **W969 — THE RECORD CATCHES UP, AND THE REPLICATION PROMISE BECOMES TRUE.**
> `activations.py` re-run on `TNFFormat(3, 4)`: MNIST **−0.008 / +0.018 pp**, Fashion
> **−0.016 / −0.068 pp**, all **|t| ≤ 1.48**, sign changing — a **second independent record**
> confirming the asymmetry: area inverted, accuracy untouched. **And the bigger finding:**
> `FALSIFY-ME.md` has invited replication since W948d, and that was true of **two rigs out of
> twenty** — **fifteen files still named this machine's directories**. Now **zero**; `tri
> audit` gates it: **32 rigs parse, none names a path outside its tree**. Two self-inflicted
> defects, both caught at once: the automated edit **broke `stability.py`** (caught by parsing
> every rig, not running one), and the new gate **killed the audit precisely on success** —
> `grep -l` exits 1 on no match, and under `pipefail` that aborted everything the instant the
> corpus became clean. T814/T814a, lessons 1466–1468, **216 derived checks**.
> **Readiness 78 %.**

> **W968 — CLASS CLEARED, RECORDS ANNOTATED, AND ONE FIX WAS HALF-APPLIED.** All five rigs
> now instantiate `TNFFormat(3, 4)`; **`tri rungs` is wired into `tri audit`** — 34
> instantiations, **0 standing alone**. Six records carrying an unqualified "TNF8" got a
> `_format_note` pointing at what supersedes them. **Two needed nothing**:
> `accuracy_coordinate_w938.json` keys `"TNF8 (E_t=4,M=3)"` and `structural_w942.json` stores
> `physical_bits` — they wrote the **object** into the record, not the name. **The half-fix:**
> the regex left `oracle_rtl.py` reading `("tnf8", 11, … TNFFormat(3, 4))` — width **precedes**
> format on that line — which would have enumerated **2¹¹ codes for a 10-bit format** and run
> happily, fitting a clean line through 1 024 phantom entries. **Worse than the defect it
> replaced**, and caught only by reading the diff. T813/T813a, lessons 1464/1465.
> **Readiness 77 %.**

> **W967 — THE SUBSTITUTION IS A CLASS: FIVE RIGS STILL BIND A RUNG'S NAME TO A NON-RUNG.**
> `tri anomaly` checks record *shape* and can never see a name bound to the wrong object, so
> **`tri rungs`** now resolves every `TNFFormat(…)` call to its width and rung across both
> ladder versions. **34 instantiations, 5 standing alone**: `accuracy_coordinate`,
> `accuracy_seeds`, `activations`, `conv`, `oracle_rtl` — all binding "TNF8" to `(4, 3)` at
> 11 bits. Files that also instantiate `(3, 4)` are **labelled controls**, not defects.
> **Damage splits by axis:** the census one inverted the area sign, the four accuracy ones
> are within ±0.1 pp. **The tool's own first run was wrong** — regex over source text flagged
> a file for a *comment* mentioning the substitution, and another for a line deleted two waves
> ago; rewritten to walk the **AST**, 36/6 became 34/5. Both retracted flags were the tool's.
> T812/T812a, lessons 1462/1463. **Readiness 76 %.**

> **W966 — SAME GRID, TWO PER CENT DEARER; W965's "MORE VALUES" WAS SUBNORMALS.** TNF's
> decoder flushes offset 0 to zero — **the format has no subnormals** — and the floats W965
> compared against did. That is where the float's extra values came from, and it is not free:
> subnormals need a leading-zero normaliser the rung never pays for. **W965 priced a design
> choice as a property of the lattice.** Rebuilt with the float in TNF's own discipline and
> priced structurally over **every code**: TNF16 **516 097** values / **127.00** binades /
> **450.29** cells against `fp19 e7m11`'s **516 096** / **126.00** / **441.29** — **+2.0 %**.
> At ten bits TNF8 `(3,4)` is **961**/**30.95**/**230.57** against **960**/**29.95**/**225.57**
> — **+2.2 %**. **The grids are the same grid; the gap is in the decoder** (27 vs 18, 18 vs
> 13). A stronger negative result than W965's, because "same grid, 2 % dearer" cannot be
> answered with "your axes were unmatched". TNF16 reproduces the W942 record **exactly**.
> T811, lessons 1460/1461, **208 derived checks**. **Readiness 75 %.**

> **W965 — RUNG SIXTEEN IS STRICTLY DOMINATED, AND THE GRIDS ALONE SAY SO.** No training,
> no synthesis. Factoring every grid value as `odd · 2^s` gives the pair that T807 showed
> fixes lane cost at equal width. At **17 bits** TNF16 `(4,9)` is `(10, 135)` — and so is
> `fp17 e7m9`, which carries **131 071 values against 129 025** and **136 binades against
> 127**. At **19 bits** TNF16 `(4,11)` is `(12, 137)` — and so is `fp19 e7m11`, with
> **524 287 vs 516 097** and **138 vs 127**. **Same cost, more range, more values: strict
> domination**, not parity. The φ-lattice loses ~**1.6 %** of its code space to
> unrepresentable or duplicate points. A conventional-exponent peer at the same width runs
> a **166-bit** bus against the rung's **290** — so the rung is on **neither frontier**.
> Scope: the cost equality is a **prediction** from T807 (validated at 6 and 10 bits); a
> case table over 2¹⁹ codes is not buildable. T810, lessons 1458/1459, **185 derived
> checks**. **Readiness 74 %.**

> **W964 — THE EIGHTH RUNG HAS AN ACCURACY AT LAST, AND IT IS PARITY.** MNIST, 5 seeds,
> three recipes: TNF8 `(3,4)` against a width-matched `fp10 e5m4` is **−0.010 / −0.018 /
> +0.060 pp** — none significant, sign changing between recipes. **Zero failures in all 45
> runs, including under the learned scale** that destroyed `fp6 e2m3` 28/40 times at six
> bits: exactly what T801 predicts, since every ten-bit grid spans 31+ binades against 5.91.
> **The instability studied for eight waves has a width above which it does not exist.**
> **The asymmetry:** the same substitution **inverted the area sign** (W963) and is
> **harmless in accuracy** — so "the published numbers are approximately right" cannot be
> inferred from one axis. Ladder now: rung 4 parity, rung 8 parity on both axes, **rungs 16+
> unmeasured** with a width that depends on which ladder version was imported. T809, lessons
> 1456/1457, **147 derived checks**. **Readiness 73 %.**

> **W963 — THE PUBLISHED TNF8 PENALTY HAD THE WRONG SIGN.** The census rig binds `"tnf8"`
> to `TNFFormat(4,3)` in its own table. Re-measured in **this project's original metric**,
> the ladder's true rung `TNFFormat(3,4)` is **212.57** consumer cells against a
> width-matched `fp10 e5m4`'s **214.57** — **0.9 % cheaper**; the substitute is **270.57**
> against **257.57** — **5.0 % dearer**. Not an overstatement: **the wrong sign**. Decoder
> alone **12.00 vs 29.00** (2.4×) — the substitute inherits TNF16's four-trit exponent, so
> its table spans 127 binades and resists factoring. In the MAC-lane metric the same rung is
> **+1.1 %**: **two metrics straddling zero, which is the strongest available statement of
> parity**. Still open: accuracy for the true rung is **unmeasured** — those rigs need
> datasets. T808, lessons 1454/1455, **130 derived checks**. **Readiness 72 %.**

> **W962 — UNBLOCKED AFTER SIX DEAD WAVES; THE RATE COEFFICIENT IS WITHDRAWN.** The volume
> sat at zero for six waves. `Bash` dies there (capture file opened before exec) and so does
> **`Write`** — it stages through `path.tmp.NNNN`, so truncating a file to free blocks fails
> identically. **At true zero the session has no lever.** Nothing was lost: `tri anomaly`
> and eight curve points sat in the working tree and were committed the moment writes
> returned. **Science:** T806's "≈2.4 cells per binade" is **withdrawn** — the full split
> sweep is **not monotone in range** (`fp6 e1m4` spans *less* and costs *more*, 80 vs 74;
> same at ten bits, 230 vs 215), and a binade-only fit gives 6.49 cells/binade at R² = 0.85.
> **The replacement is exact and unfitted:** formats sharing **(odd-mantissa bits, max
> shift)** cost the same — TNF4/`fp6 e4m1` **108 vs 106**, TNF8/`fp10 e5m4` **380 vs 376**.
> Every format in both W954 and W955 records reproduces **exactly**. T807, lessons
> 1452/1453, **118 derived checks**. **Readiness 71 %.**

> **W954 — THE LADDER'S RUNG WAS NEVER MEASURED, AND AT MATCHED RANGE WE ARE AT PARITY.**
> `LADDER` defines TNF8 as `TNFFormat(3,4)` — **10 bits, 30.95 binades**. Every rig here
> instantiated **`TNFFormat(4,3)`** — **11 bits, 126.91 binades**, TNF16's exponent field
> with a cut mantissa. Every "TNF8" number published describes that, not the rung. And the
> module contradicts itself: `LADDER` is **v1-research** while `DEFAULT_LADDER_VERSION` is
> **v2-spec**, disagreeing above rung 8 (TNF16 **17 vs 19 bits**) — **which resolves #644**:
> 16 by name, 17 by research, 19 by spec. **Second result:** the +46 % of W953 priced
> *range*, not the lattice. `fp6 e4m1` — an ordinary float widened to **15.58** binades —
> costs **106** against TNF4's **108** (**+1.9 %**) and carries **31** values to 28.
> At ten bits on the true rung: **380 vs 376, +1.1 %**. Cost is ≈**2.4 cells per binade per
> lane**, for floats and the lattice alike. **No configuration measured here has the lattice
> winning.** T805/T806, lessons 1450/1451, **101 derived checks**. **Readiness 70 %.**

> **W953 — THE BRACKET CLOSES AT +46 %.** The float-style lane — multiply odd mantissas,
> add shifts, align — costs **108 / 82 / 74** cells: **1.46×**, not 4.83×. It is cheaper
> than the fixed-point lane **for every format** (108 vs 768; 74 vs 159), so fixed point
> was simply the wrong design. TNF4 has the **narrowest multiplier** (2-bit odd mantissa)
> and the **widest aligner** (15 shifts) — the φ-lattice's trade, in gates. Built without
> assuming a field layout: the `(1.mantissa, exponent)` form **fails on both fp6 grids**
> (truncated bottom binade), so the rig factors `M = odd·2^s`, exact for any grid.
> **Correction to our own headline:** the 51.29-vs-50.29 figure is decode **plus a multiply
> by a constant** (8.0 + 43.29), not a decoder — a constant operand folds away the very
> width range forces; with both operands varying it is 1.46×. **Six bits, complete: no
> accuracy, no stability, +46 % of the MAC datapath.** New: **`tri cost`**. T804, lessons
> 1448/1449, **91 derived checks**. **Readiness 68 %.**

> **W952 — RANGE IS A BILL PRESENTED AT THE ACCUMULATOR.** Every census here priced a
> *decoder* (2 % dearer). A datapath also multiplies and accumulates, and range forces the
> width: TNF4 needs **17** bits per value, **33** per product, a **38-bit** block-32
> accumulator, against `fp6 e2m3`'s **7 / 13 / 18**. Measured: a fixed-point MAC lane costs
> **768 cells against 159 — 4.83×** (fixture 0.0, R² 1.00000). The part no implementation
> escapes — the accumulator — is **48 vs 23**, i.e. **+0.78 cells/element** amortised over
> 32, about **+1.5 %**. **So the honest answer is a bracket: +1.5 % … +383 %**, and the
> float-style lane that would pin it is **not measured** — quoting 4.83× alone would repeat
> the error the 2 % figure made. With W949–W951: at block 32 range buys no accuracy and no
> stability, and costs width. Rig defect caught: `yosys -q` silenced `stat`, and four zeros
> fitted a perfect line (R² = 1.00000) — the rig now refuses a zero reading. T803/T803a,
> lessons 1446/1447, **80 derived checks**. **Readiness 66 %.**

> **W951 — SATURATION OBSERVED, THE SWEEP REDONE, A PROXY RETIRED.** The mechanism is
> now measured, not inferred: the rig logs `max|x|/s / max(grid)` directly. **Everything
> overshoots** — including all 90 successes — so the binary "saturates ⟹ fails" agrees on
> **6.7 %** and is dead. What separates is **magnitude**: among 45 learned-scale runs the
> worst success overshoots **1 510×**, the best failure **84 775×**, **non-overlapping**,
> gap 56×. A computed power-of-two scale bounds the overshoot in **[1, 2)** by
> construction — measured max **2.0000** over 90 runs — which is why it never fails.
> **Sweep redone under that scale on all three tasks: 0 failures in 90 runs**, against 9
> under the learned scale on the same tasks and seeds. TNF4 has still never failed
> anywhere — a true claim about tolerating a bad recipe, **not** about deployment. New:
> **`tri sweep`** derives the whole table from every record. T801/T802, lessons
> 1443/1444, **62 derived checks**. **Readiness 64 %.**

> **W950 — THE MECHANISM RECOVERED, AND IT KILLED THE LAST CLAIM.** The traces held
> the answer: in every failure the scale **collapses**, so `x/s` grows and what matters
> is headroom **above** the operating point — `max(grid)` = 3072 / 28 / 7.5. Over all
> **120 traces**, saturation and failure agree **90.8 %**; `fp6 e2m3`'s 28 failures
> saturate **28/28**. TNF4's scale collapses **32.4×**, twenty times harder than the
> competitor's *successful* runs, and never fails — **not stability, headroom** (T799).
> **The prediction that follows killed us:** a scale that cannot collapse should end the
> failures. With the OCP-style **computed power-of-two** scale, **all 30 runs succeed** —
> `fp6 e2m3` goes **28/40 → 0/5** at *unchanged* per-tensor granularity, so the block
> size explained nothing and the **learned scale was the whole effect**. Paired
> per-tensor, TNF4 is **−0.376 pp** vs `fp6 e2m3` (t = −7.24, **0/5** seeds favour it).
> **At six bits: no advantage on cost, none on accuracy, and a deficit under the
> standard recipe.** T799/T800, lessons 1439–1442, **53 derived checks** now gating
> `tri audit`. **Readiness 62 %.**

> **W949 — THE MECHANISM WAS COMPUTED IN A CONVENTION THE EXPERIMENT NEVER USED.**
> The range explanation (`zeroes below 1.67 % / 0.22 % / 0.0041 % of peak`) is
> `min/max` of each grid — it assumes the peak maps to the format's **maximum**. Every
> run mapped it to grid value **1.0**, and the grids peak at 3072 / 28 / 7.5. Under
> what the rig did, TNF4 zeroed **12.5 %** against `fp6 e3m2`'s **6.25 %** and held
> **7** levels against **12** — and won 40/40 anyway. **Mechanism withdrawn; we do not
> know why TNF4 trains.** The claim itself *widened*: adding the convention as a fifth
> recipe axis, TNF4 **45/45**, `e3m2` **16/45**, `e2m3` **12/45** — and `e3m2` flips
> **0/5 → 5/5** from the convention alone while TNF4 moves 0.10 pp. **New top threat,
> published before a referee could raise it:** at the OCP MX block size 32 TNF4's RMS
> error is **3.46× worse** than `fp6 e2m3` and worst of the three at every block size;
> **no run here has ever used a block scale.** Also: the published rig had `EPOCHS = 3`
> hard-coded and could not reproduce two of its own records. **38 derived checks.**
> Theorems **T798/T798a/T798b**, lessons **1435–1438**. **Readiness 79 %.**

> **W948d — THE TALLY WAS TRANSCRIBED, AND IT WAS WRONG BY ONE.** The report said
> `fp6 e2m3` failed **29** of forty; recomputing from the eight records gives **28**.
> Nothing had changed — the figure was carried by hand into three documents while the
> record set grew underneath it, hidden by the same measurement circulating as
> *successes* on one page and *failures* on another. **Fixed at the root:**
> `verify_numbers.py` now derives the tallies from every record — **27 checks, all
> passing** — and every count is stated in one polarity with its denominator.
> **Derived final tally: TNF4 40/40, `fp6 e3m2` 16/40, `fp6 e2m3` 12/40.** Second
> defect: `stability.py` omitted `EPOCHS` from its record name, so two runs collided
> silently and a scratch recount returned 30 of 40. Theorems **T797/T797a**; lessons
> **1433/1434**. **Readiness 82 %.**

> **W948c — THE SWEEP COMPLETES, AND OUR OWN STATISTIC HAS A BLIND SPOT.** MNIST at
> thirty epochs: TNF4 **0/5** at 97.82 ± 0.15 (98.0, 97.6, 97.8, 97.9, 97.8);
> `fp6 e2m3` 4/5; `fp6 e3m2` 2/5 **by threshold** — but its best run (71.9) is
> **25.7 points below TNF4's worst** (97.6) and the distributions do not overlap.
> **The mean hides bimodality; the failure rate hides uniform degradation; only the
> per-seed list hides neither.** TNF4 converges (96.76 → 97.68 → 97.82) rather than
> drifting. **Totals over eight configurations: 0/40 against 28/40 and 24/40.**
> Landed as **tf#660**. **Readiness 82 %.** Theorem T796a; lesson 1431.

> **W948b — THIRTY EPOCHS, AND THE FIRST NUMBER FROM THE BENCH.** TNF4 climbs
> **85.50 → 87.38 → 88.77 ± 0.41** across 3 → 10 → 30 epochs with **0/5 failures
> throughout**, so the coarser step never starts costing. `fp6 e3m2` goes
> **2/5 → 4/5 → 5/5** and `fp6 e2m3` 2/5 → 2/5 → 4/5 — monotone in the step count,
> a runaway. Totals: **TNF4 0/35**, 25/35, 22/35.
>
> **And openFPGALoader never needed Docker.** Seven waves called the hardware axis
> blocked; the read-only JTAG scan runs in forty seconds and returns
> `idcode 0x3636093 · artix a7 200t · xc7a200` — confirming the t27 SSOT, confirming
> that trinity-fpga's `0x13636093` criterion would reject this board (#633), and
> correcting our own note: **one** cable at bus 001:005, not three. The author now
> has a one-page summary (**#658**). Landed as **tf#659**. **Readiness 80 % → 82 %.**
> Theorem T796; lessons 1429–1430.

> **W948 — THE RANGE IS FREE, AND THE FAILURES ARE DYNAMICAL.** Ten epochs instead
> of three: among the runs that trained, TNF4 and the narrow grids land **within
> 0.7 pp**, and on one task `fp6 e3m2` is **ahead** — so the 14.6 binades cost
> nothing at convergence. But `fp6 e3m2` goes from **0/5 failures at three epochs to
> 5/5 at ten** on the same task with the same recipe: noise averages out with steps,
> a runaway consumes them. Totals over six configurations: **TNF4 0/30**,
> `fp6 e2m3` 20/30, `fp6 e3m2` 17/30.
>
> **And the project is now refutable from outside**: `FALSIFY-ME.md` states five
> claims with the experiment, the expected outcome and the result that would kill
> each. Landed as **tf#657**. **Readiness 78 % → 80 %.** Theorem T795; lesson 1428.

> **W947 — WHAT SURVIVES IS RECIPE-INSENSITIVITY, AND ITS MECHANISM IS RANGE.**
> At six physical bits TNF4 spans **14.6 binades** against `fp6 e3m2`'s 8.8 and
> `fp6 e2m3`'s 5.9 — the φ-lattice spends its width on range, so under a max-rule
> scale nothing underflows where a narrow grid loses everything below 1.7 % of the
> peak. My percentile-init fix was **refuted** (it broke the configuration that
> worked), and W946's repair proved **task-specific**. Failure counts across three
> recipes and three tasks: **TNF4 0/20**, `fp6 e3m2` 8/20, `fp6 e2m3` 14/20; TNF4
> on KMNIST is 87.13 ± 0.15, the tightest spread measured here. **The mean was the
> wrong statistic** against a bimodal competitor (86.0, 86.8, 23.0, 87.3, 30.6).
> Landed as **tf#656**. **Readiness 76 % → 78 %.** Theorem T794; lesson 1427.

> **W946 — AT MATCHED WIDTH IT IS PARITY, AND THE LAST ADVANTAGE WAS OUR OWN
> OMISSION.** Cost at six physical bits: TNF4 **51.29** cells against `fp6 e3m2`'s
> **50.29** — **2 % dearer**, where W941 claimed 2.76× cheaper by comparing six
> bits against eight. Stability: in every failing run the layer-2 activation scale
> collapsed 0.81 → 0.29 → **0.0065** — the documented LSQ failure mode — because our
> quantiser **omitted the gradient-scaling factor** of the paper it cited.
> Restoring it took `fp6 e3m2` from **2/5 failures to 0/5**, 96.58 ± 0.56 against
> TNF4's 96.70 ± 0.38. **Parity on all three axes.** Landed as **tf#655**.
>
> **Eight corrections in seven waves, none found by an outside reviewer.** What
> stands: the mathematics, the 8-bit null, the apparatus, and parity itself — a
> novel lattice matching a mature encoding at equal width costs nothing to adopt.
> **Readiness 75 % → 76 %.** Theorem T793; lesson 1426.

> **W945 — MOST OF THE ADVANTAGE WAS WIDTH.** TNF4 is physically **six bits**
> (57 grid values) and had been compared against a **four-bit** fp4 (15 values) for
> four waves. Against real 6-bit floats: **+0.11 pp (MNIST, t 2.2)** and **+0.17
> (Fashion, t 1.2 — not significant)**, and with quantised activations on Fashion
> **fp6 e3m2 wins by 0.42**. What survives is **stability**: TNF4's σ is 0.17–0.72
> pp everywhere against **σ = 46.09** and **32.33** for the fp6 formats on MNIST.
> The chain now reads **37.9 → 0.19 → 1.58 → 0.11**, every step against our own
> interest. Landed as **tf#654**; the container pinned by digest as **tf#653**.
> **Readiness 74 % → 75 %.** Theorem T792; lesson 1425.

> **W944 — THE RANGE IS THE RESULT, AND MY OWN PREDICTION WAS REFUTED.** W943
> predicted that a stronger QAT recipe would close the residual 4-bit gap. With a
> **learned scale** it *widened* eightfold on MNIST: **+0.19 → +1.58 pp** (SE 0.19,
> t 8.2); Fashion held at +0.91. With **4-bit activations** fp4 e2m1 becomes
> **seed-dependent** — 13.93 % ± 6.30 on MNIST against 84.58 % on Fashion — while
> TNF4 loses 0.26 / 0.55 pp with σ ≤ 0.58. The honest headline is now a **range**:
> **13–65 pp** without retraining, **0.9–1.6 pp** with a trained scale, plus
> competitor instability at 4-bit activations. Landed as **tf#652**.
>
> **`tri verify` added:** every headline number recomputed from its committed
> record — **23 checks, all passing**. **Readiness 71 % → 74 %.** Theorem T791;
> lesson 1424.

> **W943 — THE ADVANTAGE IS CONDITIONAL ON NOT RETRAINING.** Training through the
> quantiser closes the 4-bit gap **44×**: TNF4 − fp4 falls from **+37.88 → +0.19 pp**
> (MNIST) and **+64.42 → +0.89** (Fashion). Still positive, still 5/5 seeds, still
> significant — but it changes category. The claim is now conditional: **13–65
> points for a fixed model that cannot be retrained, under one point where
> retraining is available.** On a CNN the collapse is smaller and unstable
> (−13.13 ± 13.66, −25.21 ± 11.31); **the 8-bit null survives convolutions too**.
> Landed as **tf#646**. Hardware branch blocked: the Docker daemon does not
> respond, so no bitstream builds here. **Readiness 68 % → 71 %.** Theorem T790;
> lesson 1422.

> **W942 — THE FRONTIER IS CLOSED, AND THE ADVANTAGE IS AT ONE RUNG.** Structural
> decoders generated from each format object and verified against the oracle over
> **every** code (64 / 2,048 / **524,288**, zero mismatches): TNF4 **55.29** cells,
> TNF8 260.57, TNF16 **450.29**. Against the cheapest working option per class:
> `binary16` 438.57 vs TNF16 450.29 (**TNF 2.7 % dearer**), `fp8` 152.57 vs TNF8
> 260.57 (**1.71× dearer**), and at four bits `fp4` loses 70 pp while **TNF4 loses
> 0.33–1.05 at 2.76× less cost than fp8**. Landed as **tf#645**; the three-widths
> question as **tf#644**.
>
> **The 16-bit comparison inverted for the third time**, each correction forced by
> the previous one's own principle and each moving against us: 386.57 → 424.86 →
> **450.29**. Method bias measured at under 8 % and not one-directional. A
> **referee page** now lists every claim, its record, its limitation and the four
> withdrawals. **Readiness 63 % → 68 %.** Theorem T789; lesson 1421.

> **W941 — THE PARETO POINT.** Every decoder regenerated from its own conformance
> oracle by exhaustive enumeration, so no implementation-quality difference can
> enter and every width comes from the format object. With **weights and
> activations both quantised**, five seeds, two tasks:
> **TNF4 costs 51.29 consumer cells at −0.33 pp (MNIST) / −1.05 (Fashion), against
> fp8 e4m3's 152.57 cells at −0.02 / −0.04 — 2.97× cheaper for fp8-class
> accuracy**, and it is the only sub-8-bit format measured that works at all
> (fp4 and GF4 lose seventy points). The 8-bit null survived the activation
> experiment designed to break it. Landed as **tf#643**.
>
> **And a width error three instruments shared:** the oracle says TNF16 is
> physically **19 bits** (16 in the name, 17 in the caption and in the module W940b
> switched to), TNF8 is **11**, not 10. It hid because the positive half of a
> sign-magnitude format enumerates perfectly well alone — 1,008 healthy-looking
> values, no negatives. Both rigs now assert otherwise. **Readiness 57 % → 63 %.**
> Theorems T787–T788; lessons 1419–1420.

> **W940 — TWO SCALING AXES, AND A FRONTIER THAT CONTRADICTS THE RUNG NAMES.** On a
> 269 k-parameter MLP the 4-bit gap is **+37.88 pp (t = 12.4)** on MNIST and
> **+64.42 pp (t = 24.7)** on Fashion, 5/5 seeds — monotone along difficulty *and*
> capacity. The bigger baseline (97.26 %) now matches FINN's own fp32 MLP, and the
> 8-bit null survives both axes at 0.02–0.04 pp.
>
> Fourteen decoders priced behind an identical multiply give a monotone curve
> spanning **112×** (3.43 cells at 2 bits → 385.14 at 16), with the decoder at 2 %
> of the unit. Joined with accuracy: **at zero accuracy loss fp8 costs 138.57 cells
> and TNF8 costs 230.57 — 1.66× more, because TNF8 stores ten bits while named for
> eight.** Where TNF wins is four bits, where fp4 is not cheaper but unusable.
> Landed as **tf#641**. **Readiness 52 % → 57 %.** Theorems T784–T785; lessons
> 1415–1416; `tri frontier` added.

> **W939 — THE 4-BIT RESULT IS NOW SIGNIFICANT, AND THE ALPHABET IS THE ARGUMENT.**
> Five seeds, MNIST and Fashion-MNIST, paired: TNF4 − fp4 e2m1 = **+8.40 pp
> (t = 3.7)** and **+27.75 pp (t = 4.5)**, 5/5 seeds each, p < 0.05 — and the effect
> is **3.3× larger on the harder task**, which is how you know it is real. The
> losing formats are unstable, not merely worse (σ 13.95 pp against 0.51).
>
> And the fusion test: the decode gap **survives exactly** (TNF16 vs BNF16 is
> 8.000 cells bare and 8.000 fused), so the LUT-absorption objection does not erase
> it — **but it is 2 % of the unit**, and the consumer's own cost is set by the
> **alphabet**: 382.4 cells behind 16 input bits, 4.1 behind two. **93× on the
> consumer against 8 cells on the decoder.** Landed as **tf#640**. **Readiness
> 47 % → 52 %.** Theorems T782–T783; lessons 1413–1414; `tri last` added.

> **W938 — THE ACCURACY COORDINATE EXISTS.** `top-1`, `ImageNet`, `CIFAR`, `MNIST`
> were 0 hits in 7,858 lines. Now: MNIST 784-32-10 MLP, fp32 **93.39 %**, weights
> round-tripped through the shipped oracles with a per-tensor scale — at **16 bits
> six formats whose error spans 16× land within 0.02 pp**, at 8 bits within
> 0.19 pp, and at **4 bits TNF4 holds 93.38 % (−0.01 pp) while fp4 e2m1 and GF4
> lose 5.49**. Above four bits the format is invisible to the task. Landed as
> **tf#638**.
>
> **And a 70-point artefact of ours, caught and recorded:** the unscaled 4-bit run
> flushes 98.8 % of weights to zero, so it measures dynamic range, not the number
> system — the tell was two distinct formats agreeing to the digit. The empirical
> prior of a trained tensor is **8.1 binades**, against the **77** the regenerators
> draw from. **Readiness 41 % → 47 %.** Theorems T779–T780; lesson 1411;
> `tri cells` added.

> **W937 — THE BASELINE WAS DOWNLOADED AND THE PRIOR WAS MEASURED.** PACoGen, the
> field's reference posit hardware, cited zero times in the manuscript, is public
> Verilog: through our own rig it puts `data_extract_v1` at **92.000** cells
> against this tree's `posit16_decode` at **125.000** — **the reimplemented
> baseline is sound**, doing strictly more work for 1.36×. At operator level and
> matched 16-cell width, TNF's adder is **561.670** against `posit_add`'s
> **693.000**: **1.23×**, where the paper claims 6.1× from decoder models.
>
> And the accuracy prior: TNF16 leads under all five priors tested, but its
> advantage over posit16 is **14.63× under the paper's uniform-77-binade draw and
> 1.02× under a standard normal**. The claim that survives is **prior-invariance**
> — TNF16's error moves 1.046× where posit16's moves 14×. Landed as **tf#636**;
> optional honest-Fmax search as **tf#637**. **Readiness 36 % → 41 %.** Theorems
> T777–T778; lessons 1409–1410; `tri recall` added.

> **W936 — THE DECODE COST IS MEASURED, AND THE FREQUENCY COLUMN HAS A NAMED
> DEFECT.** yosys runs locally, so the CI queue stopped being a blocker: each
> decoder was instantiated N times in a chain and `cells(N) = fixture + cost·N`
> fitted at N = 1,2,4,8 — eighteen of nineteen fits exact with integer slopes.
> **The ternary exponent field decodes 5× cheaper than the binary one** (2.000 vs
> 10.000 cells), TNF's cost is width-independent across 16/32/64, `int8` is
> exactly free, and the spread to the tapered formats is 62–152×. Landed upstream
> with its rig as **tf#634**.
>
> Reading nextpnr-xilinx's own source then corrected one of our theorems and
> named a defect in every frequency we have: `0.1 ns` setup/hold/clock-to-Q for
> every flip-flop, one speed grade in the chipdb, `--freq` consumed by router1
> and ignored by router2, and router2 emitting placer pre-route estimates in
> post-route-looking text (**tf#635**, T776, T771 erratum). **Readiness 33 % →
> 36 %.** Theorems T774–T776; lessons 1406–1408; `tri audit` added.

> **W935 — THE AUDIT TURNED ON US, AND THE PAPER GOT A REFEREE.** A hostile
> referee pass plus two prior-art sweeps, every load-bearing claim re-verified by
> hand: the manuscript's headline ranking (10.2 %) is **below the resolution the
> manuscript itself states** (11.4 %), `placer`/`router` occur **zero** times in
> 7 858 lines against a measured 4.66× effect, the empty control in
> `tab:cleandecode` is beaten in area and speed by two of its own rows, and
> thirteen defining references are missing. Filed as **tf#631**.
> **Publication readiness: 33 %** — the mathematics is submission-ready, the
> hardware section is not; re-centring on the 6.08× decode-cost separation is one
> decision worth ~55–60 %.
>
> **And three blocker findings were ours.** `G8-VERDICT.md`'s "LNS16 does not
> reproduce" is **WITHDRAWN** (tf#632): `MATRIX.md:35` lists LNS16 at 43.11 MHz,
> 0.16 % from the published value — we read a blank cell in our own reference
> table as absence of evidence, and applied the dispersion band with a
> denominator it was never defined with. tf#625 closed as not planned. The CAD
> configuration is now recorded per row and the report refuses to rank across
> configurations (tf#630). Theorems T764–T773; lessons 1402–1405; `tri` grew five
> local wave commands (t27#2244).

> **THE LANDING (W913–W916).** The user's standing order «сам все мержи всегда»
> converted every waiting row below into agent action: tf#603, tf#612, tf#615
> (audit → main), **t27#2217 (the whole ladder + master merge, ratchet
> 221/221 CLEAN) — MERGED 12:24:15**, and t27#2221 (nine dangling .lake
> gitlinks that broke every recursive checkout since #1304) — MERGED 12:44:54.
> G8's cost sweep runs in CI now (run 32249246232: generate green, ~100 yosys
> arms green, PnR ahead). Remaining human input: the two decision words
> (forall / dialect) and the G8 verdict when CI finishes.

Nineteen autonomous waves closed two arcs. Everything below is measured,
committed, and waiting on FIVE human actions; nothing else blocks.

## Track 1 — the TNF paper (`gHashTag/trinity-fpga`): NO-GO on one gate

Work done: 20 of 59 tables under runnable regenerators; document defects found,
fixed and confirmed (PR #601 merged by owner) — **the ledger enumerates twelve:
nine numbered paper defects plus three gate corrections; earlier pages said
"16", which no enumeration in the tree supports (W935 audit)**; 8 further
findings reported for the author's judgement; toolchain properties measured (seed dispersion
1.6–41.7 %, placer/router flips fp8-vs-TNF winners). Full ledger:
`docs/reports/upstream/TNF-FINDINGS-LEDGER.md`.

| # | action | who | cost |
|---|---|---|---|
| 1 | merge PR **tf#603** (19-commit paper audit) | owner | one review |
| 2 | merge PR **tf#612** (one-file workflow registration) — **the only thing between G8 and a green release checklist** | owner | one click |
| 3 | ~~after #612: one `tnf-cost-sweep` dispatch closes G8~~ — **superseded W920/W935**: the cost sweep measures (E_t, M) ladder arms, and `tab:untraced`'s sixteen are format-comparison tracts, so that dispatch could never have closed G8. It was closed by the tract sweep instead (19/19 routed) | done | — |

Also standing: three leaked credentials (trinity#601, trios-dwagent#1,
trios-railway#124) still need human rotation.

## Track 2 — the t27 grammar ladder (`gold-ring/0001-0002-…`, t27#2217)

Fourteen shipped rungs: 67,760 → **25,670** discarded tokens (−62.1 %),
BDD-line readability 45 % → **98.5 %**, zero undisclosed regressions, every
rung probed + adversarially panelled + corpus-certified. The residue is fully
priced, and BOTH remaining decisions are REHEARSED — built, measured on the
full corpus, reverted, patch filed:

| # | decision | page | effect of «2» |
|---|---|---|---|
| 4 | **forall bodies** (74 % of residue) | `docs/reports/gold-ring/FORALL-DECISION.md` | 25,670 → 6,592 |
| 5 | **dialect bodies** | `docs/reports/gold-ring/DIALECT-DECISION.md` | with #4: → **4,711 (−93 %)** |

Answer format: two words on t27#2217 (e.g. «forall: 2, dialect: 2»). Each
lands as a one-wave rung; W910 verified the patches REPRODUCE their quoted
numbers exactly (6,592 and 4,711) and that the measurement is
mutation-sensitive (a broken capture boundary shifts both the token count and
the parse-fail diff).

## Where everything lives

- Ladder branch: `gold-ring/0001-0002-compound-assign-nested-fn` (cumulative
  patch `LADDER-COMPLETE-0001-0014.CUMULATIVE.patch`, 1,273 lines, FROZEN_HASH
  verified at HEAD)
- Theorems T650–T753 in `docs/theory/IGLA-FORMAL-RESULTS.md`; method in
  `.claude/skills/oracle-method.md` (Parts I–III)
- Session narrative: `docs/reports/SESSION-SUMMARY-W846-W890.md` (through W906)
- Decision thread: t27#2217 (ten comments carry the whole ladder history)

---

*φ² + φ⁻² = 3 | TRINITY*
