# NOW — the second blocking check was also right, and also unread (2026-08-23)

`Corpus ratchet`, the other permanently-red required check, had been printing a precise verdict for its whole history and nobody had acted on it:

```
UNEXPECTED PASSES  : 3
UNEXPECTED FAILURES: 2
```

- **The same two specs appear in both lists.** They used to fail `parse` outright; they now parse and fail the narrower `parse-no-discard`. The third simply passes.
- Three removals and two re-labels. The distinction matters: **the excuse for those two specs went from "does not parse at all" to "parses but discards tokens", and the third lost its excuse entirely.** Ledger 221 → 220 against a cap of 221 — strictly tighter. `RATCHET: CLEAN`.

**Both permanently-red required checks turned out to be right.** Neither was a broken instrument; both reported real, small, actionable findings that had gone unread long enough to become scenery. `coverage` asked for 58 lines to be dropped; `Corpus ratchet` asked for 3 removals and 2 re-labels. **Five minutes of work each, blocking every merge in the repository.**

That is the failure mode neither "a gate that cannot fail" nor "a gate that cannot pass" covers. **A gate that is right, and whose verdict is a short actionable list, becomes furniture if nobody reads it — and the longer it stays red the more certain everyone is that it means nothing.**

The check that separates the two cases takes one command: run the gate locally and read what it says. Both of these named their own remedy in the output, on every run, for days.
