# NOW — the floor guard pins the comparison but not the number

Last updated: 2026-08-22

## A third assertion bounds the floor's value (Closes #2374)

- Branch: `fix/2374-floor-value-bound`
- Issue: #2374

### Что легло

`cli/tri/src/gates.rs` — one assertion, `floor >= 10`, read from clap alongside the two
that #2371 added. Those two pin the comparison's strictness (mutating `total < min_runs`
to `total <= min_runs` fails the second) but barely constrain the number: assertion 1
fails only for a floor of 0, 1 or 2. Setting `default_value_t = 3` passes both while a
three-run workflow becomes reportable as a dead gate — the judgement `--min-runs` exists
to prevent.

A bound rather than an exact pin, so a deliberate retune stays possible and dropping to a
handful does not. The doc comment above the test was corrected in the same change: it said
"both assertions", which is no longer the count and was never the whole claim.

### Границы честности (BINDING)

- **Not run locally.** This machine has been at ~215 MB free and hit literal zero eleven
  times today; `cargo test -p tri` was not attempted. CI is the verification, via
  `cli-tri.yml`'s `build` job, which does run it. No local pass is claimed.
- The assertion's biting is legible by construction — it compares the value read from clap
  against a bound — but it was **not** demonstrated by a planted mutant, which is the bar
  the rest of today's guards met.
- This does not make `tri gates dead` correct, and it changes no runtime behaviour. It
  constrains one default.
- A prior comment on #2369 called the second assertion vacuous. That was wrong and is
  corrected there: it pins the comparison operator. Only its insensitivity to the *value*
  was real, and this change addresses that.
