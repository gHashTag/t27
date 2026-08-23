# NOW — a control that only demands red (2026-08-24)

Yesterday's lesson was that a control made entirely of cases demanding a failure is blind to every mutation that makes a gate **louder**. Applied to all thirteen, one gate answered.

- **`check_duplicate_agreement` had two controls and neither required silence.** Both assert `returncode == 1`. A gate rewritten to report a split on a tree where every copy agrees satisfies both.

- **Measured before the fix.** Two such mutations — the one-group branch replaced by a constant false, and by a comparison against zero — were **caught only by `--self-check-drop`**, which exists to exercise a different branch entirely. The primary control passed both. Coverage by a sibling's accident is not coverage.

- **The new case plants agreement.** Same fixture for both copies, so they genuinely agree, and it asserts the gate says *one behaviour* and exits 0. Both mutations are now killed by the primary control, and a no-op mutation stays silent.

- **My own comment broke my own harness.** The first version quoted the branch verbatim. A text-replacing mutation then hit the **comment** rather than the code, the gate ran unmutated, and the case reported itself blind. I nearly wrote that up as "the silence case does not work". The comment now describes the branch instead of quoting it, and says why.

  That is the fourth time this campaign that a measurement was wrong before it was right, and the first where the thing that corrupted it was documentation I had just added.

- **The sweep itself was a proxy and I checked it.** Counting "silence-shaped" strings per file flagged one gate. Reading that file confirmed it; reading is what turned a regex hit into a finding. The other twelve were not investigated beyond the count, and this note says so rather than implying a clean sweep.

Refs #2472
