# NOW -- I nearly republished a claim this repository withdrew (2026-09-04)

## The measurement was right and the finding was already settled

- Pass 64 showed the shape lives in the wiring, so I measured it: **12 of 50** live workflows have
  never had a `push`-to-master run. Five are named as gates -- `catalog-count-gate` (CI-01),
  `emit-bitexact-gate` (CI-02), `check-now-freshness`, `seal-staleness-warn`, `lean-proofs` -- and
  all five have `pull_request` and **no `push:` at all**.
- Then I read `docs/now/2026-08-30-a-window-read-as-a-lifetime-withdrawn.md`, whose title matched
  what I was about to write. It says: *"emit-bitexact has NEVER run on master; **it has run
  twice**"* -- withdrawn on 2026-08-30.
- Both readings are right about different questions, and the API says so exactly:

      branch=master                 total_count = 2     <- manual dispatches
      event=push                    total_count = 0
      event=push & branch=master    total_count = 0

  It has run on master, twice, and never automatically. The withdrawn note had already reached the
  better framing -- **reachability, not staleness** -- and `has_auto_default_run` was written for
  it with 11 tests including two counterexamples a review would not produce.
- `tri gates unmeasured` reports **1 of 50** today and states the rule in its own output:
  *"Reading a window and reporting a lifetime is how this section came to exist."* It also settles
  the framing I was about to reopen: PR-only by construction stops being read as a gap.

**No new finding. Second consecutive pass where the hunt came back empty** -- and both times the
thing that stopped a wrong publication was reading what this repository had already written down.

## What did ship

`tools/check_coq_in_build.py` -- the 18 Coq files no `_CoqProject` names, pinned two-sided (#3153).
Three mutants; one survived, and the corpus proved it right: `module`/`` `timescale`` change no
file's classification here. Kept as a defence with a fixture that makes it testable rather than
decorative. `coq-kernel.yml`'s `paths:` widened to `proofs/**` in the same commit -- a gate whose
trigger is narrower than its subject is the shape this pass spent its time measuring.

Refs #3153
