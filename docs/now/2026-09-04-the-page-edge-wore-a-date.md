# NOW -- The page edge wore a date (2026-09-04)

## The only red workflow on master, and the instrument that reported it was wrong about when

- `OpenSSF Scorecard` is the sole workflow whose latest master run failed. Measured over every
  page of its history: **675 runs, 428 successes, 105 consecutive failures**, streak beginning
  **2026-09-03T07:19:43Z**, last success **2026-08-31T13:50:24Z**.
- The cause is **upstream and not ours**: every run dies at step 2 pulling
  `gcr.io/openssf/scorecard-action:v2.4.0` with *"denied: This API method requires billing to be
  enabled ... project #367732848534"* -- OpenSSF's own GCP project. It blocks nothing: the
  workflow is named merge-critical nowhere and emits no required context, which is why 105 red
  runs cost zero merges and produced no signal. `publish_results: true` makes repairing it an
  outward-facing act, so it is **#3176** and not a patch.

## `tri red now` said `30+ in a row since 2026-09-04T06:01`

- It is off by **75 runs and 23 hours**. The printed instant is exactly the **30th newest run**:
  the edge of the page, predicted before it was checked and confirmed to the second.
- `streak()` reads one page and sets two values in one loop -- `n += 1; since = at`. The count was
  marked as a lower bound; the instant, assigned on the next line from the same bounded read, was
  printed as a fact. **The file's own comment names the trap and covers one of the two values it
  applies to.**
- **The bounds point in opposite directions**, which is why one marker cannot serve both: reading
  newest-first the count is a floor and the instant is a ceiling. `06:01+` would say *after*. And
  the direction is the damaging one for this command's purpose -- a date drifting newer makes an
  old outage read as fresh, while its own closing line calls a streak "the number of times nobody
  looked".

## The fix is a boundary that does not depend on page size

The start of an outage is the **last PASS**, one request away (`?status=success&per_page=1`), so:

    30+ in a row  after 2026-08-31T13:50, by 2026-09-04T06:01   OpenSSF Scorecard

and the true start lies inside the bracket. Three renderings for three states -- exact, bracketed,
or a bare ceiling with *no pass on record*. `--deep` walks every page for the exact pair and
reproduces **105 in a row since 2026-09-03T07:19**, agreeing with the independent measurement.

**Prior art converges and names it.** Prometheus latches `ActiveAt` at the transition and never
re-derives it; `ALERTS_FOR_STATE` carries the start *as the sample value*, so one sample anywhere
yields it exactly -- the naive `min_over_time(...[1h])` is censored at the window edge, this defect
exactly. Elasticsearch marks a truncated count `relation: "gte"` and marks **no** timestamp, so the
asymmetry survives even there. GitHub scopes `incomplete_results` to the whole response. The
failure mode has no name in monitoring -- a real negative result -- but survival analysis calls it
**left-censoring**.

## Two of my own, in the change about unmarked bounds

- **My insertion landed between a `#[test]` and its function.** `the_query_and_the_marker_read_one_constant`
  stopped being a test and one of mine ran twice; the suite still read a plausible 4. The compiler
  said so -- *never used* -- and it is written in my own notes.
- **My first two tests were controls that cannot fail.** They built the expected string inline
  instead of calling the code, so deleting the entire fix left them green: **2 mutants survived**.
  Extracting `render_since` and `is_bounded_read` so the tests reach them takes it to **6 of 6
  killed**, and the sixth needed a case where a short deep read and the shallow rule disagree --
  `is_bounded_read(false, true, 675)` cannot tell them apart because 675 clears the page anyway.

Refs #3176
