# NOW -- tri pr ready --why: a red check's name is not its reason (2026-10-04)

## A failure called pre-existing is compared by what its failing step printed (Closes #5852)

- `tri pr ready` calls a failure pre-existing when a check of the same NAME is red on recent master commits, on merged pull requests, or on its own workflow's newest master run. Its base PR's NOW entry lists the hole under "Not established": a pull request that adds a fifth conflicted type name to `Corpus ratchet (expected-failure ledger)`, red on master for four, read exactly like one that adds nothing. The ratchet's four `##[error]` lines are the same whichever name moved; the difference is in the step's output above them.
- `--why` (off by default): for each failure called pre-existing, the jobs API names the first failing step of this pull request's job and of the baseline's job, and each job's log gives that step's own output, from its `##[endgroup]` to the first `##[error]`. Timestamps, colour codes, durations, shas (7-40 hex with a digit and a letter) and ids of 9+ digits are masked. The baseline job is the one the baseline used: the master walk's check-run, the workflow fallback's run, or the first merged pull request failing that name.
- Each side's last 60 lines are looked for anywhere in the other side's whole step output, so a line dropped earlier does not shift the window into a false difference. A line not found as itself is looked for by its shape, every run of digits read as `#`: `observed 78` and `observed 81` are one shape. Such a line is printed (`~ here: / there:`) and not judged -- a count moves when the cause does not (#5663 below). The same step, and no line here whose shape is not there: `SAME REASON` (or `SAME REASON, fewer` when shapes there are missing here). A different step, or a line here with a shape not there: `NEW REASON`, up to 8 such lines printed with `+`. A job that is not Actions, a log that cannot be read (expired, 404) or holds no error annotation: `why: cannot compare -- ...`, listed at the end as not established, verdict unchanged.
- `NEW REASON` is its own verdict, exit 7; `--merge` refuses on it. Precedence: WAIT 2 > CANNOT TELL 3 > DO NOT 1 > NEW REASON 7 > safe 0. Under another verdict the new-reason failures are still listed. The verdict test now calls the real `verdict_code` instead of a copy of the chain in the test.

## Measured, 2026-10-04

- The rule was run against the open population before it was trusted. Of 40 open t27 pull requests, 24 had a failing Corpus ratchet job; 15 printed master's own `observed 81` and four names. #5663 printed `observed 78` and one of master's four names (CounterState) -- the same reason, fewer -- and the first draft, which kept counts as words, would have called it NEW. That draft is why numbers are read as shapes.
- Real runs, all four of cron 8782e5f8's open t27 pull requests: #5849 Corpus ratchet job 111329604306 vs master 6e3322918 job 111324093808 -> `SAME REASON`, 8 lines; GitGuardian (not Actions) stays `DO NOT`, exit 1. #5839: Corpus ratchet SAME (master walk), `emit-bitexact` SAME against merged #5811 job 111314496536 (merged-PR path), `fpga-conformance` SAME against master e7ed3790b job 111186471017 (workflow-fallback path), exit 0. #5828 and #5824: Corpus ratchet SAME, exit 0. About 85 s per pull request.
- Real runs where the reasons differ:
  - #5781, job 111256330898: `NEW REASON`, `+     + ModuleInterface  NEW conflict`, `observed 78` vs `81` printed and not judged; exit 7.
  - #5663, job 111256439933: `SAME REASON, fewer`, `observed 78` vs `81` printed and not judged; exit 0.
  - #5812, job 111280188895: `NEW REASON`, the failing step differs (`Run the corpus ratchet` here: three specs as unexpected failures); exit 7.
  - #5452: `DO NOT MERGE`, exit 1, for two failures only here, and three more listed as red elsewhere for another reason -- `emit-bitexact` with 15 specs newly failing to generate against one, `spec-guards` and the ratchet failing in other steps.
- Tests: 6 new in `prcheck::why_tests` on a fixture shaped like job 111329604306's log, plus the rewritten verdict test (59 in `prcheck`, all pass). 16 mutations, each red: no timestamp strip (5 tests), no colour strip, no duration mask, no sha mask, sha mask without the digit rule, no id mask, the script echo kept, annotations kept, tail compared to tail, the step ignored, fewer lines read as new (3), 7 above 1, 7 never set, no dedup, numbers judged as words (2), a renumbered line counted as new (2).
- Census: `fetches` moved 72 -> 74 lines, 31 -> 33 fetch sites (`--paginate` 9 -> 11): `failing_jobs_on` and `failing_job_in_run`, both complete by construction. Blessed in the same commit.

## Not established

- That the same text is the same cause.
- A step whose whole reason is a count: two runs that differ only in a number read SAME, and the numbers are printed for a person to read. #5452's `FAIL: 15 spec(s)` against `FAIL: 1` was NEW only because the 15 names were printed too.
- Output after the first `##[error]` (an annotation printed mid-step cuts the tail there).
- A step whose reason is only in an artifact or in a summary.
- Logs past their retention, which read as cannot compare.
