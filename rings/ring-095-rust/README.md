# ring-095-rust — phi-Adam Optimizer

T27 Wave 22 — Rust import of `specs/ml/optimizer/{adam, adamw}.t27`.

AdamW (Loshchilov & Hutter 2019) with optional **phi-damped betas**
(`PHI_BETA1 = 0.9 / phi`, `PHI_BETA2 = 0.999 / phi`) and optional AMSGrad
(Reddi et al. 2018). no_std, zero deps, allocation-free `step()`.

## What is inside

- **Spec constants** mirrored byte-for-byte from `adamw.t27`:
  - `DEFAULT_LEARNING_RATE = 1e-3`, `DEFAULT_BETA1 = 0.9`,
    `DEFAULT_BETA2 = 0.999`, `DEFAULT_WEIGHT_DECAY = 0.01`,
    `DEFAULT_EPSILON = 1e-8`, `DEFAULT_AMSGRAD = false`
  - `PHI_BETA1 = 0.9 / phi ~= 0.556`, `PHI_BETA2 = 0.999 / phi ~= 0.617`
- **Types:** `AdamWConfig`, `AdamWState<'_>` (caller-owned buffers),
  `StepResult`, `OptimError`
- **Helpers (spec-named):** `compute_bias_correction`, `update_first_moment`,
  `update_second_moment`, `apply_weight_decay`, `compute_update`
- **Math primitives (no libm):** `pow_u64` (fast exponentiation),
  `sqrt_newton` (Newton-Raphson square root)
- **Orchestrator:** `step()` — one full AdamW / phi-Adam step in place
- **Anchor:** `identity_witness()` returns `phi^2 + 1/phi^2 = 3`

## phi-Adam preset

`AdamWConfig::phi_preset()` selects the phi-damped betas the spec
explicitly carves out:

| Hyperparameter | Classic AdamW | phi-preset       |
|----------------|---------------|------------------|
| beta1          | 0.9           | `0.9 / phi`      |
| beta2          | 0.999         | `0.999 / phi`    |
| use_phi_betas  | false         | true             |

The phi-damped betas converge faster but oscillate more; both modes share
the same `step()` codepath, gated by `use_phi_betas`.

## Out of scope

- GF16 wrapping — the spec aliases `gf16::GF16` to a float scalar; we use
  `f64` directly.
- libm — `pow(beta, t)` uses fast exponentiation; `sqrt(v)` uses
  Newton-Raphson.
- LAMB / Adagrad / RMSProp / SGD — separate specs, separate rings.

## Constitutional compliance

- **L1 TRACEABILITY** — `Closes #731` on every commit.
- **L2 GENERATION** — zero edits under `specs/`, `gen/`, `coq/`, etc.
- **L3 PURITY** — ASCII source, English doc-comments.
- **L4 TESTABILITY** — 25 `#[test]` blocks, all green on Rust 1.83.0.
- **L5 IDENTITY** — `phi^2 + 1/phi^2 = 3` exercised by anchor #7.
- **L6 CEILING** — spec constants mirrored byte-for-byte.
- **L7 UNITY** — no shell scripts added.

## Local verification

```
$ cargo check        # clean
$ cargo test --lib   # 25 passed; 0 failed
```

`phi^2 + 1/phi^2 = 3`
