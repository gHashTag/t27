# NOW -- The invoice is claimed before it is credited (2026-09-17)

## What was read

- `payment-ai-server-process` (999-multibots-telegraf) credited an invoice by `inv_id` through `updateUserBalance`: status set to COMPLETED unconditionally, `true` returned even when no row matched, and the amount in the event never compared with the amount that was invoiced.
- The Robokassa ResultURL route settles the same `payments_v2` rows with a compare-and-set on PENDING and credits only when it wins. The Inngest path had no such claim.
- The events of this function have no sender inside the repo (orphan-events baseline); who sends them was not established.

## What this spec says

- STEPS gains `claim-invoice` between `get-bot-config` and `update-user-balance`.
- The row must exist and its RUB amount must equal the rounded IncSum, else the run ends with NonRetriableError and the admin chat hears about it; nothing is credited.
- The PENDING -> COMPLETED flip is a compare-and-set; a lost race or an already COMPLETED row returns `already_claimed` without crediting or notifying.
- A database error on the claim stays retriable.

## Not verified

- The sender of `payment/ai-server.process`; the live path was not exercised.
- Behaviour on the deployed build: needs a deploy and a real or staged invoice.

## Where the code goes

- 999-multibots-telegraf, branch `payment-claim-invoice`: `core/supabase/claimPendingInvoice.ts`, `paymentProcessing.ts`, `functions.manifest.json`, test `money/paymentClaimInvoice.test.ts` citing this spec.
