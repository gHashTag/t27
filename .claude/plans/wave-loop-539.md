# Wave Loop 539 — Typed 64-bit VCD probe + full Python expression evaluator

**Issue:** #1510  
**Branch:** `wave-loop-539`  
**Status:** in-progress  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the two residual boundaries left by Wave Loop 538:

1. **Fixed 64-bit probe width** — replace `reg [63:0]` with width-typed probes
   (`reg [W-1:0]`) whose width and signedness are inferred from the t27 type of
   the `assert_eq` actual expression.
2. **Literal-only Python expected-value evaluator** — extend
   `scripts/cocotb_ref_model.py` with a recursive interpreter for the
   Icarus-lowerable expression subset, including variables, parameterless
   function calls, struct field access, scalar array indexing, binary/unary
   operators, and bit-accurate signed/unsigned comparison.

The final gate is `./scripts/tri test --icarus-lowerable --cocotb --fast` staying
at 0 cocotb failures and 0 seal mismatches.

---

## Literature & prior art (investigated)

1. **SyoSil, *Python-based verification environment using PyUVM and cocotb***  
   Whitepaper showing PyUVM/cocotb scoreboard + C/Python reference models.  
   <https://www.syosil.com/images/resources/osv_whitepaper-1.0.3.0.pdf>
2. **Gadde et al., *Towards Efficient Design Verification – Constrained Random Verification using PyUVM***  
   arXiv 2407.10317v1; reference model/scoreboard checking, signed/unsigned and
   width-sensitive comparison experience.  
   <https://arxiv.org/html/2407.10317v1>
3. **DVCon EU 2025, *FPGA Firmware Verification: a common approach for simulation and hardware tests***  
   Unified cocotb simulation + pytest hardware validation, shared scoreboard with
   reference model.  
   <https://dvcon-proceedings.org/wp-content/uploads/DVConEU_2025_paper_95.pdf>
4. **Verilator issues #5968 / #4174 / cocotb discussion #5268**  
   Real-world signed/unsigned and non-byte-multiple width mismatches between
   reference models and event-driven simulators; reinforces the need for explicit
   width/sign metadata in the reference model.
5. **angr `claripy`**  
   Python bit-vector AST evaluation with explicit width and signedness; the
   closest architectural precedent for evaluating hardware expressions in Python.

Key takeaways for W539:
- Width and signedness must travel with every value; never infer from the
  Python `int` alone.
- Non-byte-multiple widths and signed comparisons are the most common mismatch
  sources.
- A standalone Python evaluator is sufficient for a reference model, but it must
  mirror the Verilog packed layout exactly.

---

## Decomposed plan

### Phase 1: Compiler — typed probe emission
Files: `bootstrap/src/compiler.rs`

- Add `VerilogCodegen::expr_width_signed(&self, node: &Node) -> Option<(u32, bool)>`
  that reuses the existing `packed_width`/`packed_signed` helpers plus the
  field/index offset logic already present in the packed-vector emitters.
- In `gen_verilog_test`, replace the fixed `reg [63:0] _t27_probe_...;` with
  `reg [W-1:0] ...;` where `W` is inferred from the actual expression; fall back
  to 64 bits when inference fails.
- In `gen_verilog_test_stmt`, assign the probe using the same width.  Do not pad
  or truncate.
- Store a `Vec<(String, u32, bool)>` probe metadata list in the test preamble so
  the Python side can know each probe's width and signedness without re-inferring.
- Preserve `// synthesis translate_off` and `[PROBE]` debug lines for
  observability.

### Phase 2: Reference model — type inference
Files: `scripts/cocotb_ref_model.py`

- Add `type_width_signed(type_name: str) -> Tuple[int, bool]` for primitive t27
  types (`i8/16/32/64`, `u8/16/32/64`, `int`, `nat`, `bool`, `float`, `trit`,
  enums) and sized array syntax `[N]T`.
- Add `resolve_type(node) -> Tuple[Optional[int], Optional[bool]]` for
  expression nodes, recursively resolving identifiers from module/function
  declarations, struct fields, and array element types.
- Keep fallback `(64, signed)` for unresolved scalar-looking expressions; skip
  wide/non-scalar expressions explicitly.

### Phase 3: Reference model — expression evaluator
Files: `scripts/cocotb_ref_model.py`

- Build `_eval_expr(node, ctx, width, signed)` that evaluates:
  - literals
  - variable identifiers (scalar module/function/local variables)
  - parameterless function calls (look up the called function body, evaluate the
    single `return` expression)
  - struct field access (reverse declaration order, packed-vector layout)
  - scalar array indexing (`a[i]`, `a[i][j]`)
  - binary/unary operators with width-aware semantics
  - casts
- Represent every value as `(int, width, signed)`; after each operation mask to
  the result width and sign-extend if signed.
- Implement signed/unsigned comparison, division, remainder, and shifts.

### Phase 4: Reference model — VCD value comparison
Files: `scripts/cocotb_ref_model.py`

- Change `_VcdParser` to remember the declared width of each VCD identifier.
- In `_cross_check`, read probe metadata `(width, signed)`; interpret the raw VCD
  value with `width`-bit masking and signedness.
- Remove the old 64-bit signed/unsigned heuristic.
- Skip probes only when `width` is unknown or exceeds a reasonable VCD limit
  (keep 64-bit fallback to avoid breaking existing behavior).

### Phase 5: Validation
- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness`
- Update `bootstrap/stage0/FROZEN_HASH` and reseal specs if compiler-generated
  code changes.

### Phase 6: Closeout & next-wave cooperation variants
- Write `docs/reports/WAVE_LOOP_539_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W540_2026-07-08.md`.
- Advance `.trinity/current-issue.md` to W540.
- Update `.trinity/experience.md` and persistent memory.
- Update `.claude/skills/t27-wave-loop.md` if the charter refined.

---

## Risks

- Width inference must match Verilog packed layout exactly; any mismatch will
  manifest as cocotb failures on struct/array assertions.
- Python signed division/remainder semantics differ from Verilog for edge cases;
  handle with explicit two's-complement operations.
- Probe width changes alter VCD files and may require baseline resealing.
- Parameterless function calls need a minimal inlining evaluator; recursion is
  not in the lowerable subset.

---

*φ² + φ⁻² = 3 | TRINITY*
