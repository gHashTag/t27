# Where t27 actually stands

Written 2026-08-18 after reading what the alternatives do, rather than assuming. The point of
this document is to name what is occupied, so effort goes where something is not.

## The claim, as the tools state it

The verification scripts print lines like `ALL TARGETS BIT-EXACT` and describe themselves as
closing *"one spec → any target, bit-exact"* across {Verilog, C, Rust, model}. That phrasing
invites two readings, and only one is supported.

## What is already occupied

**Multi-target generation from one source — occupied.** Chisel elaborates to FIRRTL and CIRCT
lowers it to Verilog *and* to a C++ cycle-accurate simulator from the same source; this is the
standard flow, not a frontier. High-level synthesis has gone C/C++/SystemC → RTL for two
decades. A language that emits more than one target is not, by itself, a position.

**Proving target-to-target equivalence — occupied, and by stronger methods than ours.** Formal
equivalence checking between an RTL design and a C model is industrial practice: Synopsys HECTOR
does sequential equivalence checking; the ACL2/Restricted-Algorithmic-C line translates Verilog
to C++ to ACL2 and produces mechanically checked proofs against architectural specifications,
in production use for floating-point datapaths. Those are proofs over *all* inputs.

## What this repository actually proves, precisely

Two different strengths, and until 2026-08-18 the output line did not distinguish them:

| function | input space | what is checked |
|---|---|---|
| `ternary_mul(a: i8, w)` | 256 × 256 = **65,536** | **every input**, C and Rust against the model, FNV-1a digest `6b2724c5` |
| `ternary_mac(acc: i32, …)` | 256 × 256 × 2³² ≈ 2.8 × 10¹⁴ | 800 sampled vectors, edge cases included |
| `gft_smul` / `gft_sadd` | ~4.3 × 10⁹ | 600 sampled operands — 1.4 × 10⁻⁷ of the space |
| `systolic_ternary_pe` | i16 accumulator | 800 sampled vectors |

So one arm is exhaustive and the rest are **randomised differential testing**. That is a real
technique with a real name, and it is weaker than formal equivalence checking. The verdict lines
now say which is which.

The repository does carry genuine formal work — 11 Coq developments under `coq/`, with a CI gate
that rejects any `Admitted` proof. They are about the kernel and φ, **not** about backend
equivalence, and should not be cited as if they were.

## What is genuinely less occupied

Three things, and they are narrower than the slogan:

1. **The target set.** Chisel emits Verilog and a C++ *simulator*. HLS goes C → RTL. Neither
   emits a standalone **Rust** or **Zig** library from the same source as the RTL. t27 does.
2. **No vendor licence anywhere in the path.** The flow is `t27c` → Yosys → nextpnr → prjxray,
   reproducible from a pinned container digest. HECTOR and the commercial EC tools are the
   opposite of that.
3. **Exhaustive verification is reachable here, and is not for the alternatives' targets.**
   A ternary primitive over an 8-bit operand and a 2-bit weight has 65,536 inputs. Enumerating
   that costs milliseconds. Small ternary domains make *complete* cross-target agreement
   affordable, where a 32-bit float datapath forces you to either sample or invoke a prover.
   That is the one place where the ternary choice buys a verification advantage rather than
   an area claim.

## What to stop saying

- **"bit-exact across targets"** without a qualifier, when three of four arms are sampled.
  Say *exhaustive* where it is exhaustive and *sampled over N* where it is not.
- Anything implying the Coq proofs cover the backends. They do not.
- Treating multi-target emission as the differentiator. It is table stakes; the target *set*
  and the licence-free path are the differences.

## What would strengthen the position, in order of cost

1. Push exhaustive coverage up the datapath wherever the domain allows it — every ternary
   primitive whose input space is under ~2²⁴ can be enumerated rather than sampled.
2. For the arms that cannot be exhausted, state the sampled fraction next to the result, as the
   table above does. A reader can then judge it.
3. If a proof over all inputs is wanted for the wide arms, that is an equivalence-checking or
   theorem-proving problem with existing literature, not something to reinvent.

## Sources

- [Chisel](https://www.chisel-lang.org/) and [chipsalliance/chisel](https://github.com/chipsalliance/chisel) — single source to Verilog and a C++ simulator via FIRRTL/CIRCT
- [Formal Verification of Arithmetic RTL: Translating Verilog to C++ to ACL2](https://arxiv.org/pdf/2009.13761) — mechanically checked proofs against architectural specs
- [Automated Formal Equivalence Verification of Pipelined Nested Loops in Datapath Designs](https://arxiv.org/pdf/1712.09818) — industrial sequential equivalence checking (HECTOR)
- [Translation Validation for an Optimizing Compiler](https://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf) — the source-to-target equivalence framing
