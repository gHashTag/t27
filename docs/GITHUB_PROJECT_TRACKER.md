# GitHub Project — “t27 Research & Publication Tracker”

**Goal:** A **public** project so researchers see backlog, in-progress, and publication-ready work without reading the whole monorepo.

## Create the project

1. Repository **Projects** → **New project** → choose **Table** or **Board** (Roadmap style).  
2. Name: `t27 Research & Publication Tracker`.  
3. Visibility: **Public**.  
4. Link the repository `gHashTag/t27`.

GitHub documentation: [Planning and tracking with Projects](https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects).

## Suggested custom fields

| Field | Type | Suggested values |
|-------|------|------------------|
| `Status` | Single select | `backlog`, `scoped`, `in progress`, `blocked`, `validation`, `publication-ready`, `published`, `archived` |
| `Priority` | Single select | `P0`, `P1`, `P2` |
| `Domain` | Single select | `core`, `numerics`, `fpga`, `ai`, `docs`, `publication`, `audit` |
| `Evidence` | Single select | `none`, `partial`, `validated`, `peer-visible` |
| `DOI` | Single select | `none`, `planned`, `reserved`, `published` |
| `Visibility` | Single select | `internal`, `public-facing`, `flagship` |
| `Target month` | Date or text | e.g. `2026-06` |

## Views

- **Board** by `Status` (kanban).  
- **Table** grouped by `Domain` or `Priority`.  
- **Roadmap** (if using timeline) by `Target month`.

## Automation (optional)

Use **workflow** or built-in rules to move items when PRs merge or labels change — add incrementally.

## Single source of truth

- **Specs / laws** → files in repo (`docs/`, `specs/`).  
- **Intent and schedule** → Issues + this Project + pinned dashboard issue.  
- Do not rely on chat or unlinked commits for “what we agreed.”

---

*An empty Project is worse than none — seed it from EPIC issues in `docs/ROADMAP.md`.*
