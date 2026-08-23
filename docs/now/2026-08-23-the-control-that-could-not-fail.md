# NOW — a negative control that could not go red (2026-08-23)

Four gates had never been shown to fail. They have controls now. Writing those controls turned up a defect in one of the controls, which is the part worth recording.

- **`tri gates sweep` named four of twelve gates with no negative control at all**: catalog-integrity, catalog-count, vector-data, elab-ratchet. For each, green on the happy path was the whole of the evidence. That is the same evidence a gate that *cannot* fail produces.

- **One of the new controls was itself the defect it was written to remove.** The integrity control calls `check()` in-process for each planted fault — deliberately, so module-level `ROOT` can never resolve to `/`. That reasoning is sound. But nothing then covered the wiring from `check()` to the process exit code, which lives in `main()`. Measured, one variable changed: with `main()`'s `return 1` rewritten to `return 0`, the gate printed `OK` on a catalog with a dangling `source=` — completely dead — and every one of its seven cases still reported `RED, correct branch`.

- **The fix adds a layer rather than replacing one.** Two runs now spawn the whole program end to end against a planted tree, with the script *copied into* that tree so `ROOT` resolves there by the ordinary `parent.parent` rule. No `--root` flag, no environment override: nothing new that could aim a live gate at somewhere harmless.

- **My first attempt to measure this was itself a broken ruler.** I neutered every `return 1..4` in each file with one regex, including the returns inside `self_check` — so two controls "passed vacuously" when in fact I had disabled their reporting. Reading the printed output rather than the exit code showed catalog-count's control had detected the mutant correctly and merely lost the ability to say so. Only the one-variable rerun separated a real defect from my own instrument.

- **Two reach defects in `catalog-count-gate.yml`.** It runs `check_catalog_integrity.py` without listing it under `paths:`, so a one-line edit to that gate lands in a PR that never runs it. And it did not list `specs/numeric/*.t27` — while the gate exists *because* a glob deleted `gfternary.t27` and its catalog row survived. Deleting a spec, the founding incident, did not trigger the workflow written to catch it.

- **Three of the four controls were about to ship as self-tests nothing invokes** — the first class in the reach taxonomy, reintroduced by the very batch documenting it. All four are wired beside the gates they certify.

- **A claim was corrected rather than deleted.** `docs/NOW.md` said the integrity gate had been "verified red on each of those failures individually". That verification left no trace in the tree; a fortnight on it is indistinguishable from a gate only ever seen green. The sentence now carries its correction, and the control makes it true.

- **Recorded and not fixed:** the paper-divergence WARN in `check_catalog_count.py` fires on every run and will until an erratum lands, so it carries no information (#2466). I checked whether the row floor I added in #2443 caused this. It did not — `--strict-paper` returns the same exit code before and after that commit.

- **What the sweep still cannot tell you.** It reports zero gates without a control. It measures whether a control *exists*, not whether it can fail — which is the same substitution of a label for a property that the audit's tenth class is about, now in the tool doing the auditing.

Refs #2465, #2466
