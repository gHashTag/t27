# .len() is usize, rebuilt on a master that had moved

**2026-09-05** — Refs #3249

#3257 went DIRTY in the squash cascade for the third time. Rebuilt on today's master as a
new branch, because the branch could only be salvaged by rewriting its history and that is
not available.

Extracting the change with `git diff -- bootstrap/src/compiler.rs` produced a patch that
applied cleanly and **did not build**: the path filter dropped `bootstrap/stage0/FROZEN_HASH`,
which the M5 ceremony requires in the same commit. The build's own output made this harder
than it needed to be — forty lines of language-policy warnings about pre-existing docs sit
above the panic that names the real cause.

Measured, 651 specs, two binaries with distinct hashes:

    master  333 OK  248 FAIL  69 NOGEN
    here    335 OK  246 FAIL  69 NOGEN

`+ igla/race/cordic_top.t27`, `+ igla/race/opcodes.t27`, nothing lost. FAIL fell by exactly
the two that rose. Seals re-sealed after the measurement, not before: drift 25 -> 0.
