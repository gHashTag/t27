# NOW -- Solana tri-mining and depin.prove rewards marked superseded by mint-on-acceptance (2026-10-01)

## Solana tri-mining and depin.prove rewards marked superseded by mint-on-acceptance (Closes #5412)

- contrib/solana README and tri-mining lib.rs carry a SUPERSEDED / do-not-deploy note: a fixed block_reward per proof is not the TRI emission rule.
- specs/depin/prove.t27 gets a comment after the module declaration saying reward_lamports and block_reward are not the TRI emission rule; parse and typecheck output equal master's apart from line numbers.
- Design of record: gHashTag/trinity-fpga @ d7e9718e9, contracts/solana/tri_mint.rs and specs/trinet/mint_on_acceptance.t27 (accepted work only, M-of-N quorum, genesis 0, cap 3^21, nothing deployed).
- Resealed with t27c built from this master: the four gen hashes are unchanged; only spec_hash and sealed_at move, in both seal files naming the spec.
