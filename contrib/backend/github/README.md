# GitHub Backend for t27 SSOT Integration

GitHub API integration for autonomous issue/PR/documentation management with two-way sync to NotebookLM.

## Modules

| Module | Description |
|---------|-------------|
| `auth.py` | GitHub authentication via GH_TOKEN |
| `issues.py` | Issue CRUD operations |
| `prs.py` | PR management (NEW) |
| `docs.py` | Documentation sync with NotebookLM |
| `comments.py` | Comment management |
| `client.py` | gh CLI wrapper (singleton) |
| `tri_integration.py` | Bridge to /tri skill |

## Usage

```python
from contrib.backend.github import GitHubClient, TriBridge

# Get authenticated client
client = GitHubClient.get_instance()

# Or with explicit token
from contrib.backend.github import GitHubAuth
client = GitHubClient(auth_token=GitHubAuth.token_load())

# Use through bridge
bridge = TriBridge()
issue_id = bridge.create_issue_from_notebook(notebooklm_id="abc123")
source_id = bridge.sync_github_to_notebooklm(issue_id=128)
```

## Authentication

Uses `GH_TOKEN` environment variable. Token must start with `ghp_` or `github_pat_`.

```bash
export GH_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
```

## Integration with NotebookLM

Two-way sync between GitHub entities and NotebookLM sources:
- GitHub Issue ↔ NotebookLM Source (bidirectional)
- GitHub PR ↔ NotebookLM Note (bidirectional)
- Documentation ↔ NotebookLM (upload)

## See Also

- `/tri` skill — PHI LOOP workflow
- `/contrib/backend/notebooklm/` — NotebookLM backend
