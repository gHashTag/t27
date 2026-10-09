# NOW -- spec(automation): tri-wallet-chains v1 -- the TRI payout wallet on TON, EVM, Solana and TRON (2026-10-09)

## spec(automation): tri-wallet-chains v1 -- the TRI payout wallet on TON, EVM, Solana and TRON (Closes #8293)

- Four chains numbered as the mint oracle numbers them (ton 1, solana 2, evm 3, tron 4); TON binds per network, one EVM address for every EVM chain, one Solana address for every cluster, one TRON address for mainnet and its testnets.
- The verifying key never comes from the request: TON from the state init, EVM and TRON recovered from the signature, Solana is the address; 15 min window, 60 s skew, 5 open nonces.
- A phone's wallet browser only proposes an address; the person's confirmation in the mini app binds. TRI is paid on TON only until the bridge is live (CROSS_CHAIN_DOUBLE_MINT).
