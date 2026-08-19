# The oracle method — reconstructing a published table from its record

Distilled from the TNF audit (W846–W876): twenty tables put under regenerators,
sixteen document defects found and fixed, five of my own claims withdrawn along
the way. Each rule below was paid for by a concrete failure named beside it.

## When to use

A document prints numeric tables; machine-written records exist (or should).
Before trusting a number, before asserting a defect, and before shipping a new
record of your own.

## The order of operations — this exact order

1. **`t27c known --dir <docdir> --about <X>`** before measuring anything about X.
   Gates, baselines and captions answer most questions in a minute. Paid for by
   W870: 55 place-and-route runs rediscovered a defect whose figure sat in one
   baseline line. Signal weights: baseline = strong, gate = medium, caption =
   weak (a caption *naming* a table is not prior art about your claim).

2. **Read the record's own metadata** — `description`, `seed`, `n`, `method`,
   generator name. It outranks every similarity heuristic. Paid for by
   `breakeven`: every score pointed at the wrong table; the JSON's own
   description field named the right one.

3. **Never map by similarity.** Substring stems voted wrong on a third of one
   corpus; numeric overlap over-reported 86 % where reading gave 8 %; a perfect
   16/16 mapping once sat at F=0.155 because the record stores 16 digits and the
   table prints 3. Similarity measures formatting overlap, in both error
   directions. Reconstruction is the only verdict.

## Writing the regenerator

- **The table is a view.** Either every printed cell comes back at *printed*
  precision or it does not. `0.282` asserts three decimals; a 1 % band passes
  values the table does not claim.
- **Match rows by key, never position.** A selection makes position meaningless
  (33 phantom mismatches from zipping σ=1.5 against σ=1.0).
- **Identify columns; never assume them.** Assert an identity —
  `ratio == a/b` to 1e-9 — before judging any cell (38 phantom mismatches from
  assuming column 3).
- **Guard the vacuous pass.** Pin the expected printed-row count; an emptied
  table must fail, not pass 0/0. Found only by mutation testing.
- **Caption claims are data.** Recompute counts, sample sizes, superlatives —
  *conditionally on the phrase being present in the source*, so a later fix
  turns the light green. A check that cannot pass is not a check.
- **Report the selection.** If the record holds more rows than the table prints,
  list the unprinted rows with one salient value each. Two tables hid their most
  damaging rows this way (a 2.48e+35 blowup one step past the printed sweep; a
  competitor's 4×-better cell).
- **Formatting is data.** Bold/dagger marks encode a criterion; recompute it
  (12/4/14 at the record's own tolerance caught a parser that dropped rows).
- **Do not edit the document from the oracle.** Report, exit 1, fix separately.

## Verifying the regenerator — adversarially, always

Run it, read it for the cheats above, then **mutation-test on scratch copies**:
perturb one record value → must fail; delete a printed row → must fail; empty
the table → must fail; restore a fixed defect phrase → must fail. Two real holes
in five scripts survived ordinary review and fell to mutations in minutes.

## Reporting

- Verdicts agreeing across instruments that share a blind spot are one verdict.
  Diversity of instrument beats count of readers.
- A reconstruction proves the table matches its record, **not that the record is
  right** — the reach column matched its record perfectly and both were wrong;
  only an independent definition (the paper's own proposition + the oracle)
  broke the tie. Two signals derived from one quantity are one signal.
- Report what *survives* beside what falls (a neighbouring claim recomputed true
  while its sibling fell — that distinction is what separates an audit from a
  hit job), and score your own record: which claims are about the document,
  which about your method; the withdrawal count is the number to watch.
- A readiness percentage that averages progress cannot see a binary gate.
  Report the conjunction of the gates.

## Shipping your own records

Ship the generator in the same commit; state which fields reproduce on a re-run
and which pin the shipped file (nextpnr logs hash differently every run — a
non-reproducible hash is a trap for the next reader). A record no script can
rebuild cannot be corrected at source.

## Tools

`t27c known` (prior art), `t27c provenance` (three weighted signals, UNDECIDED
on divergence, per-column mode for multi-record tables), `t27c battery` (run
every oracle and gate with per-script exit codes — never read `rc=$?` after a
pipeline), `t27c editcheck`, `t27c recompute-diff --label` (whole-file mode is a
false-pass generator).


---

# Part II — certificates and stores (learned bringing the method home, W878–W890)

## A certificate answers four questions

Who minted it (`sealed_by`), from which text (`spec_hash`), what came out of each
backend (`gen_hash_*`), and **how much of the file the parse silently skipped**
(`discarded_top_level_tokens`). A green light answering fewer is a mood, not a
verdict. Paid for by: 49 of my own reseals certifying truncated readings — the
parser reached EOF having thrown 43,875 tokens away, 55 % of them tests.

## Truncation is vacuity's subtler sibling

Vacuity certifies nothing; truncation certifies less than it looks like. Both
belong INSIDE the certificate, not in a report someone must remember to run.
Reaching EOF is not reading everything — use the accounted parse, and read what
was dropped line by line (`parse-complete --show`): the content class matters
(here it was the L4-mandated tests).

## Ratchets, not always-red gates

A gate that is permanently red is ignored; a defect class with a standing backlog
goes behind `--strict` or a baseline, and the ratchet bites only on GROWTH:
parse regressions (`classify --baseline`), seal staleness, truncation growth.
Refresh on recovery is a prompt, never an error.

## Stores are stratigraphies

One store minted by two compilers reports their grammar gap as data rot unless
the minter is recorded. Verify strata separately (bootstrap layer 165/165; meta
layer 11/100 — one blended number would have said "mostly broken" and meant
nothing). Two independent instruments agreeing on one boundary is the check:
reseal-guard refusals ≡ parse-baseline failures, 234 = 234.

## Freezes, prototypes, and scope

Validate a frozen-file patch in a detached copy (update the copy's FROZEN_HASH;
a failed build's leftover binary is a false-positive factory). One grammar change
per ring proposal. A soundness check's scope is the scope the transform crosses —
the capture check that asked "is it module-level?" rejected the SSOT for an
imported constant. And measure a feature's CONSUMPTION before designing it:
33 generic types, zero concrete instantiations — the question dissolved.

## Insure content before debating containers

1,766 scenarios extracted into a transfer checklist BEFORE the
grammar-vs-migration debate. The reverse order loses content exactly when the
debate drags.

## Instrument artefacts — the standing tax

Thirteen and counting: substring containment (thrice), sha-prefix,
case-insensitive glob, API pagination, parallel starvation read as regression,
wrong-scope checks, synthetic-repro generalisation, RHS-first-token
misattribution -- and the sharpest: an instrument whose aggregate counter and
itemised recorder walk DIFFERENT code paths (the discard counter incremented in
three channels, the span recorder in one, so "show" answered "nothing" for a
file charged 2,438 tokens). Two cures: **read one raw case before believing any
count**, and **reconcile sum(items) == total before trusting either account.** A timeout under load is a fact about the load; UTC stamps
sit on yesterday's local date; `rc=$?` after a pipeline is the tail's status.


---

# Part III — grammar recovery at scale (the ladder, W891–W907)

## The rung discipline

One convicted cause -> one probe file -> one measured rung. A rung ships with:
exact-line probes (synthetic repros lie — artefact #8), a full-corpus
certificate (discard total, parse-fail DIFF not count, consume-all), and an
adversarial break panel. Fourteen rungs recovered 62.1 % of 67,760 discarded
tokens and took BDD-line readability from ~45 % to 98.5 % with zero
undisclosed regressions.

## The corpus tests what exists; the panel tests what can be written

Four consecutive rungs passed 624 specs and fell to ~70 adversarial probe
attempts each. Every worst-class find — assertions vanishing under "nothing
discarded", scope theft with corrupted dataflow — came from a panel, none
from the corpus. Three lenses, ~8 probes each, AFTER the corpus sweep, every
grammar rung, no exceptions.

## Conviction maps compile

A fan-out that only reads produces suspects; give every reader an INTERVENTION
DUTY (copy to scratch, delete the suspect, re-measure) and the map's rows are
convictions that convert to fixes with no re-diagnosis. Presence is not
causality: the top offender of one map was fully acquitted by exact-line
probes; ddmin and single-variable variants assign cause, classification only
ranks suspects.

## The keyword-collision family

Five members here: `and` (operator vs clause), `let`/`const` (one token),
`invariant` (opener vs field label), `var` (declaration vs name), `then`
(clause vs element type). One template: find where the token STANDS (position,
follow-set, line, column) and give the parser that rule. Ambiguity resolves by
layout; ties need an explicit safe answer (scope theft at a column tie was
invisible to every instrument because both readings parse).

## Instrument law

Every recovery path feeds BOTH accounts (counter and spans) — the fourth
unaccounted channel manufactured zombie parses (a green file whose AST held
one declaration). `sum(items) == total` is an oracle; run it after every
instrument change. `Ok` is not done: a sub-parse that lands mid-line is a
truncation wearing a success. And a ladder ENDS: price the frontier
(intervention map), ship the tail, and hand the residue to its decision-makers
— 74 % of this one's residue is a single Architect word.
