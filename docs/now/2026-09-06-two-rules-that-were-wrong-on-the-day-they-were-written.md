# NOW -- Two rules that were wrong on the day they were written (2026-09-06)

## Two rules that were wrong on the day they were written (Closes #3368)

- LOOP-RULES R12 said `Refs #N` does NOT satisfy `check-linked-issue`. The gate's matcher has carried `Refs?` since 2026-07-07; the sentence was written 2026-08-20 -- 44 days late on arrival. Seven readers in the tree accept it; R12 was the sole dissenter, and it demanded the one spelling R11 forbids nineteen lines above.
- R12 also still told authors to write to `docs/NOW.md`, frozen since #2298 with 'do not add entries here' on its first line. Same instruction in `.claude/skills/spec-first-ternary-nn.md:62`.
- The dictionary is NOT re-transcribed in the repair -- R12 now points at `issue-gate.yml:69`, because a hand-copied copy missing `Refs` once matched 4 references where the gate matched 33, and this sentence was the next copy to go wrong.
- The obvious guard is declined by measurement: over 600 commits since the freeze, 2 touched `docs/NOW.md` and one REPAIRS damaged entries, so 'fail any diff containing it' would have blocked a legitimate repair. The narrow rule -- fail an ADDED `## ` heading -- is a required-gate change and is filed, not done.
