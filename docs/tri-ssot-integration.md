# Tri SSOT Integration

GitHub ↔ NotebookLM Single Source of Truth (SSOT) integration for t27.

## Overview

This integration provides bidirectional synchronization between:
- **GitHub Issues** ↔ NotebookLM sources
- **GitHub Pull Requests** ↔ NotebookLM sources
- **GitHub Documentation** ↔ NotebookLM sources

All sync operations are orchestrated through the `UnifiedSyncOrchestrator` and
exposed via the `/tri` skill commands.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Tri CLI (/tri)                         │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ├──► tri-issue-create.py
                    ├──► tri-sync.py
                    ├──► tri-search.py
                    ├──► tri-doc-sync.py
                    └──► tri-pr-create.py
                    │
        ┌───────────┴───────────┐
        │                       │
┌───────▼────────┐    ┌────────▼──────────┐
│  GitHub Client │    │ NotebookLM Client │
│  (gh CLI)      │    │ (notebooklm-py)   │
└───────┬────────┘    └────────┬──────────┘
        │                      │
        └──────────┬───────────┘
                   │
        ┌──────────▼──────────┐
        │ UnifiedSyncOrchestrator│
        │ (sync.py)            │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────┐
        │  Trinity State      │
        │  .trinity/state/    │
        └─────────────────────┘
```

## Installation

### Prerequisites

1. **GitHub CLI (gh):**
   ```bash
   brew install gh  # macOS
   ```
   Or: https://cli.github.com/

2. **GitHub Authentication:**
   ```bash
   gh auth login
   ```

3. **Environment Variables:**
   ```bash
   export GITHUB_TOKEN=ghp_xxx  # Optional, uses gh auth if not set
   export NOTEBOOKLM_COOKIE_PATH=/path/to/cookies.json
   ```

### Python Dependencies

The integration is part of `contrib/backend/`. No additional installation required
if using t27's bootstrap environment.

## Usage

### Via /tri Skill

```bash
# Sync all GitHub entities to NotebookLM
/tri sync

# Sync GitHub Issues only
/tri sync issues

# Sync GitHub PRs only
/tri sync prs

# Search across GitHub + NotebookLM
/tri search "query"

# Create a GitHub issue
/tri issue create "Title" "Description"

# Sync documentation
/tri doc sync

# Create a GitHub PR
/tri pr create "branch" "title" "body"
```

### Via Wrapper Scripts

```bash
# Full sync
./scripts/tri-sync.py

# Issues sync
./scripts/tri-issue-create.py "Title" "Description"

# Search
./scripts/tri-search.py "query"

# Documentation sync
./scripts/tri-doc-sync.py

# PR creation
./scripts/tri-pr-create.py "branch" "title" "body"
```

### Direct Python Usage

```python
from contrib.backend.github import GitHubClient, GitHubIssues, GitHubPRs, GitHubDocs
from contrib.backend.notebooklm import UnifiedSyncOrchestrator

# Create clients
github_client = GitHubClient()
issues = GitHubIssues(github_client)
prs = GitHubPRs(github_client)
docs = GitHubDocs(github_client)

# Create orchestrator (with NotebookLM integration)
orchestrator = UnifiedSyncOrchestrator(
    github_issues=issues,
    github_prs=prs,
    github_docs=docs,
    notebooklm_issue=notebooklm_issue_sync_fn,
    notebooklm_pr=notebooklm_pr_sync_fn,
    notebooklm_doc=notebooklm_doc_sync_fn,
)

# Run sync
result = orchestrator.full_sync()

print(f"Synced {result.items_synced} items in {result.duration_ms}ms")
print(f"Success: {result.success}, Errors: {len(result.errors)}")
```

## Configuration

### State File

Sync state is maintained in `.trinity/state/github-bridge.json`:

```json
{
  "last_sync": "2026-04-08T12:00:00Z",
  "synced_issues": [1, 2, 3],
  "synced_prs": [4, 5],
  "synced_docs": ["docs/intro.md"],
  "version": "1.0.0"
}
```

### Sync Limits

Default sync limits to prevent overwhelming GitHub/NotebookLM:

- **Issues:** 5 per sync (open state)
- **PRs:** 5 per sync (open state)
- **Docs:** All files in `docs/` directory

## Testing

### Run Unit Tests

```bash
# Run all sync tests
pytest contrib/backend/notebooklm/tests/test_sync.py -v

# Run specific test
pytest contrib/backend/notebooklm/tests/test_sync.py::TestUnifiedSyncOrchestrator::test_sync_issues -v
```

### Run Verification

```bash
# Verify full integration
./scripts/verify-ssot-integration.sh
```

## Data Flow

### Issue Sync

```
GitHub Issue (API)
    ↓
GitHubIssues.issue_list()
    ↓
UnifiedSyncOrchestrator.sync_issues()
    ↓
NotebookLM source_upload_text()
    ↓
NotebookLM Source
    ↓
State update (.trinity/state/github-bridge.json)
```

### Search Flow

```
Query → UnifiedSearchOrchestrator.search()
    ├─► GitHub Issues API
    ├─► GitHub PRs API
    └─► NotebookLM Query API
    ↓
Combine results by relevance
    ↓
Return sorted results
```

## Error Handling

All sync operations return a `SyncResult`:

```python
@dataclass
class SyncResult:
    success: bool           # True if no errors
    items_synced: int       # Number of items successfully synced
    errors: List[str]       # List of error messages
    duration_ms: int        # Duration in milliseconds
```

## Troubleshooting

### "gh CLI not found"

Install GitHub CLI: https://cli.github.com/

### "Authentication required"

```bash
gh auth login
# or
export GITHUB_TOKEN=ghp_xxx
```

### "NotebookLM cookie invalid"

```bash
# Re-authenticate with cookies
python3 -c "
from contrib.backend.notebooklm import authenticate_with_cookies
authenticate_with_cookies()
"
```

### Import errors

```bash
# Ensure contrib/backend is in Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)/contrib/backend"
```

## Contributing

When modifying this integration:

1. Update tests in `contrib/backend/notebooklm/tests/test_sync.py`
2. Run verification: `./scripts/verify-ssot-integration.sh`
3. Update this documentation
4. Ensure backward compatibility with existing state files

## Links

- [AGENTS.md](../AGENTS.md) - Agent architecture
- [SOUL.md](../SOUL.md) - Project philosophy
- [T27-CONSTITUTION.md](./T27-CONSTITUTION.md) - Invariant laws

---

phi² + 1/phi² = 3 | TRINITY
