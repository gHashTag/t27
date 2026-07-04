# Cooperation Variants for Next IGLA CODER IGLA RACE Loop (Wave Loop 149)

## Variant 1: Targeted Hardness — Pool B Return

**Focus:** Rotate back to Pool B (`systolic_array`, `systolic_ternary`, `ternary_mac`, `adder_tree`, `opcodes`, `yosys`, `backend`, `ternary_gemm`) and add 2 tests per spec, raising total to 570+ tests.

**Pros:**
- Fast average coverage increase per spec.
- Closes "blind zones" in RTL verification and hardware modules.

**Risks:**
- Possible cascading `seal` changes due to affected dependencies.

---

## Variant 2: Competitive Intelligence Surge — E8 Crowding Defense

**Focus:** Instead of adding 16 tests, allocate resources to:
1. Deep-dive analysis of Nythe (viXra:2601.0095) factorial E8 formula — can it be falsified or reconciled with Trinity's phi-monomial approach?
2. Wilson (arXiv:2507.16517) so(7,3) embedding — document differences from Trinity's H4 600-cell chain.
3. Prepare a public "E8 Crowding Defense" note differentiating Trinity from all E8-based competitors (Singh, Agyemang, McGirl, Nythe, Wilson, GIFT, GSM).

**Pros:**
- Strategic positioning refresh.
- Establishes Trinity's unique value proposition in a crowded market.

**Risks:**
- Time-intensive literature review may crowd out engineering tasks.

---

## Variant 3: Hybrid + Debt Payoff — FPGA Throughput Benchmarks

**Focus:** Add 8 tests (instead of 16) to the 4 weakest Pool B specs, and simultaneously:
- Add real FPGA throughput benchmarks (TOPS/W, tok/s) to `systolic_array.t27` or `ternary_gemm.t27`
- Fix C-backend `f32` -> `float` mapping for FFI Python bindings
- Document the 5 remaining Coq Axioms with explicit experimental falsification criteria

**Pros:**
- Balance between coverage growth and technical debt reduction.
- Infrastructure reliability improvement.

**Risks:**
- Smaller test increment per iteration.

---

**Recommendation:** For W149, **Variant 1** (Targeted Hardness) is recommended to maintain the Pool A/B alternation rhythm and coverage growth cadence, alternating with Variant 2 every 2-3 loops to defend against E8-crowding.
