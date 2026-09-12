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
| `../vsa/trinity_compat.t27` | `vsa_trinity_compat` (`KIND = "vsa-compat"`) | the VSA and numeric contract the consumer's facade actually runs: the owner of each of the sixteen re-exported operations, where the canonical `vsa_core.t27` agrees and differs, the selected reference as elementwise functions (S04) |
| `conformance/vsa_trinity_compat.json` | -- | the golden vectors of the reference and what `tools/trinity_vsa_compat.py run` replayed through the generated C |
| `../isa/ternary_encoding.t27` | `Tri27Encoding` (`KIND = "isa-encoding"`) | the TRI-27 instruction word of `src/tri27/emu/decoder.zig`: forty-seven opcodes, three layouts, the fifteen-bit immediate, the rules the owner never wrote down (S05) |
| `../isa/tri27_machine.t27` | `Tri27Machine` (`KIND = "isa-machine"`) | the TRI-27 machine of `executor.zig`: registers, memory, fetch, entry profiles, flags, every opcode's numeric rule, stack, budget, errors and status codes, exit codes, host boundary (S05) |
| `../isa/tri27_bytecode.t27` | `Tri27Bytecode` (`KIND = "isa-bytecode"`) | the `.tbin` container of `loader.zig`: header, sections, every rejection in order, where the code lands (S05) |
| `../vm/trinity_vm.t27` | `TrinityVsaVm` (`KIND = "vm-contract"`) | the VSA VM of `src/vm.zig`: opcode numbering, registers, step and run, condition codes, no budget, no serialized form, the sacred opcodes (S05) |
| `../api/c_abi.t27` | `TrinityCAbi` (`KIND = "host-abi"`) | the twenty-two exports of `src/c_api.zig` with prototypes, ownership and NULL rules; the source does not parse at the pin (S05) |
| `conformance/trinity/tri27_programs.json` | -- | twenty-two golden programs and eighteen loader vectors with final state, status and bounded trace; what `tools/trinity_tri27.py run` replayed through the generated C |
| `conformance/trinity/c_abi.json` | -- | what `tools/trinity_c_abi.py check` measured: header, source exports, agreement, the fixture's syntax check, `zig ast-check` |
| `conformance/trinity/abi/abi_fixture.c` | -- | the ABI fixture: ownership, NULL safety, clamping, normalization, the bind/unbind round trip; compiles against the header, not linked at the pin |
| `../tools/catalog.t27` | `ToolsCatalog` (`KIND = "tools-catalog"`) | the tools catalog at schema 2: repository-qualified IDs, legacy resolution, witnesses, vocabularies, the counts the consumer measures to (S06) |
| `../tools/trinity/tri/<command>.t27` | `tool_trinity_tri_<command>` (`KIND = "tool"`) | one card per command the Trinity tri exports in `.trinity/registry.json`, 29 at the pin (S06) |
| `../tools/mcp_protocol.t27` | `McpProtocol` (`KIND = "mcp-protocol"`) | what the Trinity MCP server does with a JSON-RPC message, method by method, at the pin (S06) |
| `conformance/trinity/tools_inventory.json` | -- | what `tools/trinity_tools_registry.py inventory` measured: registry, table, dispatch layers, the server literal, the help witness of the t27 tri |
| `conformance/trinity/mcp_fixtures.json` | -- | fourteen offline JSON-RPC fixtures and what `run` replayed through the generated C of the protocol spec |

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

## The VSA compatibility contract (S04)

`../vsa/trinity_compat.t27` (module `vsa_trinity_compat`, `KIND = "vsa-compat"`) is the
contract behind `trinity:src/trinity.zig`, which re-exports sixteen VSA operations from the
package the consumer pins (gHashTag/zig-golden-float `e7ce3288`, `src/vsa/core.zig`, passed
through gHashTag/zig-hdc `b73b2fa2` name by name). `specs/vsa/vsa_core.t27` describes the same
names with other semantics, so the spec records, per export, the owner function (file:line),
the canonical function (name:line), a finding and a verdict: 4 agree on every input
(`permute`, `inversePermute`, `countNonZero`, `vectorNorm`), 6 agree on equal lengths only
(`bundle2`, `bundle3`, `cosineSimilarity`, `hammingDistance`, `hammingSimilarity`,
`dotSimilarity`) and 6 differ in kind (`bind` and `unbind` -- the owner annihilates on a zero
trit, the canonical spec keeps it as an identity -- `encodeSequence`, `probeSequence`,
`randomVector`, `bundleN`). The semantics the pinned consumer executes are the reference of
the Trinity profile and are stated as elementwise functions the C backend runs, because the
compiler does not lower array parameters (S02): `trit_bind`, `trit_bundle2`, `trit_bundle3`,
`trit_differs`, `trit_nonzero`, `len_max`, `len_min`, `rotate_right`, `rotate_left`,
`packed_bytes`, `cosine_defined`, `tri27_bind`. The zero-identity bind of `vsa_core.t27` and
`ops.t27` and the additive TRI27 register bind of `src/tri27/emu/executor.zig` are classified
separately, each with its own function. The numeric representation is declared: i8 trits, five
to a byte, two capacities in one package (12000 packed, 59049 hybrid), SIMD width 32, an i32
dot accumulator in the owner, f64 similarities, no GoldenFloat format on the facade, no
allocation and no error in the sixteen operations.

`tools/trinity_vsa_compat.py vectors` writes `conformance/vsa_trinity_compat.json` from a
Python statement of the owner's semantics (149 vectors over 15 operations: equal lengths, one
SIMD chunk, a chunk plus a remainder, unequal lengths, all-zero and empty vectors, ties, the
rotation identities and the inverse law, the TRI27 fix-up; the pseudo-random trits come from the
tool's own LCG, not from the owner's PRNG). `run` generates the spec to C with `t27c gen-c`,
composes the elementwise functions into whole-vector operations in a C driver whose length
rules are the spec's own `len_max`, `len_min` and `rotate_*`, replays every vector and records
the verdict with the compiler, the C compiler and the spec hash. `check` holds the committed
record to the spec and the model; `--self-check` proves a wrong expected trit fails exactly its
vector and that a zero-identity bind planted into the spec fails only bind-shaped vectors.

Measured on 2026-09-12: `All 10 tests passed.` for the spec on C; 149 of 149 vectors pass the
replay. Not measured: the owner's Zig itself -- the package does not build with the Zig
available on this host (0.15.2 cannot link against the host SDK, the 0.17 nightly rejects the
package), so the reference is read from the owner's source and the spec says so
(`NOT_MEASURED`); a differential run against the owner remains the owner's own test suite.

```
python3 tools/trinity_vsa_compat.py vectors
python3 tools/trinity_vsa_compat.py run
python3 tools/trinity_vsa_compat.py check
python3 tools/trinity_vsa_compat.py --self-check
```

## The VM, the TRI-27 bytecode and the host ABI (S05)

The consumer has two virtual machines and one C boundary, and none of the three had an
executable contract. `../isa/ternary_encoding.t27`, `../isa/tri27_machine.t27` and
`../isa/tri27_bytecode.t27` state the TRI-27 layer as the owner's tests execute it --
`src/tri27/emu/decoder.zig`, `executor.zig` with `cpu_state.zig`, and `loader.zig` at the pin --
rule by rule, as functions the C backend runs: field extraction and encoding, opcode validity,
the numeric rule of every executed opcode, flags, bounds, stack, budget, jumps, and every
container check. `tools/trinity_tri27.py vectors` assembles twenty-two golden programs in the
decoder's layout and runs them through a Python model of the same three files, writing
`conformance/trinity/tri27_programs.json` with final registers, flags, memory changes, status and
a bounded trace, plus eighteen loader vectors (one per rejection rule and per accepted shape).
`run` generates the three specs to C, links them into a driver that owns the arrays and the loops
and calls the specs for every decision, and replays. `check` holds the record to the specs and the
model; `--self-check` plants a wrong register, a rejection declared as acceptance, a `BIND` without
its zero identity and a wrong sign bit. The status codes are the machine spec's own, because the
owner's error set has no ordinals and three of its halts are silent: `halted` is 0, everything else
is nonzero.

What the replay found, and the specs record as findings: `CALL` pushes its own address and `RET`
returns to it, so every subroutine call loops until the budget; `loader.load` writes eight-byte
words while `run` fetches four-byte units, and reads a `tri_asm.zig` container two bytes early, so
nothing it loads runs as written (the owner's tests copy the file to byte 0 and start at word 3);
`DOT`, `BIND`, `BUNDLE2` cannot carry a second register and `SACR` cannot carry its mode through
the encoding; no encodable load or store can leave memory; the `CONSTANTS` section's id byte is
read as its count; `3^27` is written where `3^9` is computed; five instruction layouts, two
assemblers and an emitted template disagree with the decoder.

`../vm/trinity_vm.t27` states the VSA VM of `src/vm.zig` (twenty-six opcodes numbered by
declaration order, four hyperdimensional registers over S04's operations, no budget, no
serialized form, forty sacred opcodes of which six are implemented) and records its CI evidence as
it is: `zig test src/vm.zig` fails to compile on every push to main. `../api/c_abi.t27` states the
twenty-two C exports with prototypes, ownership and NULL rules; `tools/trinity_c_abi.py check
--trinity-root` holds the table to the header and to the source (all twenty-two agree), compiles
`conformance/trinity/abi/abi_fixture.c` against the header, and records that `zig ast-check`
rejects `src/c_api.zig` at the pin (a duplicated `if` in `trinity_vsa_set_trit`), so the library
and its tests do not build and the fixture is not linked.

Measured on 2026-09-12: `All N tests passed.` on C for the five specs (11, 10, 6, 5, 4); 40 of 40
vectors pass the replay; 22 of 22 ABI functions agree across spec, header and source; the fixture
compiles. Not measured: anything on the owner's Zig (no build target, binaries that do not compile,
no Zig on this host that builds the tree); the string and file opcodes; `SACR` modes 2..4; the VSA
VM's programs; the ABI across a linked boundary. No hardware is involved.

```
python3 tools/trinity_tri27.py vectors
python3 tools/trinity_tri27.py run
python3 tools/trinity_tri27.py check
python3 tools/trinity_tri27.py --self-check
python3 tools/trinity_c_abi.py check --trinity-root <clone at PINNED_REVISION> [--zig <zig>]
python3 tools/trinity_c_abi.py --self-check
```

## The CLI and MCP commands from one registry (S06)

`../tools/catalog.t27` names the one artifact the consumer's own binary exports and its CI holds
to the binary -- `.trinity/registry.json`, 29 commands, the `mcp_enabled` subset of a 187-entry
table -- as the registry the Trinity cards derive from, and states the catalog at schema 2: every
card under `specs/tools/` carries `REPO`, `QUALIFIED_ID` (`<owner>/<repo>:<family>/<name>`) and
`SCHEMA = 2`; the 62 legacy cards keep their short `ID` and gain the qualified one; a Trinity card
(`../tools/trinity/tri/<command>.t27`) has only the qualified ID, so a same-named command of the
two binaries (`fpga`, `test`) is never selected by the wrong repository, and a legacy ID resolves
through a 62-pair table and nothing else. Each Trinity card carries the registry's fields, the
dispatcher's routing (six of the 29 reach no dispatcher at the pin), the exit-code and result
rules as they are, and the witness `registry-export`. `../tools/mcp_protocol.t27` states what
`tools/mcp/trinity_mcp/server.zig` does with a JSON-RPC message: stdio only, substring method
matching, no negotiation, silence for notifications and unknown methods, a 210-tool literal of
which 32 objects are malformed JSON, exact-prefix-generic call routing with no refusal, failures as
`isError` results, no cancellation, no timeout, no permission gate at the server.

`tools/trinity_tools_registry.py inventory --trinity-root <clone> --tri target/release/tri`
measures the clone and the built t27 `tri` (the `help-output` witness of the 52 t27 cards: 52 of
52 agree) into `conformance/trinity/tools_inventory.json`; `check` holds the 29 cards to the
registry field by field (an unexplained addition or removal fails), every card to schema 2, and the
two contracts' counts to the measurement; `fixtures` and `run` write and replay fourteen offline
JSON-RPC fixtures through the generated C of the protocol spec (14 of 14); `--self-check` plants
the defects each must catch.

Not measured: any Trinity command or MCP tool running -- no host here builds the binary, and no
workflow runs one without `|| true` or a server at all. The site generator in gHashTag/trinity
reads only `tri/` and `mcp/` and refuses unknown constants; schema 2 needs its companion change
before the vendored copy is refreshed.

```
python3 tools/trinity_tools_registry.py inventory --trinity-root <clone at PINNED_REVISION> --tri target/release/tri
python3 tools/trinity_tools_registry.py check
python3 tools/trinity_tools_registry.py fixtures
python3 tools/trinity_tools_registry.py run
python3 tools/trinity_tools_registry.py --self-check [--trinity-root <clone>]
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
