# contrib/backend/notebooklm/session.py
# Session context extraction for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Session context extraction from .trinity state files."""

import json
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Dict, Any, Optional
from datetime import datetime


@dataclass
class SessionContext:
    """Session context extracted from .trinity state.

    Attributes:
        session_id: Unique session identifier
        repo_root: Repository root path
        branch: Current git branch
        skill_id: Active skill ID
        issue_number: Associated issue number
        start_time: Session start timestamp
        tasks_completed: Number of completed tasks
        files_modified: Number of files modified
        git_status: Git status summary
    """

    session_id: str
    repo_root: str
    branch: str
    skill_id: str
    issue_number: int
    start_time: str
    tasks_completed: int
    files_modified: int
    git_status: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


def _read_json_file(path: Path) -> Optional[Dict[str, Any]]:
    """Read and parse a JSON file.

    Args:
        path: Path to JSON file

    Returns:
        Parsed dict or None if file doesn't exist or is invalid
    """
    if not path.exists():
        return None

    try:
        with open(path, "r") as f:
            return json.load(f)
    except (json.JSONDecodeError, IOError):
        return None


def _read_jsonl_file(path: Path) -> list:
    """Read and parse a JSONL file.

    Args:
        path: Path to JSONL file

    Returns:
        List of parsed dicts
    """
    if not path.exists():
        return []

    try:
        with open(path, "r") as f:
            return [json.loads(line) for line in f if line.strip()]
    except (json.JSONDecodeError, IOError):
        return []


def session_extract_from_trinity(repo_root: str) -> Optional[Dict[str, Any]]:
    """Extract session context from .trinity state files.

    Reads:
        - .trinity/state/active-skill.json
        - .trinity/state/issue-binding.json
        - .trinity/events/akashic-log.jsonl

    Args:
        repo_root: Repository root path

    Returns:
        SessionContext dict or None if extraction fails
    """
    trinity_path = Path(repo_root) / ".trinity"

    if not trinity_path.exists():
        print(f"Error: .trinity directory not found in {repo_root}")
        return None

    # Read active skill
    state_path = trinity_path / "state"
    active_skill = _read_json_file(state_path / "active-skill.json") or {}
    issue_binding = _read_json_file(state_path / "issue-binding.json") or {}

    # Read akashic log for session info
    events = _read_jsonl_file(trinity_path / "events" / "akashic-log.jsonl")

    # Extract session ID from latest event
    session_id = "unknown"
    if events:
        latest_event = events[-1]
        session_id = latest_event.get("trace_id", latest_event.get("agent_id", "unknown"))

    # Extract issue number
    issue_number = 0
    if issue_binding:
        issue_number = issue_binding.get("issue_number", 0)

    # Build session context
    context = SessionContext(
        session_id=session_id,
        repo_root=repo_root,
        branch=active_skill.get("branch", "unknown"),
        skill_id=active_skill.get("skill_id", "unknown"),
        issue_number=issue_number,
        start_time=datetime.now().isoformat(),
        tasks_completed=len(events),  # Count events as tasks
        files_modified=active_skill.get("files_modified", 0),
        git_status=active_skill.get("git_status", "unknown"),
    )

    return context.to_dict()


def session_extract_from_current_dir() -> Optional[Dict[str, Any]]:
    """Extract session context from current working directory.

    Returns:
        SessionContext dict or None if not in a t27 repo
    """
    import subprocess

    try:
        # Get git root
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            check=True,
        )
        repo_root = result.stdout.strip()
        return session_extract_from_trinity(repo_root)
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None
