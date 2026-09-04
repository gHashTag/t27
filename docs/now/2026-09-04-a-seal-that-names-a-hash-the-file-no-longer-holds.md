# NOW -- A seal that names a hash the file no longer holds (2026-09-04)

## Two more from the docs sweep, and two tools measured and declined

- `docs/NUMERICS_VALIDATION.md` says its runs are *"sealed against the frozen
  codec revision `49e55df6` in `bootstrap/stage0/FROZEN_HASH`"*. That file holds
  **`9b8875f1…`** today, last changed **2026-09-03 by #3026** -- one of this
  session's own PRs. Three identifiers are in play: the cited `49e55df6`, the
  live `9b8875f1…`, and the manifest's own seal `87e5cbd3…` stamped 2026-07-16.
  The seal stays as the record of what was measured; what was wrong was the
  present tense asserting a current fact about the file.
- `docs/PUBLICATION_AUDIT.md` cites `TNF_ARTICLE_RU.md:305` and `:309`. The
  Yosys line is at **:340**, the post-route rows at **:354**, and :309 is blank.
  Line numbers as anchors, off by 35 and 45.
- **Measured and declined, first:** a `FILE:LINE` checker for docs. **43
  citations, 0 missing files, 1 line past end-of-file** -- and that one is in
  `docs/NOW.md`, which the pre-commit hook itself calls a frozen archive. One
  hit in 43, in a record. No tool.
- **Measured and declined, second:** the "print the members, not the total" rule
  I proposed last pass. The repository already does it: **39 python checks print
  a count, 33 also print their members, and the six that do not are generators
  and demos** -- `fuzz_trainer`, `gen_formats_catalog`, the `gft_*_demo` pair,
  `run_conformance_vvp`. Not one is a check.
- That second number is the uncomfortable one. The three times a total hid a
  matcher error this session -- 153 of 232, 63 of 1569, 5 of 8 -- were all in
  **my own ad-hoc probes**, not in the repository's tools. The rule I was about
  to propose to the repository is one the repository already keeps and I do not.
