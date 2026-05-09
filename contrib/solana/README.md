# Trinity $TRI — Solana Mining Program

Anchor program for TTSKY26a $TRI token mining via PoUC (Proof of Useful Computation).

## Structure

```
programs/tri-mining/src/lib.rs  — Anchor program with GF(2^4) PoUC
tests/tri-mining.ts              — Integration tests (3 node mock)
```

## Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize_epoch` | Create mining epoch with block_reward |
| `submit_proof` | Submit NodeProof with phi-challenge response |

## Accounts

| Account | Size | Description |
|---------|------|-------------|
| `MiningEpoch` | 72B | epoch_id, block_reward, total_proofs, authority |
| `NodeProof` | 156B | miner, phi_response, merkle_root, signature, tokens_earned |

## Deploy (when Solana CLI installed)

```bash
solana config set --url devnet
anchor build
anchor deploy
anchor test --skip-deploy
```

## G-TRI-2 Acceptance

3 test nodes submit valid NodeProof → all receive mock TRI rewards on-chain.
