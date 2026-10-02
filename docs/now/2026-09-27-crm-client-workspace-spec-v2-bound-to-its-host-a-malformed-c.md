# NOW -- CRM client workspace spec v2: bound to its host, a malformed client is refused and never turned into self (2026-09-27)

## CRM client workspace spec v2: bound to its host, a malformed client is refused and never turned into self (Closes #4856)

- specs/automation/crm-client-workspace.t27 VERSION 2: thread_kind(client_named, digits, is_caller, is_seller) -> self / client / bad client (400) / not a seller (403), mirroring the host's threadFor. v1's thread_for turned a malformed or own id into the seller's own thread; the host refuses it, and a request about somebody never quietly becomes one about oneself.
- THREAD_SUPERVISOR_SUFFIX :queen (the Queen's thread beside the agent's, 2026-09-24), AGENT_KEEPS_CLIENT_MEMORY (a specialist in a client's chat writes to client:<id>, 2026-09-26), PANELS 7 (the story-reel panel, 2026-09-26). Tests braced.
- Host: 999-multibots-telegraf conversation.ts (SELF_THREAD, CLIENT_ID_RE, clientThread, supervisorThread from the generated module), crm-duet-tool.ts (LOST_AFTER_MS), src/spec/crm-client-workspace.test.ts runs all six tests against threadFor, viewOf, the tool registry and the player's routes and panels.
- Checked: t27c test-report 6/6 (Zig 0.16.0), validate-vacuity 0 of 6; negative control: thread_kind without the caller check -> FAIL. Seal saved.
