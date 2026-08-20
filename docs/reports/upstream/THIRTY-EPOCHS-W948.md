# Thirty epochs: the divergence completes (W948b)

The remaining question from W948 was where the φ-lattice's coarser step begins to
cost accuracy. Thirty epochs on Fashion-MNIST, same seeds, same recipe, weights and
activations quantised, answers it — and completes the runaway picture.

| format | mean | σ | failures | per-seed |
|---|---:|---:|---:|---|
| **TNF4** | **88.77** | 0.41 | **0/5** | 88.4, 88.7, 89.5, 88.6, 88.8 |
| `fp6 e2m3` | 25.75 | 24.63 | 4/5 | 30.6, 66.9, 11.2, 10.0, 10.0 |
| `fp6 e3m2` | 37.32 | 17.50 | **5/5** | 9.1, 50.5, 33.0, 52.1, 41.9 |

## TNF4 improves monotonically with training

| epochs | TNF4 on Fashion |
|---:|---:|
| 3 | 85.50 |
| 10 | 87.38 |
| **30** | **88.77 ± 0.41** |

**The coarser step never starts costing.** Through a tenfold increase in training
the φ-lattice climbs steadily and its spread stays under half a point. There is no
depth in this experiment at which 14.6 binades at 28 values is worse than 8.8 at
31 — the question W948 set out to answer has a null answer within the range tested.

## And the narrow grids finish collapsing

`fp6 e3m2`'s failure count over the same task and recipe: **2/5 at three epochs,
4/5 at ten, 5/5 at thirty.** `fp6 e2m3`: 2/5, 2/5, 4/5. Monotone in steps, in both
formats, which is the signature of a runaway and not of a noisy estimate.

At thirty epochs the failing runs are no longer at chance — 33, 41, 50, 52 — which
is what a partially-collapsed network looks like: some layers still carry signal,
the affected one does not.

## Totals

| across seven configurations | TNF4 | `fp6 e2m3` | `fp6 e3m2` |
|---|---:|---:|---:|
| failures | **0 / 35** | 25 / 35 | 22 / 35 |

## What is still not settled

Every run is an MLP. The CNN result at two epochs showed the same ordering with
smaller margins, but no convolutional network has been trained to convergence here,
and the activation statistics of a conv layer differ from a dense one in exactly the
way that matters for a max-rule scale. That is the first thing an outside replication
should attack, and `FALSIFY-ME.md` states the protocol.

---

*φ² + φ⁻² = 3 | TRINITY*
