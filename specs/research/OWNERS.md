# OWNERS — specs/research/

## Primary

**A-Architect** (research coordination) with **T-Queen** (orchestration).

## Subtree files

| File | Primary agent | Notes |
|------|----------------|-------|
| `notebooklm.t27` | **A-Architect** | Literature review and section generation spec |
| `literature.t27` | **A-Architect** | BibTeX management module |
| `tests/e2e_literature_query.t27` | **A-Architect** | E2E tests for literature queries |
| `tests/e2e_section_draft.t27` | **A-Architect** | E2E tests for section drafting |
| `tests/e2e_bibtex_export.t27` | **A-Architect** | E2E tests for BibTeX export |

## Purpose

This subtree defines the research workflow for academic paper submission, specifically:
- Literature query capabilities via NotebookLM integration
- Section generation for academic papers
- BibTeX management for citation handling
- E2E tests for research workflow validation

## Dependencies

- `memory/notebooklm.t27` — base NotebookLM client and types
- `.claude/skills/research/skill.md` — research workflow commands
