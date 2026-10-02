# agent-conversations spec: several chats per agent

Refs #4841.

- Added `specs/automation/agent-conversations.t27`: the logic 999-multibots-telegraf
  ships for several conversations with one agent (own thread and each hired
  specialist split into named conversations), as compiler-checked assertions --
  the gate order (base thread first, then the conversation id), which threads
  split, and one stream per conversation. `t27c spec-status` -> IMPLEMENTED.
- Sealed: `.trinity/seals/automation_automation::agent_conversations.json`.
