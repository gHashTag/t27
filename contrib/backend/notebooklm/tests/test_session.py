# contrib/backend/notebooklm/tests/test_session.py
# Unit tests for session.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for session context extraction."""

import sys
from pathlib import Path
import tempfile
import json

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.session import (
    SessionContext,
    _read_json_file,
    _read_jsonl_file,
)


def test_session_context_dataclass():
    """Test that SessionContext is a proper dataclass."""
    context = SessionContext(
        session_id="test-123",
        repo_root="/tmp/test",
        branch="master",
        skill_id="test-skill",
        issue_number=42,
        start_time="2026-04-08T00:00:00",
        tasks_completed=5,
        files_modified=3,
        git_status="clean",
    )

    assert context.session_id == "test-123"
    assert context.repo_root == "/tmp/test"
    assert context.branch == "master"
    assert context.skill_id == "test-skill"
    assert context.issue_number == 42
    assert context.tasks_completed == 5
    assert context.files_modified == 3
    print("[PASS] test_session_context_dataclass")


def test_session_context_to_dict():
    """Test that SessionContext.to_dict() works."""
    context = SessionContext(
        session_id="test-123",
        repo_root="/tmp/test",
        branch="master",
        skill_id="test-skill",
        issue_number=42,
        start_time="2026-04-08T00:00:00",
        tasks_completed=5,
        files_modified=3,
        git_status="clean",
    )

    as_dict = context.to_dict()

    assert isinstance(as_dict, dict)
    assert as_dict["session_id"] == "test-123"
    assert as_dict["branch"] == "master"
    print("[PASS] test_session_context_to_dict")


def test_read_json_file():
    """Test _read_json_file with valid JSON."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write('{"key": "value", "number": 42}')
        path = f.name

    try:
        result = _read_json_file(Path(path))
        assert result is not None
        assert result["key"] == "value"
        assert result["number"] == 42
        print("[PASS] test_read_json_file")
    finally:
        Path(path).unlink()


def test_read_json_file_not_found():
    """Test _read_json_file with non-existent file."""
    result = _read_json_file(Path("/nonexistent/path.json"))
    assert result is None
    print("[PASS] test_read_json_file_not_found")


def test_read_json_file_invalid():
    """Test _read_json_file with invalid JSON."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write('{"invalid": json}')
        path = f.name

    try:
        result = _read_json_file(Path(path))
        assert result is None
        print("[PASS] test_read_json_file_invalid")
    finally:
        Path(path).unlink()


def test_read_jsonl_file():
    """Test _read_jsonl_file with valid JSONL."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".jsonl", delete=False) as f:
        f.write('{"event": "test1"}\n')
        f.write('{"event": "test2"}\n')
        f.write('{"event": "test3"}\n')
        path = f.name

    try:
        result = _read_jsonl_file(Path(path))
        assert len(result) == 3
        assert result[0]["event"] == "test1"
        assert result[1]["event"] == "test2"
        assert result[2]["event"] == "test3"
        print("[PASS] test_read_jsonl_file")
    finally:
        Path(path).unlink()


def test_read_jsonl_file_not_found():
    """Test _read_jsonl_file with non-existent file."""
    result = _read_jsonl_file(Path("/nonexistent/path.jsonl"))
    assert result == []
    print("[PASS] test_read_jsonl_file_not_found")


def test_read_jsonl_file_empty():
    """Test _read_jsonl_file with empty file."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".jsonl", delete=False) as f:
        path = f.name

    try:
        result = _read_jsonl_file(Path(path))
        assert result == []
        print("[PASS] test_read_jsonl_file_empty")
    finally:
        Path(path).unlink()


if __name__ == "__main__":
    test_session_context_dataclass()
    test_session_context_to_dict()
    test_read_json_file()
    test_read_json_file_not_found()
    test_read_json_file_invalid()
    test_read_jsonl_file()
    test_read_jsonl_file_not_found()
    test_read_jsonl_file_empty()
    print("\nAll session tests passed!")
