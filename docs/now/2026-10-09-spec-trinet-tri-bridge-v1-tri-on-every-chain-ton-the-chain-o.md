# NOW -- spec(trinet): tri-bridge v1 -- TRI on every chain, TON the chain of record, lock-and-mint (2026-10-09)

## spec(trinet): tri-bridge v1 -- TRI on every chain, TON the chain of record, lock-and-mint (Closes #8302)

- TON is the chain of record and the only place new TRI is minted; EVM, TRON and Solana hold TRI only by lock-and-mint, so the bridged sum equals the vault and the 3^21 cap is counted once (answers mint_on_acceptance CROSS_CHAIN_DOUBLE_MINT by the road its README names).
- Three operation codes (TRI1 mint, TRI2 bridge mint, TRI3 release) inside the signed digest; an honest-majority quorum (M > N/2); attest only from each chain's finalized view; one spend per lock and per burn; a pause that never blocks a holder's burn.
- A payout on another chain is a vault mint plus a bridge mint, never a second minter. BRIDGE_LIVE stays false until audited, deployed and the attestor keys have holders (KEY_CUSTODY not guessed).
