# NOW -- Two readers of SKILL.md disagreed by three, and a red gate had been red for six runs (2026-09-05)

## Two readers of SKILL.md disagreed by three, and a red gate had been red for six runs (Refs #3265)

- tri skill check read 532 numbered sections and tri skill claims read 535 on the same file. Five readers walk the headings; three carried the CommonMark fence rule and section_bodies and section_ranges carried none, so three headings QUOTED inside code fences as evidence were counted as sections.
- One fence_toggle helper now serves all five. An opening fence may carry an info string, a closing fence may not - a naive toggle reads 441 sections here, 91 fewer than the truth, and it told me there were 94 phantom sections when there are 3.
- tri skill check exits 1 on master: 567 appears 3 times, 568 twice, out of order. cli-tri.yml calls it and has failed on master for six consecutive runs; it is not a required context, so nothing blocked. Renumbered to 570/571/572 by LINE; title set 535 = 535, zero missing, zero invented.
- Reported not fixed: every lesson I appended over several passes is a ### at end of file, so it lands under whichever numbered section is last. 567 and 568 now carry 7 and 6 sub-headings on unrelated topics, uncounted by claims and unvalidated by check.
