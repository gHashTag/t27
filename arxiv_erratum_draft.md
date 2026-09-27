# Erratum for arXiv:2606.09686

## Erratum Notice

**Paper**: arXiv:2606.09686  
**Original Figure**: 84 formats in the numeric format catalogue  
**Correct Figure**: 109 formats in the numeric format catalogue  
**Valid At Commit**: b92872507f6c7619acce43e5ae262b1dc9c4cbf2

## Erratum Text

> In Section X (Introduction), the paper states that the numeric format catalogue contains 84 formats. This figure is outdated. As of commit b92872507f6c7619acce43e5ae262b1dc9c4cbf2, the catalogue contains 109 distinct formats, as confirmed by both:
> 
> 1. A distinct-name count of the source-of-truth text (`specs/numeric/formats_catalog.t27`)
> 2. The shipped integrity checker (`tools/check_catalog_integrity.py`)
> 
> The integrity checker output at this commit is:
> ```
> OK: 109 catalog rows, every source= resolves, 17 GF + 9 GFT + 8 BNF + 9 TNF + gfternary,
> four families present and distinct
> ```
> 
> The figure of 84 reflects the state of the catalogue at an earlier commit and should be disregarded. All future citations should reference the canonical count of 109 at commit b92872507f6c7619acce43e5ae262b1dc9c4cbf2.

## Context

The discrepancy arose because the catalogue has been expanded since the preprint was published. The growth from 84 to 109 formats includes:
- Addition of new format families
- Expansion of existing families (e.g., GF family from 17 to 25 formats)
- Former name preservation for historical continuity (9 aliases)

## Verification

The canonical count of 109 is established by two independent rules that arrive at the same number:
- Count of distinct `name = "..."` entries in the source-of-truth text
- Count of distinct `id=` rows recognized by the integrity checker

This dual verification makes 109 the authoritative figure rather than merely the most recent count.

## Machine Inventory

```json
{
  "repo": "gHashTag/t27",
  "commit_sha": "b92872507f6c7619acce43e5ae262b1dc9c4cbf2",
  "canonical": {
    "value": 109,
    "basis": "the integrity checker agrees with a distinct-name count of the SSOT; two independent rules, same number",
    "valid_at_commit": "b92872507f6c7619acce43e5ae262b1dc9c4cbf2"
  }
}
```

## Recommendation

1. Update the arXiv preprint with this erratum
2. Replace any hardcoded figures in other documents with references to the integrity checker
3. Use the machine inventory generator for future citations
4. Always include the commit hash when citing the catalogue size