# NOW — the marker fired on real data (2026-08-23)

- **The interrupt marker caught a real one.** A run from the previous iteration had been orphaned and left a gate mutated. The marker named the gate, printed the recovery commands, and I ran exactly what it printed — `git checkout -- tools/` on the directory, the instruction I should have followed the first time.
- **And it spoke second, which is the wrong order.** The dirty-tree guard fires first, and after an interrupt the tree *is* dirty — so the informative message existed and was never shown. Marker now checked first. Found by hitting a real interrupt and reading the wrong error.
- **An orphaned run was still alive**, thirteen minutes in, from an iteration I had already reported as finished. Third time mutants got loose, and every time the cause was mine: starting a long background job and losing track of it. The guard did refuse the concurrent run I started on top of it — an unplanned safety property, since two mutation loops would each restore the other's mutations.

## Seventeen of twenty-one rows, and one is stark

```
verify_emit_bitexact.py   0/1   0/1   0/11   0/4   0/0
```

**Its control kills nothing** — not one of seventeen mutants, across four operators. I gave that gate a control three iterations ago: three planted cases, all passing, covering the skip pair and the codegen-failure branch. **None of the sites the operators can reach are among them.**

Same class as the gate fixed in §49 — a control that exists, passes, and is not connected to the verdicts anyone would break. Worse here, because I wrote this one *knowing* that class, and the number exposing it could not be produced until the run learned to be interruptible.

**The table is 17 of 21 and says so.** A partial table reported as partial is a measurement; the same table reported as complete is the thing this campaign is about.
