# specs/trinity -- the Trinity project manifest as `.t27` specs

> **Where this lives.** `specs/trinity/` in `gHashTag/t27` is the canonical home of the project
> manifest of `gHashTag/trinity` -- edit it here. The consumer vendors a byte-identical copy under
> `apps/website/public/t27/files/specs/trinity/` and generates its own manifest from that copy
> (S12 of gHashTag/trinity#988). Nothing in the consumer is a second authored registry.

Work package **S01** of the epic gHashTag/trinity#988 (gHashTag/t27#3563): the canonical inventory
of what `gHashTag/trinity` ships at a pinned revision, one capability per card, each with an owner,
a disposition, a canonical spec, its implementation and generated paths, its backend, its build
targets, the command that is its acceptance, the status of its evidence and the work package that
owns its contract.

## The files

| file | module | what it declares |
|---|---|---|
| `project.t27` | `trinity_project` (`KIND = "project"`) | the consumer repository and the pinned revision, the profiles, the disposition and evidence vocabularies, the dialect and build counts the inventory measured, the dependency pins and the twelve work packages |
| `capabilities/<id>.t27` | `trinity_capability_<id>` (`KIND = "capability"`) | one capability; `ID = trinity/<id>` |
| `compiler_matrix.t27` | `trinity_compiler_matrix` (`KIND = "compiler-matrix"`) | the executable t27 subset of the headless profile: compilers, backends, stages, features with fixtures, negatives (S02) |
| `build_graph.t27` | `trinity_build_graph` (`KIND = "build-graph"`) | the consumer's toolchain and CI commands, pins and their use, profiles, untracked outputs, generated tracked files with generators and inputs, the receipt policy (S03) |
| `conformance/trinity/build_graph.json` | -- | the derived graph: modules, import edges, packages, profiles, generated files hashed at the pin |
| `conformance/trinity/bootstrap_receipt.json` | -- | the bootstrap/fixture-profile receipt: compiler identity, fixtures and every generated output hashed over two runs |
| `conformance/trinity/compiler_matrix.json` | -- | what `tools/trinity_compiler_matrix.py run` measured, stage by stage and backend by backend |
| `conformance/trinity/inventory.json` | -- | the inventory of the pinned tree, written by `tools/trinity_manifest.py inventory` |
| `conformance/trinity/report.json` | -- | what `tools/trinity_manifest.py check` derived: counts, cards by disposition, profile, evidence and work package, findings |

## How the cards are held to the tree

```
python3 tools/trinity_manifest.py inventory --trinity-root <clean clone of gHashTag/trinity at PINNED_REVISION>
python3 tools/trinity_manifest.py check
python3 tools/trinity_manifest.py --self-check     # negative control: thirteen planted defects, each reported
```

`inventory` refuses a tree with a modified tracked file. It reads `build.zig` (every
`addExecutable` / `addTest` / `addLibrary`, every `b.step`, every `installArtifact` and the `if`
that guards it), `build.zig.zon`, `.gitmodules`, the tracked tree (one entry per directory), the `.t27` / `.tri` /
`.vibee` / `.zig` counts with the website mirror (`apps/website/public/t27/files/`) set apart,
the reachability of every `.zig` file from the files `build.zig` names through relative
`@import`, `.trinity/registry.json` and the vendored catalog counts.

`check` fails on: a pinned revision or a count that differs from the inventory; a dirty
inventory; a target `build.zig` defines that no card owns, or two cards own, or a card owns
that the build does not define; a default-installed target on a non-headless card or a
`!ci_mode`-guarded target on a headless one; a `trinity:` path that is not tracked; a mirrored
file cited as canonical; a `DIALECT` that disagrees with the extension of `CANONICAL_SPEC`; a
backend claimed by a card that is not executable, adapter or research; two owners for one
canonical spec; two cards with one `ID`; `EVIDENCE = "measured"` without `ACCEPTANCE` and
`EVIDENCE_SOURCE`; a work package without a card or a card naming a package the project does
not list. The compiler remains the authority on the files: `t27c typecheck` and the seals under
`.trinity/seals/` cover every file here.

## Vocabulary

**DISPOSITION** -- what the capability is at the pinned revision:

| value | meaning |
|---|---|
| `executable` | built from source in the consumer; a backend is named |
| `declared` | present as a declaration, a placeholder or a corpus; nothing executable is claimed |
| `adapter` | handwritten code that wraps an external service, a GUI toolkit or a host, retained until generation is demonstrated |
| `external` | owned and implemented in another repository, consumed through a pin |
| `deprecated` | retained history or a duplicate; not a source of truth |
| `out-of-scope` | named so that it is not silently omitted; no work package owns it |
| `research` | an optional research artifact; its results are not project claims |
| `catalog-only` | a public snapshot (the Queen catalog); never implementation coverage |

**PROFILE** -- `headless` (the initial reproducible profile: what `zig build -Dci=true` installs
on `ubuntu-latest`), `web`, `native`, `fpga`, `training`, `network`, `research`. A target
installed by default is headless; a target guarded by `!ci_mode` is not.

**EVIDENCE** -- the five tags of `docs/system/project.md`: `measured` (a run whose output is
in a repository or a public CI log, command and revision recorded in `EVIDENCE_SOURCE`),
`declared`, `specified`, `external`, `not-claimed`. Catalog presence, source parsing,
`typecheck.ok`, an empty test set or a module shell never count as measured.

**TARGETS** -- `exe:<name>`, `lib:<name>`, `test:<root source path>`, `step:<name>` as
`build.zig` defines them; `fnmatch` patterns are allowed (`test:src/trinity_node/*`).

**Paths** -- `trinity:<path>` is a tracked file or directory of the pinned consumer tree; a bare
path is a file of this repository; `<repo>:<path>` names another repository and is recorded,
not checked.

## What the inventory measured at gHashTag/trinity@976df517 (2026-09-12)

- 51 executables, 6 libraries, 73 tests and 68 steps in `build.zig`; 46 targets installed by
  `zig build -Dci=true`, 5 guarded by `!ci_mode` (the raylib canvas and the node GUI).
- 31 `.t27` files outside the website mirror, 1044 inside it; 764 `.tri`; 1981 `.vibee` (1428 of
  them under `deploy/trinity-nexus`); 2830 `.zig`, of which 749 are reachable from the 173 files
  `build.zig` names and 2081 are not.
- Four pinned dependencies (`emsdk`, `raylib`, `zig_hdc`, `zig_golden_float`) and one submodule
  (`external/zig-golden-float`, a second, unpinned reference to the same repository).
- 29 commands in `.trinity/registry.json`; the vendored catalog holds 856 distinct specs from 8
  repositories -- a snapshot, never coverage.
- The measured evidence of the headless profile is one public CI run (`Build & Test`,
  ubuntu-latest, zig 0.15.2) in which `zig build -Dci=true` succeeded; the test step is piped
  through `tee` there and its exit code is not measured (gHashTag/trinity#616).

## The compiler matrix (S02)

`compiler_matrix.t27` (module `trinity_compiler_matrix`, `KIND = "compiler-matrix"`) names the
native compiler (`t27c 0.2.0`, `NATIVE_REVISION` = the last commit that changed `bootstrap/`),
the vendored WASM of the site (an older revision, its own column, no runtime), the four
backends with the command that emits each and what proves a runtime result on each, the four
stages kept apart in the record, fourteen features with one fixture each under
`bootstrap/tests/fixtures/trinity_matrix/`, and five negatives with the latest stage at which
each must be rejected. `tools/trinity_compiler_matrix.py run` carries every fixture through
`t27c typecheck` and `parse --json` (annotation agreement, import resolution, declaration
count), `gen-c` / `gen` / `gen-rust` / `gen-verilog`, `cc -DT27_TEST_MAIN`, `zig test`,
`rustc --test`, `iverilog` and `t27c icarus-simulate`, and writes
`conformance/trinity/compiler_matrix.json`; `check` holds the committed record to the spec and
the fixture hashes; `--self-check` proves the negatives are negatives.

Measured on 2026-09-12 (compiler sources at `bff21b85`, host clang 21, rustc 1.94, Icarus 13,
zig 0.17.0-dev nightly because zig 0.15.2 cannot link on the host): 12 of 14 features reach a
runtime pass on their backend; `enums` is blocked on C (the enum declares `EVIDENCE_EMULATOR`,
the switch compares against `EMULATOR`) while it executes on Zig; `ffi_boundary` is blocked at
declaration (a bodyless signature is a parse error, gHashTag/t27#3472). The Rust backend emits
declarations only (`NOT LOWERED`) for every fixture. The Zig backend does not lower string
equality, array constants, invariant blocks or the clocked form. All five negatives are
rejected: the invalid annotation and the unresolved import by this tool at declaration (the
compiler and the vendored WASM accept both with `typecheck.ok`), the false assertion at
runtime, the bodyless function and the dropped module by the parser.

```
python3 tools/trinity_compiler_matrix.py run --zig <zig> --wasm <trinity>/apps/website/public/t27/t27_compiler.wasm
python3 tools/trinity_compiler_matrix.py check
python3 tools/trinity_compiler_matrix.py --self-check
```

## The build graph and the receipts (S03)

`build_graph.t27` (module `trinity_build_graph`, `KIND = "build-graph"`) declares what the
consumer's build is made of: the Zig the consumer requires and the one its CI measures with,
the CI build and test commands (and that the test exit code does not reach the job,
gHashTag/trinity#616), the `build.zig` options, the four `build.zig.zon` pins with how
`build.zig` actually uses each, the submodule, the seven profiles with what is measured for
each, the untracked build outputs, the fourteen tracked files a generator writes (with the
generator and the inputs it reads), the seven directories that are generator output
locations, and the receipt policy of the bootstrap/fixture profile (two independent runs,
no normalization). `tools/trinity_build_receipt.py graph --trinity-root` derives the graph
from the pinned tree into `conformance/trinity/build_graph.json` -- every named module with
its root, every import edge resolved to a module, a package module or an inline module -- and
hashes each generated file and its inputs at the pinned revision (blob hashes for files,
tree hashes for directories). `receipt` writes `conformance/trinity/bootstrap_receipt.json`:
the compiler-source revision, the `t27c` hash, the host, and the hash of what each backend
generated from each matrix fixture in two runs. `check` recomputes what can be recomputed
offline (the compiler sources, `Cargo.lock`, the fixtures and a fresh generation must match
the receipt) and, with `--trinity-root`, judges drift in the consumer: a generated file that
changed while its inputs did not is a suspected hand edit, inputs that changed while the file
did not make it stale, and JSON in a generated location that no entry registers is
unregistered. `--self-check` plants each of those.

Measured on 2026-09-12 at gHashTag/trinity@976df517: 44 named modules, 252 import edges (84 to
named modules, 152 to the package modules `zig-hdc-vsa` of zig_hdc and `golden-float` of
zig_golden_float, 16 to inline modules, 0 unresolved); `emsdk` is pinned but not referenced by
`build.zig` directly; the receipt of the bootstrap/fixture profile is deterministic: 2 runs
over 18 fixtures x 4 backends, 72 outputs, 0 differing (gHashTag/t27#3006's wobble does not
reach this profile); the consumer's 14 generated tracked files show no drift at the pin.

```
python3 tools/trinity_build_receipt.py graph --trinity-root <clean clone at PINNED_REVISION>
python3 tools/trinity_build_receipt.py receipt --runs 2
python3 tools/trinity_build_receipt.py check --trinity-root <clone>
python3 tools/trinity_build_receipt.py --self-check
```

## Boundaries

- No card claims that a test passes, that a benchmark number holds or that a model answers
  well; `measured` on an executable means the artifact built in CI.
- The `.tri` and `.vibee` corpora and the ten `.t27` programs under `src/tri27` are `declared`
  until S02 states which compiler proves what for each dialect.
- `trinity:t27/` is a stale second copy of this repository's compiler and numeric specs; the
  card `specs.t27-vendored-compiler` records it as `deprecated` with `gHashTag/t27` as owner.
- A local build on macOS was attempted and is not recorded: the local zig 0.15.2 could not link
  a hello-world against the Xcode 26 SDK, which is a host defect, not a finding about the tree.
