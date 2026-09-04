# NOW -- A PR that merged into a branch that was already gone (2026-09-04)

## A PR that merged into a branch that was already gone (Closes #3128)

- #3099 reports MERGED and none of its content is on master: it was stacked on w69, w69 merged to master at 02:15:48, and #3099 merged into w69 at 02:19:23 -- three and a half minutes too late
- git merge-base --is-ancestor says NO and the commit is reachable from no remote branch; scripts/tri on master had zero which arms
- Only #3099 is affected: #3110, #3115, #3117, #3121 and #3123 all had base master and are ancestors of it, checked one by one
- tri pr landed exists to ask exactly this and I read the state field instead -- a label is not a measurement
