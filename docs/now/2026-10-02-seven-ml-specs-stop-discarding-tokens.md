# NOW -- Seven ml specs stop discarding tokens (2026-10-02)

## Repair the specs/ml/ files the parser was throwing away (Refs #5497)

- The last seven core specs the corpus ratchet reported as newly discarding tokens after 2026-09-12 are under specs/ml/: gelu_approx_activation, sigmoid_activation, contrastive_loss, sgd, attention_mechanism, ppo_actor, sac_critic.
- given/when/then blocks in pseudo-syntax (`[]f32{len = 3; ...}`, `new f32[2]{...}`, `[0.0] * 12`, `then result == void`, loops inside clauses) become braced tests that keep their checks: batch lengths and signs, the SGD step, attention output, gradients and causal masking, PPO probabilities summing to one with bounded log_std and actions, the SAC soft update moving targets toward their sources.
- contrastive_loss carried its loop body twice; the copy read `anchor[anchor_len]` and its `}` closed the function before the average. sigmoid's `std.meta.fields` reflection test is removed; the `@sizeOf` test beside it pins the empty config.
- All seven parse with nothing discarded and generate in every backend as before; they are resealed. The two remaining unexpected failures are typechecker defects filed as #5573 and #5574. Published figures: test blocks -1, `x.len` field reads -8, `abs(` -1, each with its reason.
