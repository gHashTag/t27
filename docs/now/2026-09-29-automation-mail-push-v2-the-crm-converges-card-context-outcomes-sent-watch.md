# NOW -- mail-push v2: the push and jcrm become one system (2026-09-29)

## specs/automation/mail-push.t27 -- VERSION 2, #5174

- v1 pushed the letter; v2 closes the other direction. jcrm (job-search
  crm/jcrm.py) already derives the whole campaign from this same mailbox by
  enumeration; the cloud push now reads jcrm back and writes what it did to
  tables jcrm reads -- one system, two watchers, one mailbox.
- The card carries the matter's CRM context (stage, who owes the reply, days
  since the last word) from a digest jcrm exports after every sync. A digest
  older than a day is refused, not shown -- the channels.json law, carried into
  the card: a stale number is never presented as current.
- The draft is composed with the matter's recent both-direction history, still
  with no tools and the letter still fenced as data. History is ours; the
  letter is a stranger's.
- Every settle is written back: an Allow records the sent reply, a Deny, an
  expiry or a foreign tap records the refusal. jcrm stops being surprised by
  what the cloud did with a letter the local index also sees.
- The Sent folder is watched. Our own tap-sends are recognised by the id the
  send itself returned; what remains new in Sent is a reply the owner typed by
  hand in the webmail -- recorded as evidence and paired with the draft that
  was offered for the same matter, which is how drafts get measured instead of
  argued about.
- Identity everywhere is the enumeration message_id: the search index is
  twelve days stale (measured 2026-08-28), and an id out of search is a lie.
- A daily heartbeat reads the cursor's `updated_at` and alarms when the sweep
  goes quiet for six missed ticks -- a cron that dies silently after someone
  else's deploy is the failure this closes.
- What this entry does NOT establish: that a letter has been carded. v1 is
  live and measured (the cron fired 10:30-10:50 on 2026-09-29, the debut
  baselined the Inbox, no letter arrived); RUN_LIVE still waits for the first
  real card, and the jcrm side (export, contact convergence) lands in the
  job-search repo beside this spec.
