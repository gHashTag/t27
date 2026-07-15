# Wave Loop 537 Plan — Close undefined-struct leniency in `Completeness.lean`

**Issue:** #1508
**Branch:** `wave-loop-537`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points audited

Wave Loop 535 aligned the Rust structural classifier and the Lean 4
`Trinity.IcarusLowerable` predicate, but it left one deliberate leniency in
`Predicate.lean`:

```lean
| fuel+1, .struct name =>
    let fields := env.structFields name
    fields.isEmpty || fields.all (fun p => Ty.isLowerableFuel fuel env p.2)
```

An undefined struct (one with no declaration in `env.structs`) is treated as
lowerable.  This was a temporary scaffolding decision to keep the large
simplified corpus model in `Completeness.lean` valid while the predicate was
being tightened.  It now creates a known soundness gap: the Lean predicate can
accept struct-typed variables/parameters that the Rust classifier would reject
because their declaration is missing or contains non-lowerable fields.

The corpus model encodes many source-level types that have no `Ty` variant
(`f64`, `usize`, `String`, `&str`, `[]f32`, etc.) as `.struct "<name>"`.  When
the struct name is undefined, the predicate currently returns `true`, diverging
from the Rust backend where `is_lowerable_scalar_struct` requires the name to exist
in `struct_decls` and every field to be a primitive scalar or fixed-size array
of primitive scalars.

Closing this leniency will make the Lean corpus model a faithful structural
mirror of the Rust classifier and unblock an automated Rust/Lean
classifier-equivalence regression test.

---

## 2. Scientific literature surveyed

- **CompCert** (Leroy, CACM 2009, “Formal verification of a realistic compiler”)
  — the canonical verified compiler in Rocq/Coq.  Its core theorem is semantic
  preservation: every observable behavior of generated code is an allowed
  behavior of the source.  CompCert splits the compiler into many passes over
  intermediate languages and composes per-pass simulation proofs.  W537 applies
  the same discipline at the *lowerability-predicate* layer: a gap between the
  source-side acceptability predicate and the backend classifier is a latent
  compiler bug, so it must be closed before the end-to-end soundness story is
  trustworthy.
- **Vericert** (Herklotz et al., OOPSLA 2021, “Formal Verification of High-Level
  Synthesis”) — extends CompCert with a verified C-to-Verilog HLS backend.  Its
  correctness theorem states that every observable behavior of the generated
  Verilog is a behavior of the source C program.  Vericert’s proof relies on a
  deterministic synthesizable Verilog semantics.  W537 mirrors this by ensuring
  the Lean lowerability predicate and the Rust structural classifier agree on
  exactly which t27 programs are in the synthesizable subset.
- **Lean4Lean** (Carneiro, arXiv:2403.14064v2) — a verified typechecker for Lean
  4 in Lean 4.  The paper explicitly discusses how soundness bugs crept into
  Lean 4 during kernel extensions and argues for independent reimplementation and
  formal verification as structural safeguards.  W537 follows the same
  “independent mirror” idea: the Rust classifier and the Lean predicate are two
  independent implementations of the Icarus-lowerable boundary, and they must
  agree.
- **cocotb + reference-model verification** (SyoSil whitepaper, 2024; LSC-Unicamp
  processor-ci-verification repo) — demonstrates Python reference models driven
  by cocotb, with trace/VCD comparison against the DUT.  W536 already added a
  cocotb gate to t27; W537’s predicate-alignment work is a prerequisite for making
  that gate semantically meaningful on the full lowerable corpus.

Key insight: a soundness proof is only as trustworthy as its least precise
assumption.  The undefined-struct leniency is currently the least precise
assumption in the Icarus-lowerable boundary, so it is the right needle for W537.

---

## 3. Decomposed plan

### 3.1 Decide the structural rule

Change `Ty.isLowerableFuel` in `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
so that `.struct name` is lowerable **iff** `env.structs` contains a declaration
for `name` and every declared field is lowerable:

```lean
| fuel+1, .struct name =>
    let fields := env.structFields name
    !fields.isEmpty && fields.all (fun p => Ty.isLowerableFuel fuel env p.2)
```

This matches the Rust rule in `bootstrap/src/compiler.rs`:
`is_lowerable_scalar_struct` returns `false` when the name is missing from
`struct_decls` and requires every field to be lowerable.

### 3.2 Repair `Completeness.lean` envs

The 249 corpus envs encode many source-level types (`f64`, `usize`, `String`,
`&str`, `[]f32`, etc.) as `.struct "<name>"` because the simplified `Ty`
inductive has no variants for them.  Under the stricter predicate, only envs
whose modules actually reference a non-lowerable struct type in `globals` or
`functions` will break; modules with empty `globals`/`functions` (and tests/benches,
which the predicate does not check) remain `true` automatically.

Running `lake build Trinity.IcarusLowerable.Completeness` after the predicate
change will surface the exact failing theorems.  Based on the pre-change audit,
only four active envs reference declared structs whose fields are typed as
undefined/non-lowerable placeholders:

| Module env | Struct | Non-lowerable field | Why it breaks | Repair |
|---|---|---|---|---|
| `fpga_bootrom` | `BootStage`, `BootConfig` | `name : .struct "&str"` | returned from `boot_stage`/`boot_config` | change `name` field type to `.u32` (the functions already pass `name` as `u32`) |
| `fpga_cts` | `PllConfig`, `ClockBuffer`, `ClockTree` | `name : .struct "&str"` | returned from constructors that take `name` as `u32` | change `name` field type to `.u32` |
| `fpga_dft` | `ScanChain`, `BistCtrl`, `JtagTap` | `name : .struct "&str"` | returned from constructors that take `name` as `u32` | change `name` field type to `.u32` |
| `fpga_power` | `PowerDomain` | `name : .struct "&str"` | returned from `power_domain` which takes `name` as `u32` | change `name` field type to `.u32` |

`benchmarks_bench_nn` and `fpga_verification_build_verify` contain undefined
struct names in their envs, but the corresponding modules do not reference those
types in `globals` or `functions`, so their theorems remain true.

The repair rule is: **only fix the field type in the simplified env; do not add
fake declarations for source-level non-lowerable types.**  This keeps the Lean
predicate aligned with the Rust classifier, which already rejects `&str` fields.

### 3.3 Add a Rust/Lean classifier-agreement regression test

The full corpus Rust→Lean equivalence test is out of scope for W537 (it needs
a JSON exporter for the simplified AST and a Lean JSON loader).  W537 adds a
focused regression for the undefined-struct boundary:

1. **Lean negative theorem** in `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
   define an env with a function returning `.struct "Missing"` where `"Missing"`
   is not declared, and prove `¬ Module.isLowerable env module`.
2. **Scratch negative witness** `specs/scratch/w537_negative_undefined_struct.t27`:
   a small t27 module with a function returning an undeclared struct type.
3. **Rust integration test** in `bootstrap/tests/icarus_lowerable.rs`:
   add `rejects_w537_undefined_struct_witness` that asserts
   `t27c icarus-lowerable --json` returns `lowerable: false` for the new scratch
   witness.

This gives both predicates a shared negative test case for undefined structs and
establishes the pattern for future classifier-equivalence tests.

### 3.4 Update documentation

- Extend `docs/ICARUS_LOWERABLE_BOUNDARY.md` with a “Rust/Lean predicate
  alignment” section explaining that undefined struct names are now rejected.
- Write `docs/reports/WAVE_LOOP_537_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W538_2026-07-07.md`.

### 3.5 Validation gates

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --cocotb --fast` (must stay 0 cocotb
  failures / 0 seal mismatches)
- `lake build Trinity.IcarusLowerable.Soundness` (must stay green with zero
  `sorry`)

No change to `bootstrap/src/compiler.rs` is expected in the recommended variant,
so `FROZEN_HASH` should not need updating.

---

## 4. Cooperation variants for Wave Loop 538

### Variant A (recommended): Full Rust/Lean classifier equivalence over the corpus

Automate the classifier-agreement check for every `*.t27` spec.  Add a Rust
subcommand that exports the simplified env/module as JSON, and a Lean program
that reads the JSON and verifies `Module.isLowerable` matches the Rust verdict.
Wire it into `./scripts/tri verify --lean-lowerable`.  This completes the
structural alignment work started in W537.

### Variant B: Independent Python expression evaluator + VCD trace comparison

Extend `scripts/cocotb_ref_model.py` with a recursive interpreter for the
lowerable expression subset (literals, arithmetic, function calls, array/struct
indexing).  Drive the generated Verilog as a DUT from cocotb, capture a VCD
trace, and compare DUT signal values against the independently computed
reference values.  This is the natural next step for the W536 cocotb gate.

### Variant C: Module-level procedural semantics in Lean 4

Extend the Lean 4 formal semantics to cover module-level `const`/`var`
initialization and whole-struct assignment, with a non-scratch corpus witness
in `specs/igla/`.  Prove value preservation for a representative module-level
packed scalar struct initialized from a struct literal.

---

*φ² + φ⁻² = 3 | TRINITY*
