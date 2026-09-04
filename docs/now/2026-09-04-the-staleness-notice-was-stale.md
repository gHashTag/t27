# NOW -- The staleness notice was itself stale (2026-09-04)

## A republish notice named a version older than the one I had already merged onto

- The notice said the shared dashboard was "now version `1788523848-b8d8`" and that my copy was
  stale. The stamps are unix seconds: the notice names **12:10:48Z**, the base I had merged onto
  was **12:21:40Z**, and the live page was **13:31:53Z**. **The notice was 81 minutes behind.**
- Live was my own publish, established by content and not by the version string:
  **750,681 bytes and 622 distinct `<h3>` on both sides, zero difference in either direction.**
- **Following the instruction literally would have destroyed work in both directions.** Merging
  onto `b8d8` drops my 176 entries *and* the other session's newest ones, because that version
  predates both.

## Twice is a class

This page already carried *"a republish notice named a version 83 minutes older than the live
one -- the stamp decodes"*, written by the other session. This is the second instance and the
first with the delta measured on both sides of the comparison.

**A staleness notice that can itself be stale is a broken ruler for staleness** -- the signal
lives inside the failure domain it reports on. The settling check is two commands and does not
trust the version string: fetch what is live, then compare h3 sets and byte length against your
own file. Re-merge only when the content actually differs.

## What the merge cost, for the record

The publish that produced the live version was the first to land in **22 fetches** across many
passes. It required reading all **3,523 lines** of the fetched page -- roughly ten `Read` calls
against a 25,000-token cap -- then merging onto **their** base rather than mine, because
inserting their blocks into my file captured only **43 of 68** unique entries: the rest live in
containers the matcher did not know about. Basing on theirs makes loss impossible by
construction, and the h3 set arithmetic proves it: 622 = 446 theirs + 176 mine.

Refs #3172
