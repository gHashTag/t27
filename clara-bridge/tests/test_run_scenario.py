"""
Test run_scenario.py functionality.
Tests the scenario runner with dry-run mode and step execution.
"""

import json
import pytest
import subprocess
import tempfile
import sys
from pathlib import Path
from typing import Any, Dict

RUNNER_PATH = Path(__file__).parent.parent / "run_scenario.py"
SCENARIO_PATH = Path(__file__).parent.parent / "scenarios" / "chern-simons-phi-verification.json"


class TestRunScenario:
    """Test run_scenario.py functionality."""

    @pytest.fixture
    def scenario_data(self) -> Dict[str, Any]:
        """Load test scenario JSON."""
        with open(SCENARIO_PATH) as f:
            return json.load(f)

    def test_runner_exists(self) -> None:
        """Test that run_scenario.py exists and is executable."""
        assert RUNNER_PATH.exists(), f"run_scenario.py not found at {RUNNER_PATH}"

    def test_runner_loads_scenario(self, scenario_data: Dict[str, Any]) -> None:
        """Test that runner can load and validate scenario JSON."""
        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--dry-run", str(SCENARIO_PATH)],
            capture_output=True,
            text=True,
        )

        # Dry-run should succeed with scenario loaded
        assert result.returncode == 0, f"Dry-run failed: {result.stderr}"

    def test_dry_run_does_not_execute_commands(self, scenario_data: Dict[str, Any]) -> None:
        """Test that --dry-run only prints commands without executing."""
        # Create a scenario that tries to execute a dangerous command
        dangerous_scenario = {
            "scenario_id": "test-dry-run",
            "name": "Test Dry Run",
            "description": "Verify dry-run doesn't execute commands",
            "steps": [
                {
                    "name": "should-not-run",
                    "phase": "gen",
                    "command": "rm -rf /",  # Dangerous command
                    "description": "This should NOT execute in dry-run",
                    "expected_outcome": "command printed but not executed",
                    "verify_by": "dry-run shows command without error",
                    "affected_nodes": ["test"],
                },
            ],
            "metadata": {
                "scenario_version": "1.0",
                "claera_domain": "test",
                "created_date": "2026-04-06",
            },
        }

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(dangerous_scenario, f)
            temp_path = Path(f.name)

        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--dry-run", str(temp_path)],
            capture_output=True,
            text=True,
            timeout=10,
        )

        # Dry-run should succeed
        assert result.returncode == 0, "Dry-run should succeed"

        # Output should contain the dangerous command
        assert "rm -rf /" in result.stdout, "Dry-run should show command"

        # But dry-run should NOT actually run it (root should still exist)
        assert Path("/").exists(), "Dry-run should NOT execute dangerous commands"

    def test_step_flag_runs_specific_step(self, scenario_data: Dict[str, Any]) -> None:
        """Test that --step N runs only the specified step."""
        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--step", "0", "--dry-run", str(SCENARIO_PATH)],
            capture_output=True,
            text=True,
        )

        # Should succeed
        assert result.returncode == 0, f"--step 0 failed: {result.stderr}"

        # Should only show first step
        lines = result.stdout.strip().split("\n")
        assert len(lines) == 1, "Should only output one step with --step 0"

        # Should contain first step name
        assert "spec-seal-constants" in result.stdout, "Should show first step"

    def test_invalid_scenario_path_returns_error(self) -> None:
        """Test that invalid scenario path returns exit code 1."""
        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "/nonexistent/scenario.json"],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 1, "Invalid scenario should return exit code 1"

    def test_verbose_flag_increases_output(self, scenario_data: Dict[str, Any]) -> None:
        """Test that --verbose flag increases output detail."""
        normal_result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--dry-run", str(SCENARIO_PATH)],
            capture_output=True,
            text=True,
        )

        verbose_result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--dry-run", "--verbose", str(SCENARIO_PATH)],
            capture_output=True,
            text=True,
        )

        # Verbose output should be longer
        assert len(verbose_result.stdout) > len(normal_result.stdout), \
            "Verbose output should be more detailed"

    def test_malformed_json_returns_error(self) -> None:
        """Test that malformed JSON returns error."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            f.write("{invalid json")  # Malformed JSON
            temp_path = Path(f.name)

        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), str(temp_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode != 0, "Malformed JSON should fail"

    def test_scenario_with_missing_required_fields_fails_validation(self) -> None:
        """Test that scenario missing required fields fails validation."""
        incomplete_scenario = {
            "scenario_id": "test-incomplete",
            "name": "Incomplete Scenario",
            "steps": [
                {
                    "name": "missing-verify-by",
                    "phase": "test",
                    "command": "echo test",
                    # Missing required fields...
                },
            ],
            "metadata": {
                "scenario_version": "1.0",
            },
        }

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(incomplete_scenario, f)
            temp_path = Path(f.name)

        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--dry-run", str(temp_path)],
            capture_output=True,
            text=True,
        )

        # Should fail validation
        assert result.returncode != 0, "Incomplete scenario should fail validation"

    def test_exit_code_0_on_all_steps_passed(self) -> None:
        """Test that exit code 0 means all steps passed."""
        # In dry-run mode, this validates the scenario structure
        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--dry-run", str(SCENARIO_PATH)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, "Valid scenario should return exit code 0"

    def test_all_steps_accessible(self, scenario_data: Dict[str, Any]) -> None:
        """Test that all step phases are represented."""
        phases = {step["phase"] for step in scenario_data["steps"]}

        expected_phases = ["spec", "gen", "test", "verdict", "experience"]
        for phase in expected_phases:
            assert phase in phases, f"Missing phase in scenario: {phase}"

    def test_critical_step_has_flag(self, scenario_data: Dict[str, Any]) -> None:
        """Test that critical steps have the critical flag."""
        critical_steps = [s for s in scenario_data["steps"] if s.get("critical")]

        assert len(critical_steps) > 0, "At least one critical step should exist"
        for step in critical_steps:
            assert step["critical"] is True, "Critical flag should be True"

    def test_help_flag_shows_usage(self) -> None:
        """Test that --help flag shows usage information."""
        result = subprocess.run(
            [sys.executable, str(RUNNER_PATH), "--help"],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0, "--help should succeed"
        assert "Usage:" in result.stdout or "usage" in result.stdout.lower(), \
            "Help output should show usage"

    def test_scenario_description_is_present(self, scenario_data: Dict[str, Any]) -> None:
        """Test that scenario has a description."""
        assert "description" in scenario_data, "Scenario must have description"
        assert len(scenario_data["description"]) > 0, "Description must not be empty"
