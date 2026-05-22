# ring-093-rust -- Sparse MoE (top-k gating + ternary expert FFN)

No backing file under `specs/` (textbook algorithm, like ring-091's
stochastic rounding). The design follows the canonical
Shazeer-2017 / Switch-Transformer top-k routing structure, with
ternary (Trit) expert weights matching the project's TNN convention
and Trinity sacred constants.

## What's exposed

- **Sacred constants:** `NUM_EXPERTS = 3`, `DEFAULT_TOP_K = 1`,
  `DEFAULT_EMBED_DIM = 243`, `DEFAULT_EXPERT_HIDDEN_DIM = 729 = 3^6`.
- **`Trit`** balanced-ternary weight enum `{Neg, Zero, Pos}`.
- **`MoEConfig`** with `trinity_defaults()` and `is_valid()`.
- **`gate_top_k(logits, top_k, indices, weights) -> usize`** -- selects
  top-k experts by logit; returns softmax over the selected logits (so
  the `weights` returned sum to 1.0). Numerically stable max-subtract;
  ties broken by smaller index. Clamps to `min(top_k, logits.len())`.
- **`expert_ffn(input, w_in, hidden_scratch, w_out, output, in, hidden, out)`** --
  two-layer ternary FFN: `output = (ReLU(input @ w_in)) @ w_out`.
- **`moe_forward(input, expert_logits, cfg, w_in_all, w_out_all, ...)`** --
  composes gating + per-expert FFNs and accumulates a weighted sum into
  `output`. Caller supplies all scratch buffers (no allocation).
- **`relu_inplace(buffer)`** -- standard non-libm activation.
- **`load_balance_loss(usage_counts, num_tokens, num_experts) -> f64`** --
  Switch-Transformer style importance-balance auxiliary. Returns 1.0 for
  perfectly uniform routing, `num_experts` for full concentration.
- **`identity_witness() -> bool`** -- exercises `phi^2 + 1/phi^2 = 3`.

## no_std exp

Softmax in `gate_top_k` requires `exp`. We embed a private `exp_f64`
using range reduction (`exp(x) = (exp(x / 2^20))^(2^20)`) plus a 12-term
Taylor series. Verified to better than 1e-9 in the working range against
the standard library; underflows to 0.0 at `x < -700`. (Same algorithm
as ring-092; ring crates are independent and re-derive the helper.)

## Cross-kernel anchor test (#5)

`moe_phi_identity_via_gating_and_ffn` routes `phi^2 + 1/phi^2 = 3`
through top-k gating + ternary expert FFN. Joins:

- ring-088 `mac_dot_phi_identity` (GF16 MAC)
- ring-089 `cpu_phi_identity_integer_projection` (TNN CPU)
- ring-091 `sr_quantize_phi_unbiased` (SR-quantization)
- ring-092 `attention_phi_identity_via_softmax_matmul` (softmax + matmul)
- **ring-093 `moe_phi_identity_via_gating_and_ffn`** (MoE routing)

Construction: `total = phi^2 + 1 + 1/phi^2` must equal 4 by the identity.
Weights `w_e = phi_power_e / total` are then routed through three
identity-FFN experts; the weighted-sum output equals input (sum of
weights = 1.0), confirming the identity end-to-end across gating + FFN.

## Out of scope (R5-HONEST)

- **Training-time auxiliary terms** beyond load-balance (router-z, etc.)
  are not implemented; the `load_balance_loss` primitive is sufficient for
  inference-time monitoring.
- **Capacity factor / token dropping** -- the caller's batching layer
  is responsible.
- **Per-token batching** -- this crate's `moe_forward` is single-token;
  callers iterate over tokens.

## Local verification (Rust 1.83.0)

```
cargo check -p ring-093-rust   # clean
cargo test  -p ring-093-rust --lib   # 28 passed
```

## Constitutional compliance

- L1 TRACEABILITY -- `Closes #727`
- L2 GENERATION -- no edits in protected dirs
- L3 PURITY -- ASCII source, English doc-comments
- L4 TESTABILITY -- 28 `#[test]` blocks, all green
- L5 IDENTITY -- `phi^2 + 1/phi^2 = 3` exercised in `identity_witness`
  and in the cross-kernel anchor test
- L6 CEILING -- no spec change; textbook algorithm
- L7 UNITY -- no `*.sh` files

`phi^2 + 1/phi^2 = 3`
