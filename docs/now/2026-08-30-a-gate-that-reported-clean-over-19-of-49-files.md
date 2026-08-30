# NOW -- A gate that reported CLEAN over 19 of 49 files (2026-08-30)

## A gate that reported CLEAN over 19 of 49 files (Closes #2957)

- check_pr_branch_filters.py printed the two list sizes beside the file count and never subtracted them; 30 workflows were read by nothing and the last line still said CLEAN
- two of the thirty carried pull_request branches:[master] -- the defect it exists to detect -- and one is corpus-ratchet, absent from the check list of any stacked PR
- third bucket printed with its sum, a down-only ceiling of 27 so the next added workflow cannot land unread, and the clean line now states what remains unread
- tri harness scratch --gate is wired now that #2949 and #2955 have both landed; no paths filter and a push trigger on master, because emit-bitexact has no master baseline for exactly the opposite choice
