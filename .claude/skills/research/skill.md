# Research Skill: Academic Writing Workflow

Execute literature review and section drafting for academic papers using NotebookLM integration.

## Overview

This skill provides a structured workflow for academic research and paper generation, specifically designed for the Trinity S3AI project's NeurIPS OPT Workshop submission.

## Features

- **Literature Query**: Search for related papers and identify research gaps
- **Section Draft**: Generate paper sections (Introduction, Methods, Results, etc.)
- **Citation Support**: Extract and format academic citations
- **GoldenFloat Focus**: Pre-configured for ternary floating point research

## Usage

### Literature Review

Query NotebookLM for related academic papers on a specific topic:

```
/tri research lit-query \
  --topic "ternary floating point" \
  --keywords "ternary,balanced,phi" \
  --year-min 2018 \
  --year-max 2026 \
  --limit 20
```

**Parameters:**
- `--topic`: Main research topic (required)
- `--keywords`: Comma-separated keywords (optional)
- `--year-min`: Minimum publication year (default: 2018)
- `--year-max`: Maximum publication year (default: 2026)
- `--limit`: Maximum number of results (default: 20)
- `--venues`: Comma-separated venues (optional, e.g., "arXiv,NeurIPS")

**Output:**
- List of related papers with relevance scores
- Identified research gaps
- Methodology summaries
- Related work summaries

### Section Draft

Generate a paper section with optional citations:

```
/tri research section-draft \
  --section introduction \
  --prompt "Draft introduction: Explain GF mathematical foundation, hardware context, and opportunity" \
  --words 500
```

**Parameters:**
- `--section`: Section type (required)
  - Options: `abstract`, `introduction`, `related-work`, `methods`, `results`, `discussion`, `conclusion`
- `--prompt`: Description of content to generate (required)
- `--words`: Target word count (default: 500)
- `--citations`: Include citations flag (default: true)

**Section Types:**
- `abstract`: Brief summary (200-300 words)
- `introduction`: Background and motivation (500-800 words)
- `related-work`: Literature review (800-1200 words)
- `methods`: Technical approach (800-1500 words)
- `results`: Experimental findings (600-1000 words)
- `discussion`: Analysis and implications (500-800 words)
- `conclusion`: Summary and future work (300-500 words)

### GoldenFloat Quick Start

Pre-configured workflow for GoldenFloat paper:

```
/tri research gf-lit-quick
```

This runs a standard literature query for ternary floating point research with default parameters suitable for the NeurIPS submission.

### GoldenFloat Section Generation

Generate specific sections for the GoldenFloat paper:

```
/tri research gf-section --type introduction
/tri research gf-section --type methods
/tri research gf-section --type results
```

## Configuration

### Active Notebook

- **Default Notebook**: `t27-QUEEN-BRAIN`
- **Location**: Configured via `NotebookLMConfig` in `specs/memory/notebooklm.t27`

### Storage

- **Auth Tokens**: Stored at path specified in config
- **BibTeX**: Manual management (use existing `.bib` files in project)

### Timeouts

- **Default**: 30 seconds
- **Adjustable**: Via `--timeout` flag (future)

## Examples

### Example 1: Query for Ternary Arithmetic

```
/tri research lit-query \
  --topic "ternary arithmetic" \
  --keywords "balanced,ternary,phi" \
  --limit 10
```

### Example 2: Draft Related Work Section

```
/tri research section-draft \
  --section related-work \
  --prompt "Compare ternary floating point with binary IEEE 754, highlight GoldenFloat advantages" \
  --words 1000
```

### Example 3: Draft Methods Section

```
/tri research section-draft \
  --section methods \
  --prompt "Describe GF16/GF32 format specification, encoding/decoding algorithms, and hardware implementation considerations" \
  --words 1200
```

## Integration with t27 Build System

### Spec Files

- **Main Spec**: `specs/research/notebooklm.t27`
- **E2E Tests**: `specs/research/tests/e2e_literature_query.t27`, `specs/research/tests/e2e_section_draft.t27`

### Compilation

```bash
cd bootstrap && cargo build --release
./target/release/t27c test specs/research/notebooklm.t27
./target/release/t27c test specs/research/tests/e2e_literature_query.t27
./target/release/t27c test specs/research/tests/e2e_section_draft.t27
```

## Workflow for NeurIPS Submission

### Sprint 9 (April 8-15): Research & Drafting

1. Run literature query for ternary floating point
2. Identify key papers and research gaps
3. Draft Introduction and Related Work sections
4. Gather supporting citations

### Sprint 10 (April 16-22): Polish & Submit

1. Generate remaining sections
2. Manual polish via Overleaf
3. Final review and NeurIPS submission
4. Post-submission wrap-up

## Compliance

### Constitutional Compliance (L1-L7)

| Law | Compliance |
|------|------------|
| **L1 Traceability** | PRs link to Ring-XXX issues |
| **L2 GENERATION** | All logic in `.t27` specs |
| **L3 PURITY** | English identifiers only |
| **L4 TESTABILITY** | All specs contain test/invariant/bench |
| **L5 IDENTITY** | phi² + φ⁻² = 3 verified |
| **L6 CEILING** | GF16/GF32 from conformance JSON |
| **L7 UNITY** | No new `*.sh` on critical path |

## Notes

- Manual BibTeX management required (not automated)
- Use Overleaf for final paper editing and formatting
- NotebookLM authentication required (manual setup via cookies)
- Results are research aids, not final submission content

## Future Enhancements

- Automated citation extraction from literature queries
- Direct BibTeX export from query results
- Integration with arXiv API for up-to-date paper search
- LaTeX output formatting for direct Overleaf import
