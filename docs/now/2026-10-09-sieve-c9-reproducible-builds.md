# NOW -- sieve C9 measured: t27b builds bit-identical, t27c does not (2026-10-09)

## seven builds of each binary on two labs (Closes #8148)

- Commit be9757784, rustc 1.99.0, x86_64 release, built seven times on the two Railway labs,
  varying the build path, clock, user, HOME, TZ, locale, umask, SOURCE_DATE_EPOCH (set and unset),
  clone depth, CARGO_HOME and the host.
- t27b: one sha256 in all seven (258950de...), and the aarch64 cross build agrees across its two
  builds (b0917f88...). Sieve C9 for t27b goes from UNKNOWN to TRUE.
- t27c: four binaries from seven builds. Two inputs that are not the source reach it. One is the
  length of the git short sha that build.rs bakes in: a depth-1 clone gives 7 characters and a
  full clone 9, and `core.abbrev 9` on the shallow clone gives back the full clone's hash. The
  other is the absolute CARGO_HOME path of 20 crates.io dependencies in panic locations (115
  occurrences). Sieve C9 for t27c goes from UNKNOWN to FALSE, with fix issue #8178.
  `--remap-path-prefix=$CARGO_HOME=/cargo` was measured to give one t27c (253be263...) across
  three cache paths and both labs.
- gen, gen-c and gen-rust agree across five runs and three distinct t27c binaries, on every slot
  of the 2037-slot corpus (stdout, stderr and exit code). A failed clone, which produced no
  output, is the negative control: the comparison flagged all 2037 slots as different.
- The rule and the data live in specs/compiler/theory/reproducible.t27 (10/10, 0 vacuous, now
  sealed). sieve.t27 moves T738-T741 on top of #8182 (C6/C7), whose be9757784 rows it reuses:
  t27c is FALSE on C1, C4, C5 and C9 and t27b on C1 only (12/12, 0 vacuous, resealed).
  One mutant per spec is caught.
