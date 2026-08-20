Seven waves of autonomous measurement (W942–W948) produced fourteen merged PRs against this repository. They are easier to act on as one page than as fourteen threads, so here is what was measured, what it changed, and what it costs you to check.

**Everything below is committed**: rigs in `research/arxiv_tnf/*.py`, records in `research/arxiv_tnf/measurements/*.json`, write-ups in `docs/`. `verify_numbers.py` recomputes every headline figure from those records — 23 checks, all passing.

---

## 1. The bottom line changed four times, always downward

The φ-lattice's advantage over a float at four to six bits, as measured across successive waves:

| what was compared | result | why it moved |
|---|---|---|
| TNF4 vs fp4 e2m1, post-training quantisation | +37.9 / +64.4 pp | PTQ only, no retraining |
| …trained through the quantiser | +0.19 / +0.89 pp | QAT closes it 44× |
| …with a learned scale | +1.58 / +0.91 pp | a better recipe re-opened it |
| **…against a same-width float (`fp6 e3m2`)** | **+0.11 / +0.17 pp, n.s. on one task** | **fp4 is 4 bits; TNF4 is physically 6** |

And on cost: what was reported as "2.76× cheaper than fp8" is, at matched width, **51.29 cells against 50.29 — 2 % dearer**.

**At six physical bits the φ-lattice is at parity with a same-width float on cost and on accuracy.**

## 2. What does survive, with a mechanism

TNF4 trained successfully in **30 of 30 runs** across four quantiser recipes, three tasks and two training lengths. `fp6 e3m2` managed 13/30, `fp6 e2m3` 10/30.

The mechanism is measured, not asserted: at six bits TNF4 spans **14.6 binades** with 28 positive values; `fp6 e3m2` spans 8.8, `fp6 e2m3` 5.9. Under a max-rule scale the narrow grids zero everything below 1.7 % (e2m3) or 0.22 % (e3m2) of the tensor peak. In every failing run the layer-2 activation scale collapses monotonically — 0.81 → 0.29 → 0.0065 — and once it is near zero the gradient that would raise it has gone with the activations.

**The coarser step costs nothing where both converge:** at ten epochs the successful runs land within 0.7 pp of each other, and on one task the narrow grid is ahead.

**And longer training makes the narrow grids worse:** `fp6 e3m2` goes from 0/5 failures at three epochs to 5/5 at ten, same task, same recipe. A failure rate that grows monotonically with steps is a runaway, not noise.

## 3. What this does to the manuscript

- The headline **MHz/LUT ranking is unresolved by the paper's own stated instrument**: the margin is 10.2 % against a stated resolution of 11.4 % (issue #631).
- **`placer` and `router` occur zero times** in 7,858 lines, against a measured 4.66× effect.
- The rungs' **physical widths disagree across three places** in this repo — TNF16 is 16 by name, 17 by caption, **19 by the oracle** (issue #644). Everything downstream is priced by physical width, so this is load-bearing.
- **Thirteen defining references are absent**, including the papers that define this comparison's own fp8 and posit rows.
- At **8 and 16 bits no format difference reaches any task** we ran — five formats within 0.06 pp with activations quantised, across MLP and CNN, two sizes, five seeds.

## 4. What we withdrew about our own work

Eight corrections in seven waves, **none prompted by an outside reviewer**: four comparisons at unmatched width; a variance reported as an instability when all five runs had in fact failed; an enumeration that missed a sign bit and so contained no negative numbers; a frontier priced by module name; and a quantiser missing the gradient-scaling term of the paper it cited — which was the last thing keeping a stability advantage alive.

`docs/REFEREE-PAGE.md` lists every current claim with its record and its limitation. `docs/FALSIFY-ME.md` states five claims with the experiment, the expected outcome, and **the result that would kill each**.

## 5. What we cannot do from here

- **No hardware.** The Docker daemon on this bench does not respond and cannot be started non-interactively, so no bitstream can be built. Three XC7A200T boards are attached and unmeasured. **One human action — starting Docker — unblocks the entire hardware axis**, which is the only axis where this project has no number at all.
- **No external replication.** All thirty runs are ours, on one machine.
- **Every editorial decision** about what the manuscript claims.

---

*Filed by the autonomous wave loop. Related: #631, #644, and PRs #634, #636, #638, #640, #641, #642, #643, #645, #646, #652, #653, #654, #655, #656, #657.*
