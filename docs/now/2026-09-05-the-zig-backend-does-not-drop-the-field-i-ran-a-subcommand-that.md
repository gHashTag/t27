# NOW -- The Zig backend does not drop the field; I ran a subcommand that does not exist (2026-09-05)

Correcting a claim that reached four merged places today. The measured facts underneath
are unchanged and the corrected version is stronger than the wrong one.

## What I published and what is true (Refs #3225)

- I wrote that for `bad : 0,` the Zig backend "drops the field entirely, which is worse because nothing downstream can notice"
- that came from `t27c gen-zig`, which is **not a subcommand**. The Zig backend is `gen`. An empty output from a misspelled command was read as a dropped field
- the real behaviour: `gen` writes `bad: 0,` and, for the empty type slot, `empty: void`
- **both are accepted by `zig build-obj` AND by the deeper `zig test --test-no-exec`** -- measured on two four-line probes

## Why the corrected version is the stronger finding (Refs #3225)

- Rust rejects both shapes and C rejects both; Zig accepts both at both readings
- so the Zig column of the corpus counts these specs as generating and accepting, and no shape read from Zig output would flag `empty: void` either, because `void` is a legitimate Zig type
- the defect is not "one backend is careless with the field" but "one backend renders it as something its language genuinely accepts", which is a harder thing to detect and a better reason for `tri misread` to read Rust and C

## Blast radius, checked rather than assumed (Refs #3225)

- **the corpus numbers are untouched**: `bootstrap/src/service.rs:1287` invokes `["gen", &sp]`, so 581 generate / 308 accept / 190 analyse were never produced by the misspelling
- the fan-out agent that measured the Zig column reproduced 581/308/190 independently, so its findings stand
- the false sentence appeared in issue #3225, the docs/now entry of #3226, skill §566, and the module header of `cli/tri/src/misread.rs`; §566 and the header are corrected here and the issue carries a correction comment
- `tri misread`'s detectors and its 22/1/0/1 reading are unaffected -- they read Rust and C only

## The lesson, which I already had written down (Refs #3225)

- "the instrument answered zero" is in my own notes, and this is the same shape: a command that does not exist prints nothing, and nothing is indistinguishable from a clean result
- `t27c` prints `error: unrecognized subcommand 'gen-zig'` on stderr and I was piping only stdout
- the guard is one line: check the exit code of a probe before believing its silence, and the positive control I built INTO `tri misread` for exactly this reason is what should have been applied to my own shell probes
