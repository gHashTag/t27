# NOW — a finding recorded as a line number expires on the next edit (2026-08-23)

Three boundary survivors were carried forward from the last triage as "possibly real thresholds", identified by file and line. Reading those lines today gave three completely unrelated statements — a `subprocess.run` argument, a `None` guard, and a random-vector append.

- **The files had been edited in between.** Five equivalence markers went in, two controls grew whole-program cases, and every line below each insertion moved. The triage note was measurably false about its own repository within a day of being written.
- This campaign already knows the fix in one place: `check_gate_preconditions.py` names its uncovered branch **by message**, with a comment explaining that an earlier version said `:346` and `:390` and was wrong before it was ever pushed. The same discipline did not reach my own notes.

**The rule:** a survivor list is a snapshot of one run against one tree. Carry forward the *file and the expression*, or re-run and re-read — never the line number alone.

**And I caught myself with a truncated view first.** Reading the fresh table through an `awk` that printed six columns, I concluded the cache was not working — the marker lives in the seventh. The conclusion came from a ruler I had cut short myself.

The full re-measurement is still running; its numbers belong to the next entry rather than this one.
