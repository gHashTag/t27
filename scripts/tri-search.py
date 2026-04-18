#!/usr/bin/env python3
# scripts/tri-search.py
# Wrapper for unified GitHub + NotebookLM search
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unified search across GitHub Issues, PRs, Documentation and NotebookLM."""

import argparse
import sys
from pathlib import Path

# Add backend to path
sys.path.insert(0, str(Path(__file__).parent.parent / "contrib" / "backend"))

from github import GitHubClient
from notebooklm import client_new, notebook_query


def main():
    parser = argparse.ArgumentParser(
        description="Unified search GitHub + NotebookLM"
    )
    parser.add_argument("query", help="Search query")
    parser.add_argument("--types", default="issues,prs,docs,notebooklm",
                       help="Comma-separated types: issues,prs,docs,notebooklm")
    parser.add_argument("--limit", type=int, default=10, help="Results per type")
    parser.add_argument("--json", action="store_true", help="Output as JSON")

    args = parser.parse_args()

    types = [t.strip() for t in args.types.split(",")]

    results = {
        "query": args.query,
        "github_issues": [],
        "github_prs": [],
        "docs": [],
        "notebooklm_notes": []
    }

    # Search GitHub Issues
    if "issues" in types:
        try:
            github_client = GitHubClient()
            issues = github_client.issue_find_similar(
                query=args.query,
                threshold=0.5,
            )
            results["github_issues"] = [
                {
                    "id": issue.id,
                    "title": issue.title,
                    "state": issue.state,
                    "url": issue.url
                }
                for issue in issues[:args.limit]
            ]
        except Exception as e:
            print(f"GitHub Issues search error: {e}", file=sys.stderr)

    # Search GitHub PRs
    if "prs" in types:
        try:
            github_client = GitHubClient()
            prs = github_client.pr_find_similar(
                query=args.query,
                threshold=0.5,
            )
            results["github_prs"] = [
                {
                    "id": pr.id,
                    "title": pr.title,
                    "state": pr.state,
                    "merged": pr.merged_at is not None,
                    "url": pr.url
                }
                for pr in prs[:args.limit]
            ]
        except Exception as e:
            print(f"GitHub PRs search error: {e}", file=sys.stderr)

    # Search NotebookLM
    if "notebooklm" in types:
        try:
            notebooklm_client = client_new()
            result = notebook_query(notebooklm_client, args.query)

            if result.get("answer"):
                results["notebooklm_notes"] = [
                    {
                        "content": line[:200],
                        "source": result.get("sources", ["NotebookLM"])
                    }
                    for line in result["answer"].split("\n")[:args.limit]
                    if line.strip()
                ]
        except Exception as e:
            print(f"NotebookLM search error: {e}", file=sys.stderr)

    # Output results
    if args.json:
        import json
        print(json.dumps(results, indent=2))
    else:
        print(f"🔍 Search: {args.query}")
        print()

        if results["github_issues"]:
            print(f"📌 GitHub Issues ({len(results['github_issues'])})")
            for i, issue in enumerate(results["github_issues"][:5], 1):
                print(f"  {i}. #{issue['id']} {issue['title']} [{issue['state']}]")
            print()

        if results["github_prs"]:
            print(f"🔀 GitHub PRs ({len(results['github_prs'])})")
            for i, pr in enumerate(results["github_prs"][:5], 1):
                merged = "✓" if pr["merged"] else "○"
                print(f"  {i}. #{pr['id']} {pr['title']} {merged} [{pr['state']}]")
            print()

        if results["notebooklm_notes"]:
            print(f"📓 NotebookLM ({len(results['notebooklm_notes'])})")
            for i, note in enumerate(results["notebooklm_notes"][:3], 1):
                print(f"  {i}. {note['content']}")
            print()

        total = len(results["github_issues"]) + len(results["github_prs"]) + len(results["notebooklm_notes"])
        print(f"Total: {total} results")

    return 0


if __name__ == "__main__":
    sys.exit(main())
