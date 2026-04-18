#!/usr/bin/env python3
# scripts/tri-sync.py
# Wrapper for unified GitHub ↔ NotebookLM sync
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unified sync orchestrator for GitHub ↔ NotebookLM SSOT."""

import argparse
import json
import sys
from pathlib import Path
from datetime import datetime

# Add backend to path
sys.path.insert(0, str(Path(__file__).parent.parent / "contrib" / "backend"))

from github import GitHubClient
from notebooklm import client_new, client_authenticate, notebook_create
from notebooklm.sync import UnifiedSyncOrchestrator
from notebooklm.issues import issue_upload_notebooklm
from notebooklm.prs import pr_upload_notebooklm
from notebooklm.docs import doc_upload_notebooklm


def main():
    parser = argparse.ArgumentParser(
        description="Unified sync GitHub ↔ NotebookLM"
    )
    parser.add_argument("--scope", default="all", choices=["all", "issues", "prs", "docs"],
                       help="Sync scope")
    parser.add_argument("--status", action="store_true", help="Show sync status only")
    parser.add_argument("--dry-run", action="store_true", help="Print only, no action")

    args = parser.parse_args()

    # State file
    state_file = Path(__file__).parent.parent / ".trinity" / "state" / "github-bridge.json"

    if args.status:
        if state_file.exists():
            with open(state_file) as f:
                state = json.load(f)
            print(f"GitHub ↔ NotebookLM Sync Status")
            print(f"Last sync: {state.get('last_sync_at', 'Never')}")
            print(f"  Issues: {state['sync_stats']['issues']['synced']} synced, {state['sync_stats']['issues']['failed']} failed")
            print(f"  PRs: {state['sync_stats']['prs']['synced']} synced, {state['sync_stats']['prs']['failed']} failed")
            print(f"  Docs: {state['sync_stats']['docs']['synced']} synced, {state['sync_stats']['docs']['failed']} failed")
        else:
            print("No sync state found. Run --all to initialize.")
        return 0

    if args.dry_run:
        print(f"[DRY-RUN] Would sync scope: {args.scope}")
        return 0

    # Initialize clients
    try:
        github_client = GitHubClient()
        notebooklm_client = client_new()
        if not client_is_authenticated(notebooklm_client):
            notebooklm_client = client_authenticate(notebooklm_client)

        # Get or create notebook
        notebook = notebook_create(notebooklm_client, "t27-GH-SSOT")

        # Create orchestrator
        orchestrator = UnifiedSyncOrchestrator(
            github_issues=github_client,
            github_prs=github_client,
            github_docs=github_client,
            notebooklm_issue=lambda **kwargs: issue_upload_notebooklm(notebooklm_client, **kwargs),
            notebooklm_pr=lambda **kwargs: pr_upload_notebooklm(notebooklm_client, **kwargs),
            notebooklm_doc=lambda **kwargs: doc_upload_notebooklm(notebooklm_client, **kwargs),
        )

        # Run sync
        result = orchestrator.full_sync(scope=args.scope)

        # Update state
        if state_file.exists():
            with open(state_file) as f:
                state = json.load(f)
        else:
            state = {
                "version": "1.0.0",
                "last_sync_at": None,
                "sync_stats": {
                    "issues": {"synced": 0, "failed": 0},
                    "prs": {"synced": 0, "failed": 0},
                    "docs": {"synced": 0, "failed": 0},
                },
                "issues": {},
                "prs": {},
                "docs": {}
            }

        state["last_sync_at"] = datetime.now().isoformat()
        state["sync_stats"][args.scope if args.scope != "all" else "issues"]["synced"] += result.items_synced

        with open(state_file, "w") as f:
            json.dump(state, f, indent=2)

        if result.success:
            print(f"✓ Sync complete: {result.items_synced} items synced")
            return 0
        else:
            print(f"✗ Sync errors: {len(result.errors)}")
            for error in result.errors[:3]:
                print(f"  - {error}")
            return 1

    except Exception as e:
        print(f"✗ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
