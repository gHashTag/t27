# NOW -- Three submodule registrations were deleted by a commit about a skill document (2026-08-28)

## Three submodule registrations were deleted by a commit about a skill document (Refs #2754)

- chips/{phi,euler,gamma} were registered on 2026-05-23 and dropped in b79702ee1, three deletions inside 174 changed files
- submodules: true in cli-tri.yml was already set and fetched nothing, because there were zero gitlinks to fetch
- tri rtl check died with a bare os error 2; top_from_info runs before the one function that named its file
- restored to the exact SHAs held before the deletion, each verified to still exist upstream
