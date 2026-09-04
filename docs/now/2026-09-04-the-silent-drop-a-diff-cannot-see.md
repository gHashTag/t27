# NOW -- The silent drop a diff cannot see (2026-09-04)

## A survey corrected the framing I shipped on, and named a hole

§519 shipped `tri census pin` on my own measurement. A survey of nine systems then said three
things the measurement could not.

- **"Always-on gates get muted" is a mechanism, not a law, and the mechanism is NOISE.** Every gate
  the survey found removed or demoted -- dask, google/wire, jaeger, Mozilla Perfherder, rustc-perf,
  the whole `codecov: informational: true` population -- measured something with a real noise floor.
  dask's config carries a comment about a red X caused purely by upload ordering. **A census here is
  a grep over a directory producing small integers.** Deterministic, so the mechanism does not apply.
  Before importing received wisdom, name its mechanism and check whether you have it.
- **Nobody gates on a bare move.** Of nine systems each does one of four things: beat a noise model
  (Chromium perf wants ≥10% *and* ≥2.5σ, step-shaped, ≥6 samples a side, plus a reference control);
  shrink the population (SonarQube New Code, Codecov `patch`); set a ceiling far above ordinary
  movement (size-limit); or **compare against a declaration the author committed** -- Metalava's
  `api/current.txt`, `cargo-semver-checks`. A deterministic count belongs in the fourth family, and
  that is what this is.
- **No snapshot tool tells intent from accident. Not partially -- not at all.** They supply exactly
  two things: the moved value forced into a committed diff, and a deliberate keystroke. Intent is
  human every time -- which is why the failure text asks for the reason in the commit message
  instead of pretending the ledger holds it.

## The hole: a ledger with no census

`insta` ships `--unreferenced=reject`. **Drop a name from the pinned list and every remaining
reading still matches, so the gate goes green having quietly stopped watching something** -- a
silent drop that a comparison of OUTPUTS is structurally blind to, because the dropped output is no
longer compared. It is now a failure in its own right, and the control is one `cp`: plant
`orphan.txt`, the gate must exit 1.

## And that control caught a second defect, mine

The orphan scan was written **after** the `PASS` early return, so on an otherwise-clean tree it
never ran and the planted orphan **passed with exit 0**. A guard placed after the return it exists
to prevent is a comment, and only running it said so. Fixed by lifting the scan above the return;
re-verified both ways -- planted orphan exit **1**, clean tree exit **0**, 649 crate tests pass.

Refs #3176
