# Kaggle Dataset Upload Learnings

## Task
Upload TTM v5 and TSCP v5 datasets to Kaggle for AGI Hackathon (deadline 2026-04-16).

## Approach Used
1. BrowserOS MCP was NOT configured (manual setup required by user)
2. Used existing Kaggle CLI (`kaggle datasets version`) instead
3. Updated `external/kaggle/scripts/upload_mc_datasets.py` to include v5 entries

## Script Modifications
Added two new entries to `MC_DATASETS` dictionary:
- `tmp_v5`: Points to `data/ttm_mc_v5.csv`, 600 unique MC questions
- `tscp_v5`: Points to `data/tscp_mc_v5.csv`, 500 unique MC questions

## Verification Method
After upload, verified by:
```bash
kaggle datasets download playra/trinity-cognitive-probes-tmp-mc --unzip --force
kaggle datasets download playra/trinity-cognitive-probes-tscp-mc --unzip --force
```
Both downloads produced files with expected sizes (~302KB), confirming successful upload.

## Key Learnings
1. Kaggle CLI `datasets list --search` may have delays or limitations
2. Direct download verification is more reliable than search API
3. Version uploads to existing datasets (same slug) replace the current version
4. Kaggle CLI 2.0.0 works but has deprecation warnings (upgrade to 2.0.1)

## Future Recommendations
1. Consider upgrading Kaggle CLI to 2.0.1
2. BrowserOS MCP setup would enable more automated upload workflows
3. Add version history tracking to upload script for audit trail

## Success
- TTM v5: 600 questions uploaded successfully
- TSCP v5: 500 questions uploaded successfully
- Both datasets now live on Kaggle with CC0-1.0 license
