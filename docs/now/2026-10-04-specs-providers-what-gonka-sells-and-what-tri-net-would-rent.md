# NOW -- specs/providers: what Gonka sells and what TRI-NET would rent (2026-10-04)

## specs/providers: what Gonka sells and what TRI-NET would rent (Closes #5788)

- specs/providers/gonka/*.t27: one card per model the Gonka network sells (MiniMax M2.7, DeepSeek V4 Flash 0731, GLM-5.3 Flash, GLM-5.2 FP8, Kimi K2.6); every number copied from node3.gonka.ai at epoch 413 (models_all, params, /v1/governance/pricing, /v1/epochs/current/participants). No participant address, seed or validator key is recorded.
- specs/providers/trinet/*.t27: two host classes an owner could rent to TRI-NET for $TRI -- fpga-xc7a200t (measured on the bench) and gpu-consumer-24gb (planned: 14 cards make one 320 GB Gonka node; no GPU worker, job format or verifier exists yet).
- specs/providers/tri_gnk_pair.t27: how Gonka pays its hosts, four routes to a $TRI/GNK pair (Ethereum Uniswap against WGNK, a Gonka-native pool, Osmosis over IBC, TON or Solana) and seven parallels with TRI-NET. It decides none: $TRI is on no mainnet, and the chain is the owner's call.
- specs/providers/catalog.t27: kinds, families, statuses, witnesses and the field each family must carry; 6 tests.
- Invariants are written 'invariant x { assert EXPR; }': the '{ EXPR }' form compiles to a discarded bool and test-report reports BLOCKED, and 'assert (A - 1) * B < C' emits unreachable code, so gpu-consumer-24gb says A * B - B < C. A planted false invariant blocks the build in both specs tried.
- t27c built from this tree: parse-complete 9/9 consume all (0 discarded); test-report catalog 6/6 + 2 invariants, tri_gnk_pair 8/8, every card's invariants proved at comptime; seals saved with the same binary.
- Vendored byte-identically to gHashTag/trinity apps/website/public/t27/files/specs/providers by the companion gHashTag/trinity PR for issue gHashTag/trinity#1301, which merges after this one.
