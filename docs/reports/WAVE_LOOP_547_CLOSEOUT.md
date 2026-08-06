# Wave Loop 547 Closeout — Signed primitive scalar array function returns for independent VCD cross-check

**Issue:** #1518 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-547`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 547 extended W545/W546 primitive scalar array function returns to
signed element types (`[N]i8`/`[N]i16`/`[N]i32`).  The core backend gap was that
Verilog part-selects of signed packed vectors are unsigned by default, so
signed element reads (e.g. `a[0]` from a `[3]i8` packed local) lost their sign
before comparison or arithmetic.  The fix wraps such slices with `$signed(...)`
in generated Verilog and extends the Python cocotb reference model to bind and
type-infer test-block local variables so that signed VCD probes are interpreted
correctly.

---

## Deliverables

1. **`bootstrap/src/compiler.rs`**
   - `try_emit_primitive_array_access` now wraps packed primitive-array
     bit-slices with `$signed(...)` when the element type is signed.

2. **Scratch witnesses**
   - `specs/scratch/w547_signed_call_init_returns_array.t27`
     - `seq() -> [3]i8` returns `[-1, -2, -3]`.
     - `check() -> i8` sums the signed elements and returns `-6`.
   - `specs/scratch/w547_signed_element_compare.t27`
     - `seq() -> [3]i8` returns `[-1, -2, -3]`.
     - Test block binds `let a : [3]i8 = seq();` and asserts `a[0] == -1`.

3. **Python reference model**
   - `scripts/cocotb_ref_model.py`
     - Tracks test-block local variable types and temporarily binds their
       values while collecting assertions.
     - Resolves full declared types (including array dimensions) for primitive
       array element access so width/signedness inference is correct for locals
       inside test blocks.

4. **Formal model**
   - `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
     - Added `w547SignedCallInitReturnsArraySeq`, `Check`, `Env`, `Module`.
     - Added `w547SignedElementCompareSeq`, `Module`.
   - `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
     - Added lowerability and value-preservation theorems for both witnesses.
   - `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
     - Updated stale W544 comment that claimed primitive scalar array returns
       were rejected; they are now lowerable.

5. **Integration test**
   - `bootstrap/tests/icarus_lowerable.rs`
     - Added `accepts_w547_signed_primitive_scalar_array_return` to ensure the
       structural classifier accepts both new witnesses.

6. **Seals and baselines**
   - Saved seals for both new witnesses under `.trinity/seals/`.
   - Recorded Icarus baselines under `.trinity/icarus-baselines/`.

---

## Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 7/0 |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus 54/0, cocotb 54/0, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 `sorry` |
| Yosys smoke baselines | 24 pre-existing failures unchanged |

---

## Generated Verilog example

For `w547_signed_element_compare`, the signed packed return and slice access are
emitted as:

```verilog
function signed [23:0] seq; // -> [3]i8
    input _unused;
    begin : seq_body
        seq = {-8'sd3, -8'sd2, -8'sd1};
    end
endfunction

initial begin : signed_element_compare_test
    reg signed [23:0] a;
    a = seq(1'b0);
    if ((($signed(a[7:0])) != (-1))) begin
        ...
    end
end
```

The `$signed(...)` wrapper on `a[7:0]` ensures the comparison uses t27's
two's-complement semantics.

---

## Weak points closed

1. Signed packed-vector element slices are now emitted with explicit `$signed`
   casts, matching IEEE 1800-2017 signed-operator behavior.
2. Signed packed concatenation for array literals is exercised by the two new
   witnesses.
3. Signed function return declarations are validated end-to-end by Icarus
   simulation and the cocotb reference model.
4. The formal model has dedicated signed primitive-array return witnesses with
   lowerability and value-preservation theorems.

---

*φ² + φ⁻² = 3 | TRINITY*
