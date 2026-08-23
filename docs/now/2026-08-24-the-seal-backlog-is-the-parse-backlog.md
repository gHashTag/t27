# NOW — the seal backlog is the parse backlog (2026-08-24)

I proposed three things this iteration. Measurement killed two of them, and the second refutation joined three separate investigations into one.

- **Proposal: split the 131 non-parsing specs by error message, because if one class dominates it is one fix.** `t27c backlog` had already measured this and answered no. Depth distribution of the 148-spec defect backlog: **0 at depth 1**, 67 at depth 2, 48 at 3, 13 at 4, 20 at 5+. Its own words: *"DEPTH-1 SPECS -- the only ones a single fix can move: none. No single compiler fix can raise the compiling count."* The command's help text even records why frequency is the wrong measure — removing the most frequent cause (435 sites, 140 specs) moved the compiling count 151 → 151.

- **Proposal: re-seal the 18 stale twins from #2477 — pure improvement, no deletions, 121 → 103.** Run: **14 of the 18 are rejected by all four backends**, and the remaining 4 changed nothing but their `sealed_at` timestamp. Net movement: **zero**.

- **Why, and this is the part worth keeping.** All 14 rejected specs **do not parse at all** — `specs/tri/collections/*` (bitset, lru, map, queue, ring_buffer, set, skip_list, stack, variant), `specs/tri/graph/graph.t27`, `specs/tri/pipeline/builder.t27`, `specs/github/tests/e2e_full_flow.t27`, `specs/portable/relay_observer.t27`, `compiler/codegen/verilog/codegen.t27`.

  Sealing requires generating; generating requires parsing. **The seal backlog is downstream of the parse backlog, not beside it.** `coverage` cannot be moved by bookkeeping, and the 18 twins are stale for the same reason the corpus is 131 specs short — not because anyone forgot to re-seal them.

- **What that changes about #2477.** Its option 2 was "re-seal the 18 stale twins first, then delete 99". The first half is not available: 14 cannot be sealed until they parse. The 81 superseded seals with current twins are still a real, separate bookkeeping question; the 18 are not.

- **Both refutations came from running something that already existed**, not from building. Yesterday's rule — grep `--help` for the noun before writing a tool — fired before any code this time, and found `backlog`, whose entire purpose was the question I was about to re-ask.

- **Nothing was committed to the tree this iteration except this note.** Two of three proposals were wrong and the measurement says so; a commit that made the numbers move would have had to be invented.

Refs #2477, #2479
