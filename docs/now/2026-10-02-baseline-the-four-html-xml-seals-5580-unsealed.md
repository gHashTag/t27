# NOW -- the four html/xml seals #5580 unsealed get their baseline lines (2026-10-02)

## tools/seal_baseline.txt -- 4 lines, Refs #5577

- #5580 restored 30 seals on the 15 specs whose own tests fail to their
  earlier state, so the seal gate reports them stale on purpose (#5577 tracks
  the repair). #5605 then wrote baseline lines for 26 of the 30 and missed
  four: TriHtml.json, TriXml.json, encoding_TriHtml.json,
  encoding_TriXml.json. Since #5605 merged, Seal Coverage (`coverage`) has
  failed on master and on every PR with exactly those four, for example
  #5605 itself and #5610.
- Their specs still fail their own tests under `t27c test-report`
  (specs/tri/encoding/html.t27 1/3, xml.t27 0/1), so re-sealing would be the
  wrong repair -- `seal --save` refuses a failing spec since #5605. The lines
  use the same wording as the other 26; "pre-#5572" because #5572 is the
  change these two seals were rolled back past.
- Measured locally with the compiler present: `OK: 1415 seals, 1291 hold,
  124 known-broken (89 dangling, 5 no-spec-path, 30 stale)`. 30 stale is
  the #5580 set exactly.
