#!/usr/bin/env python3
# scripts/tri-pr-create.py
# Wrapper for GitHub PR creation with NotebookLM sync
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Create GitHub PR with automatic NotebookLM sync."""

import argparse
import sys
from pathlib import Path

# Add backend to path
sys.path.insert(0, str(Path(__file__).parent.parent / "contrib" / "backend"))

from github import GitHubClient
from notebooklm import client_new, notebook_create
from notebooklm.prs import pr_upload_notebooklm


def main():
    parser = argparse.ArgumentParser(
        description="Create GitHub PR with NotebookLM sync"
    )
    parser.add_argument("--title", required=True, help="PR title")
    parser.add_argument("--body", required=True, help="PR description")
    parser.add_argument("--issue", type=int, help="Link to issue number")
    parser.add_argument("--base", default="master", help="Base branch")
    parser.add_argument("--dry-run", action="store_true", help="Print only, no action")

    args = parser.parse_args()

    # Build PR body with issue reference
    body = args.body
    if args.issue:
        body = f"Closes #{args.issue}\n\n{body}"

    if args.dry_run:
        print(f"[DRY-RUN] Would create PR: {args.title}")
        print(f"[DRY-RUN] Base: {args.base}")
        print(f"[DRY-RUN] Linked issue: {args.issue}")
        print(f"[DRY-RUN] Body length: {len(body)} chars")
        return 0

    try:
        # Create PR via GitHub
        github_client = GitHubClient()
        pr = github_client.pr_create(
            title=args.title,
            body=body,
            base=args.base,
        )

        # Upload to NotebookLM
        notebooklm_client = client_new()
        notebook = notebook_create(notebooklm_client, "t27-GH-SSOT")

        source_id = pr_upload_notebooklm(
            notebooklm_client=notebooklm_client,
            github_pr_id=pr.id,
            title=pr.title,
            state=pr.state,
            merged=pr.merged_at is not None,
        )

        if source_id:
            print(f"✓ Created PR #{pr.id}: {pr.title}")
            print(f"✓ Uploaded to NotebookLM: source_id={source_id}")
            return 0
        else:
            print(f"✓ Created PR #{pr.id}")
            print(f"✗ Failed to upload to NotebookLM")
            return 1

    except Exception as e:
        print(f"✗ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
