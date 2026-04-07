# contrib/backend/notebooklm/tests/test_wrapup.py
# Unit tests for wrapup.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for wrap-up summary formatting."""

import sys
from pathlib import Path

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.wrapup import (
    WrapupSummary,
    wrapup_format_summary,
    wrapup_format_markdown,
)


def test_wrapup_summary_dataclass():
    """Test that WrapupSummary is a proper dataclass."""
    session = {
        "session_id": "test-123",
        "branch": "master",
        "skill_id": "test-skill",
        "issue_number": 42,
    }
    wrapup = WrapupSummary(
        session=session,
        summary="Test summary",
        key_decisions="Decision 1",
        files_changed="file1.py",
        next_steps="Step 1",
        created_at="2026-04-08T00:00:00",
    )

    assert wrapup.session == session
    assert wrapup.summary == "Test summary"
    assert wrapup.key_decisions == "Decision 1"
    print("[PASS] test_wrapup_summary_dataclass")


def test_wrapup_format_summary():
    """Test wrapup_format_summary."""
    session = {
        "session_id": "test-123",
        "branch": "master",
        "skill_id": "test-skill",
        "issue_number": 42,
    }
    result = wrapup_format_summary(
        session,
        "Test summary",
        "Decision 1",
        "file1.py",
        "Step 1",
    )

    assert result["session"] == session
    assert result["summary"] == "Test summary"
    assert result["key_decisions"] == "Decision 1"
    assert result["files_changed"] == "file1.py"
    assert result["next_steps"] == "Step 1"
    assert "created_at" in result
    print("[PASS] test_wrapup_format_summary")


def test_wrapup_format_markdown():
    """Test wrapup_format_markdown."""
    session = {
        "session_id": "test-123",
        "branch": "master",
        "skill_id": "test-skill",
        "issue_number": 42,
    }
    wrapup = {
        "session": session,
        "summary": "Test summary",
        "key_decisions": "Decision 1",
        "files_changed": "file1.py",
        "next_steps": "Step 1",
        "created_at": "2026-04-08T00:00:00",
    }

    markdown = wrapup_format_markdown(wrapup)

    assert "# Session Wrap-up" in markdown
    assert "test-123" in markdown
    assert "master" in markdown
    assert "test-skill" in markdown
    assert "42" in markdown
    assert "## Summary" in markdown
    assert "Test summary" in markdown
    assert "## Key Decisions" in markdown
    assert "Decision 1" in markdown
    assert "## Files Changed" in markdown
    assert "## Next Steps" in markdown
    assert "Step 1" in markdown
    print("[PASS] test_wrapup_format_markdown")


def test_wrapup_format_markdown_all_sections():
    """Test that wrapup_format_markdown has all 4 sections."""
    session = {
        "session_id": "test",
        "branch": "master",
        "skill_id": "test",
        "issue_number": 1,
    }
    wrapup = {
        "session": session,
        "summary": "",
        "key_decisions": "",
        "files_changed": "",
        "next_steps": "",
        "created_at": "2026-04-08T00:00:00",
    }

    markdown = wrapup_format_markdown(wrapup)

    required_sections = [
        "## Summary",
        "## Key Decisions",
        "## Files Changed",
        "## Next Steps",
    ]

    for section in required_sections:
        assert section in markdown, f"Missing section: {section}"
    print("[PASS] test_wrapup_format_markdown_all_sections")


def test_wrapup_format_markdown_metadata():
    """Test that wrapup_format_markdown contains metadata."""
    session = {
        "session_id": "test-123",
        "branch": "feature-branch",
        "skill_id": "my-skill",
        "issue_number": 99,
    }
    wrapup = {
        "session": session,
        "summary": "",
        "key_decisions": "",
        "files_changed": "",
        "next_steps": "",
        "created_at": "2026-04-08T00:00:00",
    }

    markdown = wrapup_format_markdown(wrapup)

    # Check metadata fields
    assert "test-123" in markdown
    assert "feature-branch" in markdown
    assert "my-skill" in markdown
    assert "99" in markdown
    assert "2026-04-08T00:00:00" in markdown
    print("[PASS] test_wrapup_format_markdown_metadata")


if __name__ == "__main__":
    test_wrapup_summary_dataclass()
    test_wrapup_format_summary()
    test_wrapup_format_markdown()
    test_wrapup_format_markdown_all_sections()
    test_wrapup_format_markdown_metadata()
    print("\nAll wrapup tests passed!")
