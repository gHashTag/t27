# NOW -- automation: queen-growth -- the Queen's next best action to earn (2026-10-02)

## automation: queen-growth v1 -- one person, their own data, read-only (Closes #5539)

- `specs/automation/queen-growth.t27`: seven ordered actions plus `unknown` (connect bot, first contacts, follow up an open invoice, first offer, win back, next offer, ask for a referral from 3 payers), decided only from the caller's own bot, CRM audience, open invoices and payers in the live wallet `token_ledger`.
- `SENDS_MESSAGES`, `PROMISES_EARNINGS` and `READS_PAYMENTS_ARCHIVE` are false constants, each with a test. An unreadable wallet is `unknown`, not zero; `may_read` holds only for an identified caller asking about themself.
- test-report 12/12 pass, 0 vacuous; the negative control (`<=` in the referral threshold) turns the referral test red. The host (999-multibots-telegraf render, tool `hive_growth`) binds all 12 titles; 9 of 9 mutants killed.
