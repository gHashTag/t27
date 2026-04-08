#!/usr/bin/env python3
# contrib/backend/notebooklm/populate.py
# Populate NotebookLM notebooks with GitHub issue content
# phi^2 + 1/phi^2 = 3 | TRINITY

import asyncio
import sys
import json
import subprocess
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))

try:
    from notebooklm import NotebookLMClient
    from .sources import source_upload_text
except ImportError as e:
    print(f"Error: {e}", file=sys.stderr)
    print("Please install: pip install notebooklm-py", file=sys.stderr)
    sys.exit(1)


async def populate_all_issues(repo: str = "gHashTag/t27", limit: int = 100):
    """Populate all notebooks with issue content."""

    # Fetch open issues
    print(f"Fetching open issues from {repo}...")
    result = subprocess.run(
        ["gh", "issue", "list", "--repo", repo, "--state", "open",
         "--json", "number,title,body,comments,labels,assignees,createdAt,updatedAt",
         "--limit", str(limit)],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error fetching issues: {result.stderr}", file=sys.stderr)
        return

    issues = json.loads(result.stdout)
    print(f"Found {len(issues)} open issues")

    # Get client
    async with await NotebookLMClient.from_storage() as client:
        notebooks = await client.notebooks.list()

        # Build title to notebook ID mapping
        notebook_map = {}
        for nb in notebooks:
            notebook_map[nb.title] = nb.id

        # Process each issue
        success = 0
        failed = 0

        for issue in issues:
            num = issue['number']
            title = issue['title']
            body = issue.get('body', '') or ''
            comments = issue.get('comments', []) or []
            labels = ', '.join([l['name'] for l in issue.get('labels', [])])
            created = issue.get('createdAt', '')
            updated = issue.get('updatedAt', '')

            # Build full content
            content_lines = [
                f"# Issue #{num}: {title}",
                "",
                "## Metadata",
                f"- Labels: {labels}",
                f"- Created: {created}",
                f"- Updated: {updated}",
                "",
                "## Description",
                body if body else "*No description*",
                "",
                "## Discussion History",
                ""
            ]

            # Add comments
            for i, c in enumerate(comments, 1):
                author = c.get('author', {}).get('login', 'unknown')
                date = c.get('createdAt', '')
                cbody = c.get('body', '')
                content_lines.extend([
                    f"### Comment {i+1} by @{author} ({date})",
                    cbody if cbody else "*No comment*",
                    ""
                ])

            # Get related commits
            commits = subprocess.run(
                ["git", "log", "--oneline", "--all", f"--grep=#{num}",
                 "--max-count=20"],
                capture_output=True,
                text=True
            ).stdout.strip()

            if commits:
                content_lines.extend([
                    "",
                    "## Related Commits",
                    commits,
                    ""
                ])

            # Get related PRs
            prs = subprocess.run(
                ["gh", "pr", "list", "--search", f"#{num}",
                 "--json", "number,title,files", "--limit", "5"],
                capture_output=True,
                text=True
            ).stdout.strip()

            if prs:
                content_lines.extend([
                    "",
                    "## Related PRs",
                    ""
                ])
                for pr in json.loads(prs):
                    pr_num = pr.get('number', '')
                    pr_title = pr.get('title', '')
                    pr_files = pr.get('files', [])
                    content_lines.extend([
                        f"PR #{pr_num}: {pr_title}",
                        f"Files: {', '.join(pr_files)}",
                        ""
                    ])

            content = '\n'.join(content_lines)

            # Find the notebook for this issue
            notebook_title = f"Issue #{num}: {title}"
            notebook_id = notebook_map.get(notebook_title)

            if not notebook_id:
                print(f"❌ Notebook not found for Issue #{num}: {notebook_title}", file=sys.stderr)
                failed += 1
                continue

            # Write to temp file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False,
                                          prefix=f'issue_{num}_') as f:
                f.write(content)
                tmpfile = f.name

            # Upload as source to notebook
            print(f"Uploading content for Issue #{num}: {title} ({len(content)} chars)")

            try:
                # Convert temp file path to Path
                tmp_path = Path(tmpfile)
                source_upload = source_upload_file(client, notebook_id, tmp_path)

                if source_upload:
                    success += 1
                    print(f"  ✅ #{num} populated ({len(content)} chars uploaded)")
                else:
                    print(f"  ❌ #{num} failed to upload", file=sys.stderr)
                    failed += 1

            except Exception as e:
                print(f"  ❌ #{num} error: {e}", file=sys.stderr)
                failed += 1
            finally:
                # Clean up temp file
                try:
                    os.unlink(tmpfile)
                except:
                    pass

        print()
        print(f"Done: {success} succeeded, {failed} failed")


async def populate_single_issue(issue_number: int, repo: str = "gHashTag/t27"):
    """Populate a single notebook with issue content."""

    # Fetch single issue
    print(f"Fetching issue #{issue_number} from {repo}...")
    result = subprocess.run(
        ["gh", "issue", "view", str(issue_number), "--repo", repo,
         "--json", "number,title,body,comments,labels,assignees,createdAt,updatedAt"],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error fetching issue: {result.stderr}", file=sys.stderr)
        return

    issue = json.loads(result.stdout)
    num = issue['number']
    title = issue['title']
    body = issue.get('body', '') or ''
    comments = issue.get('comments', []) or []
    labels = ', '.join([l['name'] for l in issue.get('labels', [])])

    # Get client
    async with await NotebookLMClient.from_storage() as client:
        notebooks = await client.notebooks.list()

        # Build title to notebook ID mapping
        notebook_map = {}
        for nb in notebooks:
            notebook_map[nb.title] = nb.id

        # Find the notebook
        notebook_title = f"Issue #{num}: {title}"
        notebook_id = notebook_map.get(notebook_title)

        if not notebook_id:
            print(f"❌ Notebook not found for Issue #{num}: {notebook_title}", file=sys.stderr)
            return

        # Build full content
        content_lines = [
            f"# Issue #{num}: {title}",
            "",
            "## Metadata",
            f"- Labels: {labels}",
            f"- Created: {issue.get('createdAt', '')}",
            f"- Updated: {issue.get('updatedAt', '')}",
            "",
            "## Description",
            body if body else "*No description*",
            "",
            "## Discussion History",
            ""
        ]

        # Add comments
        for i, c in enumerate(comments, 1):
            author = c.get('author', {}).get('login', 'unknown')
            date = c.get('createdAt', '')
            cbody = c.get('body', '')
            content_lines.extend([
                f"### Comment {i+1} by @{author} ({date})",
                cbody if cbody else "*No comment*",
                ""
            ])

        content = '\n'.join(content_lines)

        # Upload as source to notebook
        print(f"Uploading content for Issue #{num}: {title} ({len(content)} chars)")

        try:
            # Create temp file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False,
                                          prefix=f'issue_{num}_') as f:
                f.write(content)
                tmp_path = Path(f.name)

            source_upload = source_upload_file(client, notebook_id, tmp_path)

            if source_upload:
                print(f"  ✅ #{num} populated ({len(content)} chars uploaded)")
            else:
                print(f"  ❌ #{num} failed to upload", file=sys.stderr)

        except Exception as e:
            print(f"  ❌ #{num} error: {e}", file=sys.stderr)
        finally:
            try:
                os.unlink(tmp_path)
            except:
                pass


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Populate NotebookLM notebooks with GitHub issue content"
    )
    parser.add_argument("--all", action="store_true", help="Populate all open issues")
    parser.add_argument("--issue", type=int, help="Populate specific issue number")
    parser.add_argument("--repo", default="gHashTag/t27", help="GitHub repository (default: gHashTag/t27)")
    parser.add_argument("--limit", type=int, default=100, help="Max issues to process (default: 100)")

    args = parser.parse_args()

    if args.all:
        asyncio.run(populate_all_issues(repo=args.repo, limit=args.limit))
    elif args.issue:
        asyncio.run(populate_single_issue(args.issue, repo=args.repo))
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
