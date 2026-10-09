# NOW -- spec(automation): tri-wallet-chains v1 -- the TRI payout wallet on TON, EVM and Solana (2026-10-09)

## spec(automation): tri-wallet-chains v1 -- the TRI payout wallet on TON, EVM and Solana (Closes #8293)

- Three chains numbered as the mint oracle numbers them (ton 1, solana 2, evm 3); TON binds per network, one EVM address for every EVM chain, one Solana address for every cluster.
- The verifying key never comes from the request: TON from the state init, EVM recovered from the signature, Solana is the address; 15 min window, 60 s skew, 5 open nonces.
- A phone's wallet browser only proposes an address; the person's confirmation in the mini app binds. TRI is paid on TON only until the bridge is live (CROSS_CHAIN_DOUBLE_MINT).
