# NOW — the auditor audited (2026-08-23)

Yesterday's command reported nine gates with surviving mutants. One more day of using it found two defects in the command, and both are classes it exists to find.

- **It ran one control per gate and invented a survivor.** The flag lookup took the first match. `check_duplicate_agreement.py` declares two, and the one it picked never reaches that gate's drift verdict; the other kills the mutant in a line. The table published in #2468 said `SURVIVED at 298` about a line that was fully covered. Nothing in the repository was wrong — the tool was. A mutant is now killed if **any** declared control notices.

- **Its control map was 1:1, so a shared control could not be wired.** Once one control covered the precondition branch of six gates, those branches kept reporting as survivors while the file that covers them sat in the tree — the "it exists but nothing connects it" defect, in the auditor rather than in what it audits.

- **"Six of the nine share that exact shape" came from a proxy, not from reading.** I classified by *does this gate keep a baseline file*. Reading all twenty sites: **seven are preconditions across six gates; thirteen are ordinary verdict branches**, one of them a gate's main verdict. Closing the precondition class does not fix those six gates, which the original sentence implied. Corrected on #2468 and appended to the published post.

- **One control now closes the precondition class for all six.** An empty tree makes every precondition fail at once — no baseline, no compiler, no specs, no seals — so this is one file rather than six bespoke cases, and a new gate joins by a table row. The script is copied **into** the empty tree so `ROOT` resolves there by the ordinary `parent.parent` rule: no `--root` flag and no environment override, so covering the class adds no new way to aim a live gate somewhere harmless.

- **It found a live defect on its first run.** `check_elab_ratchet.py` printed `SKIP: iverilog or target/release/t27c missing` and returned 0 — the ratchet greenlighting an unchecked tree and saying so out loud. Not exploitable in `fpga-conformance` today, because the iverilog install step exits 1 after three bounded attempts. But that guarantee lives in a **neighbouring step** and is invisible from the gate: move the gate to another job — which is what happened to its own control a day earlier — and it does not travel. The precondition now belongs to the gate.

- **Two stages, because preconditions hide each other.** A bare tree reaches "t27c not built"; a tree with the compiler copied in reaches "the scan matched nothing, the instrument is broken, not the tree" — the branches worth having. The first version of the table expected the second message from the first stage, and this file's own control is what said so.

- **What is not covered is named in a constant, not left to a count.** Two sites sit behind a tool check, so the message they print depends on the machine, and an assertion whose expected value varies is not an assertion. `UNCOVERED = 2` sits next to the reason.

- **A comment went stale inside the commit that wrote it.** The note naming those two sites said `:346` and `:390`; the SKIP fix eight lines above moved both to `:359` and `:403` before the branch was pushed. Branches are named by their message now.

- **Numbers.** Survivor sites 21 → 13, precondition sites uncovered 7 → 2, gates with survivors 9 → 8. The new control kills five mutants, including two the first draft missed: a `main()` that reds without naming the count, and a `main()` that does not red.

Refs #2468, #2469
