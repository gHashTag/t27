"""
Test experience-schema.json validation.
Ensures audit trail schema follows expected structure.
"""

import json
import pytest
from pathlib import Path
from typing import Any, Dict

EXPERIENCE_SCHEMA_PATH = Path(__file__).parent.parent / "audit-trail" / "experience-schema.json"


class TestExperienceSchema:
    """Test experience JSON schema."""

    @pytest.fixture
    def schema_data(self) -> Dict[str, Any]:
        """Load experience schema JSON."""
        with open(EXPERIENCE_SCHEMA_PATH) as f:
            return json.load(f)

    def test_root_structure(self, schema_data: Dict[str, Any]) -> None:
        """Test required root-level fields."""
        required_fields = [
            "schema_name",
            "schema_version",
            "description",
        ]

        for field in required_fields:
            assert field in schema_data, f"Missing required field: {field}"

    def test_episode_entry_structure(self, schema_data: Dict[str, Any]) -> None:
        """Test episode_entry structure."""
        assert "episode_entry" in schema_data
        episode = schema_data["episode_entry"]

        required_fields = [
            "episode_id",
            "timestamp",
            "verdict",
            "error_type",
            "blocked_modules",
            "explanation",
            "invariants_checked",
            "metrics",
        ]
        for field in required_fields:
            assert field in episode, f"episode_entry missing field: {field}"

        # Test invariants_checked structure
        invariants = episode["invariants_checked"]
        assert "passed" in invariants, "invariants_checked must have 'passed' array"
        assert "failed" in invariants, "invariants_checked must have 'failed' array"
        assert isinstance(invariants["passed"], list), "invariants_checked.passed must be array"
        assert isinstance(invariants["failed"], list), "invariants_checked.failed must be array"

    def test_metrics_structure(self, schema_data: Dict[str, Any]) -> None:
        """Test metrics structure."""
        episode = schema_data["episode_entry"]

        assert "metrics" in episode
        metrics = episode["metrics"]

        required_metrics = [
            "conformance_score",
            "precision_used",
            "test_count",
            "test_passed",
        ]
        for field in required_metrics:
            assert field in metrics, f"metrics missing field: {field}"

        # Test numeric types
        assert isinstance(metrics["conformance_score"], float), "conformance_score must be float"
        assert 0.0 <= metrics["conformance_score"] <= 1.0, \
            "conformance_score must be between 0.0 and 1.0"

        assert isinstance(metrics["test_count"], int), "test_count must be integer"
        assert isinstance(metrics["test_passed"], int), "test_passed must be integer"

    def test_mistakes_entry_structure(self, schema_data: Dict[str, Any]) -> None:
        """Test mistakes_entry structure."""
        assert "mistakes_entry" in schema_data
        mistakes = schema_data["mistakes_entry"]

        required_fields = [
            "episode_id",
            "verdict",
            "error_type",
            "blocked_modules",
            "quarantine_timestamp",
        ]
        for field in required_fields:
            assert field in mistakes, f"mistakes_entry missing field: {field}"

        assert mistakes["verdict"] == "toxic", \
            "mistakes_entry verdict must be 'toxic'"

        assert isinstance(mistakes["blocked_modules"], list), "blocked_modules must be array"
        assert len(mistakes["blocked_modules"]) > 0, "blocked_modules must not be empty"

    def test_learning_entry_structure(self, schema_data: Dict[str, Any]) -> None:
        """Test learning_entry structure."""
        assert "learning_entry" in schema_data
        learning = schema_data["learning_entry"]

        required_fields = [
            "episode_id",
            "lesson_learned",
            "pattern_type",
            "confidence",
        ]
        for field in required_fields:
            assert field in learning, f"learning_entry missing field: {field}"

        assert isinstance(learning["confidence"], float), "confidence must be float"
        assert 0.0 <= learning["confidence"] <= 1.0, \
            "confidence must be between 0.0 and 1.0"

        valid_pattern_types = [
            "invariant-stability",
            "composition-pattern",
            "test-coverage",
        ]
        assert learning["pattern_type"] in valid_pattern_types, \
            f"Invalid pattern_type: {learning['pattern_type']}"

    def test_directories_structure(self, schema_data: Dict[str, Any]) -> None:
        """Test directories structure."""
        assert "directories" in schema_data
        directories = schema_data["directories"]

        required_dirs = ["episodes", "learnings", "mistakes"]
        for dir_name in required_dirs:
            assert dir_name in directories, f"directories missing field: {dir_name}"

        formats = directories["formats"]
        assert "episodes" in formats, "formats missing episodes field"
        assert formats["episodes"] == ".jsonl", "episodes format must be .jsonl"

    def test_quarantine_semantics(self, schema_data: Dict[str, Any]) -> None:
        """Test quarantine_semantics structure."""
        assert "quarantine_semantics" in schema_data
        quarantine = schema_data["quarantine_semantics"]

        required_fields = ["block", "unblock", "not"]
        for field in required_fields:
            assert field in quarantine, f"quarantine_semantics missing field: {field}"

        # Each field should be descriptive
        for field in required_fields:
            assert isinstance(quarantine[field], str), \
                f"quarantine_semantics.{field} must be string"
            assert len(quarantine[field]) > 0, \
                f"quarantine_semantics.{field} must not be empty"

    def test_verdict_values(self, schema_data: Dict[str, Any]) -> None:
        """Test that verdict allows expected values."""
        episode = schema_data["episode_entry"]

        valid_verdicts = ["clean", "toxic", "blocked"]
        assert episode["verdict"] in valid_verdicts, \
            f"Invalid verdict value: {episode['verdict']}"

    def test_error_type_values(self, schema_data: Dict[str, Any]) -> None:
        """Test that error_type allows expected values."""
        episode = schema_data["episode_entry"]

        valid_error_types = ["regression", "invariant_violation", "conformance_failure", None]
        assert episode["error_type"] in valid_error_types, \
            f"Invalid error_type: {episode['error_type']}"

    def test_schema_version_is_string(self, schema_data: Dict[str, Any]) -> None:
        """Test that schema_version is a non-empty string."""
        assert "schema_version" in schema_data
        assert isinstance(schema_data["schema_version"], str)
        assert len(schema_data["schema_version"]) > 0, "schema_version must not be empty"
