# contrib/backend/notebooklm/tests/test_e2e.py
# E2E test for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""End-to-end test for NotebookLM wrapup flow."""

import sys
from pathlib import Path
from datetime import datetime, timedelta

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.config import config_from_env
from contrib.backend.notebooklm.auth_token import AuthTokens, token_save, token_clear


def test_e2e_wrapup_flow_mock():
    """Test full wrapup flow with mocked NotebookLM client.

    This test simulates the complete workflow:
    1. Extract session context
    2. Format wrap-up summary
    3. Generate Markdown
    4. Verify all sections present
    """
    print("Testing E2E wrapup flow...")

    # Step 1: Create mock session context
    from contrib.backend.notebooklm.session import SessionContext

    session = SessionContext(
        session_id="test-session-123",
        repo_root="/tmp/test",
        branch="feature-test",
        skill_id="wrap-up-skill",
        issue_number=308,
        start_time="2026-04-08T00:00:00",
        tasks_completed=5,
        files_modified=3,
        git_status="clean",
    )

    assert session.session_id == "test-session-123"
    assert session.branch == "feature-test"
    print("  ✓ Session context created")

    # Step 2: Format wrap-up summary
    from contrib.backend.notebooklm.wrapup import wrapup_format_summary

    wrapup = wrapup_format_summary(
        session.to_dict(),
        "Completed NotebookLM integration Phase 0-3",
        "Used notebooklm-py SDK with cookie auth",
        "contrib/backend/notebooklm/*.py, specs/memory/notebooklm.t27",
        "Run integration tests and finalize",
    )

    assert wrapup["session"]["session_id"] == "test-session-123"
    assert wrapup["summary"] == "Completed NotebookLM integration Phase 0-3"
    assert "notebooklm-py SDK" in wrapup["key_decisions"]
    print("  ✓ Wrap-up summary formatted")

    # Step 3: Generate Markdown
    from contrib.backend.notebooklm.wrapup import wrapup_format_markdown

    markdown = wrapup_format_markdown(wrapup)

    # Verify all sections
    assert "# Session Wrap-up" in markdown
    assert "test-session-123" in markdown
    assert "feature-test" in markdown
    assert "wrap-up-skill" in markdown
    assert "308" in markdown

    # Verify 4 main sections
    assert "## Summary" in markdown
    assert "## Key Decisions" in markdown
    assert "## Files Changed" in markdown
    assert "## Next Steps" in markdown

    # Verify content
    assert "NotebookLM integration Phase 0-3" in markdown
    assert "notebooklm-py SDK" in markdown
    assert "contrib/backend/notebooklm" in markdown
    assert "Run integration tests" in markdown

    print("  ✓ Markdown formatted with all sections")

    print("\n[PASS] test_e2e_wrapup_flow_mock - full E2E flow simulated")
    return True


def test_e2e_config_validation():
    """Test that configuration loads and validates."""
    print("\nTesting config validation...")

    config = config_from_env()

    assert config is not None
    assert config.notebook_name == "t27-QUEEN-BRAIN"
    assert config.timeout_ms == 30000
    assert config.auto_refresh == True

    print("  ✓ Configuration loaded and validated")
    print("\n[PASS] test_e2e_config_validation")
    return True


def test_e2e_token_lifecycle():
    """Test token lifecycle: create, save, load, validate, clear."""
    print("\nTesting token lifecycle...")

    from contrib.backend.notebooklm.auth_token import (
        token_save,
        token_load,
        token_clear,
        token_is_valid,
    )

    # Clear any existing token
    token_clear()

    # Step 1: Create new token
    future = datetime.now() + timedelta(hours=1)
    tokens = AuthTokens(
        access_token="test-access-token",
        refresh_token="test-refresh-token",
        expires_at=future,
        token_type="bearer",
    )

    # Step 2: Save token
    assert token_save(tokens) == True
    print("  ✓ Token saved")

    # Step 3: Load token
    loaded = token_load()
    assert loaded is not None
    assert loaded.access_token == "test-access-token"
    assert loaded.refresh_token == "test-refresh-token"
    print("  ✓ Token loaded")

    # Step 4: Validate token
    assert token_is_valid(loaded) == True
    print("  ✓ Token validated")

    # Step 5: Clear token
    assert token_clear() == True
    assert token_load() is None
    print("  ✓ Token cleared")

    print("\n[PASS] test_e2e_token_lifecycle")
    return True


def test_e2e_client_state_management():
    """Test client state management."""
    print("\nTesting client state management...")

    from contrib.backend.notebooklm.client import (
        client_get_current,
        client_is_authenticated,
        client_reset,
    )

    # Initial state
    assert client_get_current() is None
    assert client_is_authenticated() is False
    print("  ✓ Initial state: no client, not authenticated")

    # Reset state
    client_reset()
    assert client_get_current() is None
    assert client_is_authenticated() is False
    print("  ✓ Reset state works")

    print("\n[PASS] test_e2e_client_state_management")
    return True


def main():
    """Run all E2E tests."""
    print("=" * 60)
    print("NOTEBOOKLM E2E TEST SUITE")
    print("=" * 60)
    print()

    tests = [
        ("E2E wrapup flow", test_e2e_wrapup_flow_mock),
        ("Config validation", test_e2e_config_validation),
        ("Token lifecycle", test_e2e_token_lifecycle),
        ("Client state management", test_e2e_client_state_management),
    ]

    passed = 0
    failed = 0

    for name, test_func in tests:
        try:
            if test_func():
                passed += 1
        except Exception as e:
            print(f"\n[FAIL] {name}: {e}")
            failed += 1

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Passed: {passed}/{len(tests)}")
    print(f"Failed: {failed}/{len(tests)}")

    if failed == 0:
        print("\nAll E2E tests passed!")
        return 0
    else:
        print(f"\n{failed} E2E test(s) failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
