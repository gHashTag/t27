# NOW -- ten ways a gate lies, written down (2026-08-23)

Refs #2325.

- Every gate in `tools/` was put to five questions: does it count what it
  claims, can it pass having done nothing, does its scope match its prose, can
  its ledger be lowered silently, and is anything it says about itself false
  today. 63 candidate findings, **43 survived independent refutation**.
- The list was the evidence; the ten CLASSES are the product, and they are now
  `ci-gates` section 14. The most repeated is **departure scored as repair** --
  six gates, one line to fix in each, and one gate in the tree already did it
  right while citing as its model the gate that did not.
- Seven of the classes had a correct implementation already present somewhere
  in this repository. The defect was never ignorance of the pattern; it was
  that nobody applied it uniformly.
- Recorded verbatim: **the audit was wrong once**. It proposed a `source=`
  uniqueness check for the catalog, and `source=` is a citation -- 30 of 109
  rows legitimately share one, so that check would have failed on the clean
  tree the day it landed. A finding survives refutation only in the direction
  it was checked.
