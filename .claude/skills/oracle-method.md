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
pipeline), `t27c edit-check`, `t27c recompute-diff --label` (whole-file mode is a
false-pass generator).
