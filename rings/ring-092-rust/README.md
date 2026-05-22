# ring-092-rust -- Attention (Sacred Attention primitives)

Mirrors `specs/nn/attention.t27` (SacredAttention) -- the subset that is
realizable in pure `no_std` Rust without libm.

## What's exposed

- **Sacred constants** (byte-for-byte from spec):
  `NUM_HEADS = 3`, `HEAD_DIM = 81`, `EMBED_DIM = 243`, `CONTEXT_LEN = 81`,
  `ROPE_PAIRS = 40`, `SACRED_GAMMA = phi^-3 ~ 0.2360679...`,
  `SACRED_SCALE = 81^(-SACRED_GAMMA) ~ 0.3543788557382518`.
- **`Trit`** -- balanced-ternary weight enum `{Neg, Zero, Pos}` with `value() -> i8`.
- **`ternary_matmul(input, weights, output, in_dim, out_dim)`** -- matrix-vector
  product with ternary weights. Equivalent to spec's `ternary_matmul`.
- **`add_residual(output, input)`** -- in-place residual sum (clamped to
  `min(output.len(), input.len())`).
- **`apply_softmax(scores, seq_len)`** -- per-head softmax with max-subtract
  numerical stabilization. Operates on a `NUM_HEADS * CONTEXT_LEN` buffer.
- **`compute_scores(q, cache_k, position, seq_len, scores)`** -- Q.K^T per head,
  multiplied by `SACRED_SCALE`, with a causal mask (positions `j > position`
  forced to zero).
- **`weighted_values(scores, cache_v, seq_len, concat)`** -- softmax-weighted V sum.
- **`cache_kv(k_buffer, v_buffer, position, cache_k, cache_v)`** -- KV cache store
  at offset `position * EMBED_DIM`.
- **`identity_witness() -> bool`** -- exercises `phi^2 + 1/phi^2 = 3`.

## no_std exp

Softmax requires `exp`, which is unavailable in `no_std` without libm. We
implement a private `exp_f64` using range reduction
(`exp(x) = (exp(x / 2^n))^(2^n)` with n=20) plus a 12-term Taylor series.
Verified against the standard library to better than 1e-9 across the
working range; underflows to 0.0 for `x < -700`.

## Cross-kernel anchor test (#4)

`attention_phi_identity_via_softmax_matmul` routes the constitutional
identity `phi^2 + 1/phi^2 = 3` through softmax-style normalization and a
ternary matmul. This is the fourth cross-kernel anchor in the ring set,
joining:

- ring-088 `mac_dot_phi_identity` (GF16 MAC)
- ring-089 `cpu_phi_identity_integer_projection` (TNN CPU)
- ring-091 `sr_quantize_phi_unbiased` (SR-quantization)
- **ring-092 `attention_phi_identity_via_softmax_matmul`** (softmax + ternary matmul)

## Out of scope (R5-HONEST)

- **RoPE table init** (`sacred_attention_init`) requires `cos`/`sin`; we don't
  implement these without libm. Constants `ROPE_PAIRS` and shapes are still
  exposed.
- **`sacred_attention_kernel` orchestrator** -- the full single-position
  forward pass that composes project_qkv -> apply_rope_qk -> cache_kv ->
  compute_scores -> apply_softmax -> weighted_values -> project_output ->
  add_residual. The primitives this crate exposes are sufficient to
  compose it externally.
- **`project_qkv` / `project_output`** -- thin wrappers in the spec; they
  reduce to `ternary_matmul` calls which we provide directly.

## Local verification (Rust 1.83.0)

```
cargo check -p ring-092-rust   # clean
cargo test  -p ring-092-rust --lib   # 28 passed
```

## Constitutional compliance

- L1 TRACEABILITY -- `Closes #725`
- L2 GENERATION -- no edits in protected dirs
- L3 PURITY -- ASCII source, English doc-comments
- L4 TESTABILITY -- 28 `#[test]` blocks, all green
- L5 IDENTITY -- `phi^2 + 1/phi^2 = 3` exercised in `identity_witness` and
  in the cross-kernel anchor test
- L6 CEILING -- sacred constants mirrored byte-for-byte from spec
- L7 UNITY -- no `*.sh` files

`phi^2 + 1/phi^2 = 3`
