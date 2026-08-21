# Coq kernel `build` gate: never-green since 2026-04-06, now unblocked

- Identified the red `build` check on `master` as `.github/workflows/coq-kernel.yml` (workflow `Coq kernel`, job `build`), **not** `cli-tri.yml` — both emit a check named `build` and the check-runs API returns only the newest per name, so one masked the other.
- Established it was born broken, not regressed: all 12 `master` runs concluded `failure`, run #3 (2026-04-06) through run #173 (2026-08-21). Steps 5-9 of the job have never executed once.
- Fixed cause 1: the job runs `coqorg/coq:8.19` with `--user root`, but the image's initialised opam root belongs to the `coq` user at `/home/coq/.opam`; as root `HOME=/root` had no `.opam`, so opam exited 50. Set a job-level `OPAMROOT: /home/coq/.opam` rather than `opam init`, which would rebuild Coq into an empty root and leave `coqc`/`coqchk` pointing at the image's switch.
- Fixed cause 2, latent behind cause 1: `cd bootstrap && cargo build --release` then `./target/release/t27c` resolved to `bootstrap/target/release/t27c`, but `bootstrap` is a member of the root cargo workspace so the binary lands in the repo-root `target/`. Now `cargo build --release -p t27c` from the root.
- Step not weakened: no `|| true`, no `continue-on-error`, and `coq-flocq` still installed — `coq/Kernel/PhiFloat.v` has a hard `From Flocq Require Import IEEE754.Binary`.
- Noted but not changed: `.github/workflows/coq-proofs.yml` (job `compile-proofs`) carries the identical `--user root` opam defect; its `paths:` filter means this PR's CI could not verify a change to it.

Closes #2320
