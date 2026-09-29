# NOW -- mail-push v2.1: the digest travels through the render's own door (2026-09-30)

## specs/automation/mail-push.t27 -- VERSION 2.1, #5174

- v2 shipped the digest through PostgREST, and the first live export
  refused to land: the gateway behind SUPABASE_URL serves a DIFFERENT
  database than the render's ledger (measured -- mail_push_state answers
  42P01 through the gateway while the sweep writes it). A PostgREST
  crm_matter_digest would be a second home for the truth nobody reads.
- DIGEST_DOOR = mail_push_digest: jcrm export speaks JSON-RPC to the
  render's /mcp door (initialize, initialized, tools/call), the only
  door that reaches the ledger db. 132 matters exported live on
  2026-09-29, host bound in 999 PR #3195.
- exported_at is stamped by the receiver's clock: the 26 h freshness
  rule keeps exactly one arbiter, and jcrm's laptop stops having an
  opinion about freshness.
- The same live round caught the sent-column hotfix (999 #3194):
  COALESCE($3, '{}') typed the cursor's sent expression as text and
  text[] refused it -- every v2 save failed until the parameter became
  a real array. The door's jsonb parameter carries an explicit cast for
  the same reason: an untyped expression is a future 500.
