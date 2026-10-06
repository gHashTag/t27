# NOW -- the brain region tests get a capability card, one ahead of the pin (2026-10-04)

## specs/trinity/capabilities/brain.regions.t27 (gHashTag/t27#5953)

- gHashTag/trinity#1333 puts build steps for the brain regions back into trinity's build.zig:
  `test-basal-ganglia`, `test-reticular-formation`, `test-locus-coeruleus`, `test-brain` and
  `test-brain-stress`, over six `src/brain` test roots. The S01 checker, which trinity vendors
  byte for byte, found these eleven targets with no owning card. Trinity's capability index failed
  on each as UNASSIGNED_TARGET, and the "Headless profile from a clean clone" job went red.
- The new card owns exactly those eleven targets.
  - It is executable, zig, and in the headless profile: the steps sit outside every `!ci_mode`
    block.
  - Its acceptance is the five `zig build <step> -Dci=true` commands.
  - Its evidence is `measured`, from brain-ci runs 37184075728 (push) and 37184077352
    (pull_request) at gHashTag/trinity@291ac8b24. Every step's exit code was gated, with the
    same counts in both runs: 51/51, 48/48, 52/52, 151/151, and 261/261 with 0 leaked.
  - It cites no canonical spec. src/brain is handwritten Zig, and nothing generates it from
    specs/brain/ or checks it against that directory.
- `research.unreferenced-sources` no longer names `src/brain`; its NOTE says why. At the pin, 43
  of the 46 .zig files under src/brain are unreachable and stay in the 2082. At 291ac8b24, 37 are.

## The order problem, kept visible

- The cards are held to the inventory of the pinned tree gHashTag/trinity@976df517, and none of
  the eleven targets exists there. So `python3 tools/trinity_manifest.py check` now reports
  eleven UNKNOWN_TARGET findings, and the regenerated `conformance/trinity/report.json` records
  them; before this change it recorded none. No t27 workflow runs that check. The findings stay
  until S01 re-pins. The card's header, the README and this entry each say so.
- Checking the same cards against an inventory of gHashTag/trinity@291ac8b24 gives no finding for
  the card and no unassigned brain target. What remains there is drift: one PIN_MISMATCH and
  nine COUNT_MISMATCH.
- `--self-check` passes. The new card and the edited one are sealed with `t27c seal --save`,
  using a t27c built from this branch, and `seal --verify` matches both. Neither seal is
  reported by `tools/check_seal_coverage.py` or `tools/check_seal_currency.py`.

## Follow-up in gHashTag/trinity

- After this merges: run `tools/contracts.py vendor` against the new t27 master, and add
  `trinity/brain.regions` to `specs/reproduce/capabilities.t27`. That makes the headless
  profile job of gHashTag/trinity#1333 measure the card instead of failing on unowned targets.
