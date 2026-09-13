# CRM: the dashboard starts a duet, the list shows who paid, a witness reads the schema

Date: 2026-09-13. Issue: #3611. Follows #3604 (client workspace) and #3608 (ownership and money).

## What changed in the specs

`specs/automation/crm-client-workspace.t27`

- `DASHBOARD_SENDS_NOTHING` is now `DASHBOARD_SENDS_NOTHING_UNASKED`. Reads still send nothing.
  The one sender is the start-duet control on `/crm/:clientId`: it calls `crm_duet` through the
  same `/mcp` gate, is owner-only on the server (the UI shows the refusal text instead of guessing
  the role), defaults to `dry_run: true`, and a real send needs an inline confirm step first.
  Turns: default 4, max 8 -- the tool's own limits.
- The `/crm` list: a paid badge on rows with `paid: true`, a three-way filter
  `all,clients,leads` (clients = paid or stage `client`/`winback`), and one visible line when
  `paid_known` is false.

`specs/automation/crm-client-ownership.t27`

- `PAYING_CLIENT_SEEN_LIVE` flipped to true: after the #2390 deploy, `crm_history 435572800`
  answered `stage: client, paid: true, paid_known: true` from the owner's session on app.t27.ai
  (~21:25 +07). `crm_clients` showed two paid rows out of twenty; unpaid rows kept their
  touch stages.
- `crm_schema_check`: an owner-only, read-only tool that reports `owner_column`, `legacy_pkey`,
  `unique_index`, `rows_total`, `rows_owned`, `rows_unowned`, `owners`. It never calls the
  migration. `MIGRATION_RAN_ON_PROD` stays false until this witness answers
  `legacy_pkey: false, unique_index: true` on production.

## What is not claimed

- That the migration dropped the legacy primary key on production: only inferred so far (the
  owner_id backfill read back as `144022504`), which is why the witness exists.
- That a duet started from the dashboard was seen live; the control is specified and built, the
  first run from it is not yet recorded.

Host PRs: gHashTag/999-multibots-telegraf render (`crm_schema_check`) and player (badge, filter,
duet control) -- linked from the PR body.
