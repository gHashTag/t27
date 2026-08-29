# NOW -- A default is the literal wearing a fallback (2026-08-29)

## A default is the literal wearing a fallback (Refs #2804)

- an env var whose default is the old hardcoded path changes nothing: the literal stays, the guard still allowlists it, the reader still learns one machine's layout
- run_silicon declared five paths and checked three; a guard written as a list goes stale by addition
- one match arm printed both paths and the repair command while its sibling said missing on one side -- the standard was sitting right there
- tri skill check passed while my own unmerged PR already held 219-221; the check reads the tree, not my open PRs
