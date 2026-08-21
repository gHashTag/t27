# NOW -- tri ci baseline ranks by failure history, not by trigger table (2026-08-22)

## tri ci baseline ranks by failure history, not by trigger table (Closes #2359)

- Correcting #2309. Its premise — a gate with no baseline can turn a PR red — is false for one of the four it reported. seal-staleness-warn is advisory by construction (every path ends in exit 0): 169 runs, 0 failures. Reporting it beside emit-bitexact (144 runs, 84 failures, no baseline, the one that actually blocked two merges) presented four findings that are not the same size.
- The signal was never the trigger table but whether the workflow has ever concluded failure. The command now measures that per hole, sorts loudest-first, and --strict alarms only on gates observed to fail. Fourth category added to the taxonomy: advisory by construction.

## Correction appended 2026-08-21 (#2361)

The two bullets above are left exactly as they were written; this section says
which part of them did not hold. Nothing above has been edited.

- **`emit-bitexact` was not one of "the four", and had a baseline when the claim was made.** Bullet 1 cites it as "144 runs, 84 failures, **no baseline**". The run and failure counts are real (147/88 at 2026-08-21T19:57:20Z). The "no baseline" half was not: `gh api repos/gHashTag/t27/actions/workflows/329310174/runs?branch=master` returns `total_count: 1` — a `workflow_dispatch` on `master` at **2026-08-20T01:05:13Z**, 41 hours before #2360 opened at 2026-08-21T18:05:14Z. #2309 says so in its own body ("…which is also how `emit-bitexact` acquired the baseline it had never had"), and #2360's own pasted output listed **two** holes and did not include it, contradicting its table three lines above.
- **#2309's four were `catalog-count-gate`, `check-now-freshness`, `loop-tools-gate`, `seal-staleness-warn`**, measuring 37/0, 1446/2, 12/0 and 169/0 at 2026-08-21T19:57:20Z. Exactly one of them has ever concluded `failure`, and it did so twice. The ranking argument in bullet 2 still holds — it just rests on 2 failures versus 0, not on 84 versus 0.
- **`--strict` was weakened, not just re-aimed.** "alarms only on gates observed to fail" exempted two states that are not observations of harmlessness: a gate that has never run anywhere (`0 runs, 0 failures`), and a gate whose history could not be read at all (`gh` bails on a non-zero exit, the caller defaulted to `""`, and `""` parsed to `-1`, which lost every `> 0` test it met). One transient 502 downgraded every hole in the sweep and printed "has never run anywhere at all" about a workflow nobody had measured.
- **The guard that shipped with it did not reach the command.** It copied the production sort and the `loud` predicate into its own body. Lifting the whole test into a standalone file containing no production code left it compiling and passing.
- Severity of all of the above was **latent, not live**: `cibase.rs` is the only reference to `ci baseline` in the tree, so no job changed colour.
