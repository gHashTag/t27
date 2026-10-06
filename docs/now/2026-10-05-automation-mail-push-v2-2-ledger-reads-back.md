# NOW -- mail-push v2.2: the door opens both ways (2026-10-05)

## specs/automation/mail-push.t27 -- VERSION 2, #5174

- v2.1 sent the digest in through the render's own door. v2.2 opens the same
  door outward: a read-only tool `mail_push_ledger` returns what the cloud
  wrote -- the digest, the settle outcomes, the owner's feedback -- so jcrm
  reads its half of the circle back instead of trusting that it landed.
- The tool is SELECT-only, owner-gated, bounded (100 rows per table, an
  optional since-hours window) and names three tables; anything else answers
  "unknown table" rather than guessing.
- The pulse now reports the digest's age and whether it is fresh. A digest the
  laptop has not refreshed in over a day was invisible from the cloud side;
  it is now one field in the pulse, the same place the other alarms live.
- The ledger lives in a different database from the one PostgREST serves; the
  render's door is the only way in. That is why the readback goes through the
  door and not through a REST call.
- What this entry does NOT establish: that RUN_LIVE is on, or that a letter
  has been carded. The host implementation is merged in 999-multibots-telegraf
  (#3197) with this spec vendored byte-identical; the switch stays the owner's.
