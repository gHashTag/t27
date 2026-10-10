# NOW -- spec(automation): tri-claim-batch v2 -- refusals keep their reason; confirmations counted honestly (2026-10-09)

## spec(automation): tri-claim-batch v2 -- refusals keep their reason; confirmations counted honestly (Closes #8421)

- confirmations_for: one batch of at most 16 per request, sent at once, so 874 earnings are 55 wallet confirmations on W5 (219 on v4), not the 4 the v1 trace claimed.
- Every refusal keeps its reason: done (already_minted), passing (quorum or signer trouble, retried at most 2 more rounds) or final; a wallet that accepted a transaction proves no mint (SENT_IS_NOT_MINTED). From the Codex audit of 999 at 0dc5424e.
