# Trinity $TRI — Solana Mining Program

> **SUPERSEDED — do not deploy.** This Anchor program (ea15cd54c, 2026-07-05)
> pays a fixed `block_reward` per submitted proof. That is not the TRI emission
> rule. The design of record is mint-on-acceptance in gHashTag/trinity-fpga @
> d7e9718e9: TRI is minted only for accepted work, attested by an M-of-N
> attestor quorum, with genesis supply 0 and a cap of 3^21 TRI. Its Solana side
> is [`contracts/solana/tri_mint.rs`](https://github.com/gHashTag/trinity-fpga/blob/d7e9718e90346ec4b4b9e6eef6cbebe8617cd173/contracts/solana/tri_mint.rs), and
> the rule is [`specs/trinet/mint_on_acceptance.t27`](https://github.com/gHashTag/trinity-fpga/blob/d7e9718e90346ec4b4b9e6eef6cbebe8617cd173/specs/trinet/mint_on_acceptance.t27).
> Nothing is deployed on any chain. This directory is kept as a historical
> PoUC experiment.

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
| `NodeProof` | 164B | miner, phi_response, merkle_root, signature, tokens_earned |

## Test (local validator)

```bash
# Start local validator
solana-test-validator --reset --quiet &

# Wait for readiness
solana cluster-version --url http://127.0.0.1:8899

# Deploy program
solana program deploy target/deploy/tri_mining.so \
  --url http://127.0.0.1:8899 \
  --program-id target/deploy/tri_mining-keypair.json

# Fund test wallet
solana airdrop 100 $(solana-keygen pubkey ~/.config/solana/id.json) --url http://127.0.0.1:8899

# Run tests
anchor build
rm -rf tests-compiled && npx tsc
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 \
ANCHOR_WALLET=$HOME/.config/solana/id.json \
npx mocha --timeout 1000000 tests-compiled/tests/tri-mining.js

# Stop validator
pkill -f solana-test-validator
```

## Deploy to devnet

```bash
solana config set --url devnet
solana airdrop 2 $(solana-keygen pubkey ~/.config/solana/id.json) --url devnet
anchor build
anchor deploy
```

## G-TRI-2 Acceptance

3 test nodes submit valid NodeProof -> all receive mock TRI rewards on-chain. **PASSED** (3/3 tests, 5s).
