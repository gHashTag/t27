#!/usr/bin/env python3.10
"""NotebookLM populate module.

Creates notebooks from GitHub issues and populates them with issue content.

phi^2 + 1/phi^2 = 3 | TRINITY
"""

import argparse
import asyncio
import logging
import sys
import re
from datetime import datetime

try:
    from notebooklm import NotebookLMClient
except ImportError:
    print("Error: notebooklm-py not installed. Run: pip install notebooklm-py")
    sys.exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
logger = logging.getLogger(__name__)


async def create_notebook_for_issue(
    client: NotebookLMClient,
    issue_number: int,
    title: str,
    body: str,
    labels: list[str],
) -> str:
    """Create a notebook for a GitHub issue.

    Args:
        client: NotebookLM client.
        issue_number: GitHub issue number.
        title: Issue title.
        body: Issue body.
        labels: Issue labels.

    Returns:
        Notebook ID.
    """
    # Prepare notebook content
    notebook_title = f"#{issue_number}: {title}"
    notebook_description = f"GitHub Issue #{issue_number}\n\nLabels: {', '.join(labels) if labels else 'None'}"

    logger.info(f"Creating notebook: {notebook_title}")

    # Create notebook
    notebook = await client.notebooks.create(
        title=notebook_title,
        description=notebook_description,
    )

    logger.info(f"Created notebook: {notebook.id}")

    # Add issue content as a text source
    issue_content = f"""# Issue #{issue_number}: {title}

**Labels:** {', '.join(labels) if labels else 'None'}

## Description

{body}

---
*This notebook was automatically created from GitHub issue #{issue_number}*
"""

    await client.sources.add_text(notebook.id, f"issue_{issue_number}.md", issue_content)
    logger.info(f"Added issue content as source")

    return notebook.id


async def find_notebook_by_issue(
    client: NotebookLMClient,
    issue_number: int,
) -> str | None:
    """Find an existing notebook by issue number.

    Args:
        client: NotebookLM client.
        issue_number: GitHub issue number.

    Returns:
        Notebook ID if found, None otherwise.
    """
    notebooks = await client.notebooks.list()
    pattern = re.compile(rf"#?{issue_number}\b", re.IGNORECASE)

    for nb in notebooks:
        if pattern.search(nb.title or ""):
            return nb.id
    return None


async def populate_issue(
    issue_number: int,
    repo: str = "gHashTag/t27",
    force: bool = False,
) -> dict:
    """Populate a single issue into NotebookLM.

    Args:
        issue_number: GitHub issue number.
        repo: Repository name.
        force: Force recreation even if notebook exists.

    Returns:
        Result dictionary.
    """
    async with await NotebookLMClient.from_storage() as client:
        # Check if notebook already exists
        existing_id = await find_notebook_by_issue(client, issue_number)
        if existing_id and not force:
            logger.info(f"Notebook for issue #{issue_number} already exists: {existing_id}")
            return {
                "issue": issue_number,
                "notebook_id": existing_id,
                "created": False,
                "reason": "already_exists",
            }

        # Mock issue data - in production, fetch from GitHub API
        # For now, create with placeholder content
        title = f"Issue #{issue_number}"
        body = f"This is placeholder content for issue #{issue_number}. In production, this would be fetched from the GitHub API for {repo}."
        labels = []

        notebook_id = await create_notebook_for_issue(
            client, issue_number, title, body, labels
        )

        return {
            "issue": issue_number,
            "notebook_id": notebook_id,
            "created": True,
        }


async def populate_all(
    repo: str = "gHashTag/t27",
    force: bool = False,
) -> dict:
    """Populate all issues from a repository.

    Args:
        repo: Repository name.
        force: Force recreation of existing notebooks.

    Returns:
        Summary dictionary.
    """
    logger.info(f"Populating all issues from {repo}")

    # Mock issue list - in production, fetch from GitHub API
    # For now, return empty since we need to fetch from GitHub
    logger.warning("populate_all requires GitHub API access")
    logger.info("Use --issue <number> to populate a specific issue")

    return {
        "total": 0,
        "created": 0,
        "skipped": 0,
        "results": [],
    }


async def main():
    parser = argparse.ArgumentParser(
        description="Populate NotebookLM with GitHub issues"
    )
    parser.add_argument(
        "--issue",
        type=int,
        help="Specific issue number to populate"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Populate all issues"
    )
    parser.add_argument(
        "--repo",
        default="gHashTag/t27",
        help="Repository name (default: gHashTag/t27)"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Force recreation of existing notebooks"
    )

    args = parser.parse_args()

    if args.issue:
        result = await populate_issue(args.issue, args.repo, args.force)
        print(f"\n{'='*60}")
        print(f"Issue #{result['issue']}")
        print(f"{'='*60}")
        print(f"Notebook ID: {result['notebook_id']}")
        print(f"Created: {result['created']}")
        if not result['created']:
            print(f"Reason: {result['reason']}")
        print(f"{'='*60}\n")
    elif args.all:
        summary = await populate_all(args.repo, args.force)
        print(f"\n{'='*60}")
        print(f"Population Summary")
        print(f"{'='*60}")
        print(f"Total issues: {summary['total']}")
        print(f"Created: {summary['created']}")
        print(f"Skipped: {summary['skipped']}")
        print(f"{'='*60}\n")
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
