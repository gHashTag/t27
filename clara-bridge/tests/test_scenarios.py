"""
Test scenarios JSON schema validation.
Ensures chern-simons-phi-verification.json follows expected structure.
"""

import json
import pytest
from pathlib import Path
from typing import Any, Dict

SCENARIOS_PATH = Path(__file__).parent.parent / "scenarios" / "chern-simons-phi-verification.json"


class TestScenariosSchema:
    """Test scenarios JSON schema."""

    @pytest.fixture
    def scenario_data(self) -> Dict[str, Any]:
        """Load scenario JSON."""
        with open(SCENARIOS_PATH) as f:
            return json.load(f)

    def test_root_structure(self, scenario_data: Dict[str, Any]) -> None:
        """Test required root-level fields."""
        required_fields = [
            "scenario_id",
            "name",
            "description",
            "subgraph",
            "steps",
            "invariants",
            "toxicity_policy",
            "metadata",
        ]

        for field in required_fields:
            assert field in scenario_data, f"Missing required field: {field}"

    def test_steps_structure(self, scenario_data: Dict[str, Any]) -> None:
        """Test steps array structure."""
        assert "steps" in scenario_data
        assert isinstance(scenario_data["steps"], list), "steps must be an array"
        assert len(scenario_data["steps"]) > 0, "steps must not be empty"

        required_step_fields = [
            "name",
            "phase",
            "command",
            "description",
            "expected_outcome",
            "verify_by",
            "affected_nodes",
        ]

        for i, step in enumerate(scenario_data["steps"]):
            for field in required_step_fields:
                assert field in step, f"Step {i} missing field: {field}"

            # Test phase is valid
            valid_phases = ["spec", "gen", "test", "verdict", "experience"]
            assert step["phase"] in valid_phases, \
                f"Step {i} has invalid phase: {step['phase']}"

            # Test optional depends_on field
            if "depends_on" in step:
                assert isinstance(step["depends_on"], list), \
                    f"Step {i} depends_on must be array: {step['name']}"

            # Verify dependencies reference existing steps
            if "depends_on" in step:
                step_names = [s["name"] for s in scenario_data["steps"]]
                for dep in step["depends_on"]:
                    assert dep in step_names, \
                        f"Step {i} depends_on references unknown step: {dep}"

    def test_invariants_structure(self, scenario_data: Dict[str, Any]) -> None:
        """Test invariants object structure."""
        assert "invariants" in scenario_data
        invariants = scenario_data["invariants"]

        # Should have at least TRINITY invariant
        assert "TRINITY" in invariants, "invariants must include TRINITY"
        assert isinstance(invariants["TRINITY"], str), "TRINITY must be string"
        assert len(invariants["TRINITY"]) > 0, "TRINITY invariant must not be empty"

    def test_toxicity_policy_structure(self, scenario_data: Dict[str, Any]) -> None:
        """Test toxicity_policy structure."""
        assert "toxicity_policy" in scenario_data
        policy = scenario_data["toxicity_policy"]

        required_policy_fields = [
            "blocked_modules",
            "quarantine_file",
            "unblock_requires",
        ]
        for field in required_policy_fields:
            assert field in policy, f"toxicity_policy missing field: {field}"

        assert isinstance(policy["blocked_modules"], list), "blocked_modules must be array"
        assert len(policy["blocked_modules"]) > 0, "blocked_modules must not be empty"

    def test_metadata_structure(self, scenario_data: Dict[str, Any]) -> None:
        """Test metadata fields."""
        assert "metadata" in scenario_data
        metadata = scenario_data["metadata"]

        required_metadata = [
            "scenario_version",
            "claera_domain",
            "created_date",
        ]
        for field in required_metadata:
            assert field in metadata, f"Metadata missing field: {field}"

    def test_verdict_step_marked_critical(self, scenario_data: Dict[str, Any]) -> None:
        """Test that verdict step is marked critical."""
        verdict_steps = [s for s in scenario_data["steps"] if s["phase"] == "verdict"]

        assert len(verdict_steps) > 0, "At least one verdict step must exist"
        assert verdict_steps[0].get("critical") == True, \
            "Verdict step must be marked critical"

    def test_test_steps_have_precision(self, scenario_data: Dict[str, Any]) -> None:
        """Test that test steps may specify precision."""
        test_steps = [s for s in scenario_data["steps"] if s["phase"] == "test"]

        for step in test_steps:
            # precision is optional but if present must be valid
            if "precision" in step:
                precision = step["precision"]
                assert isinstance(precision, (str, int, float)), \
                    f"Test step precision must be numeric or string: {step['name']}"

    def test_step_names_are_unique(self, scenario_data: Dict[str, Any]) -> None:
        """Test that step names are unique."""
        step_names = [s["name"] for s in scenario_data["steps"]]
        assert len(step_names) == len(set(step_names)), \
            "Step names must be unique: duplicates found"

    def test_phase_ordering_is_logical(self, scenario_data: Dict[str, Any]) -> None:
        """Test that phases follow logical order."""
        steps = scenario_data["steps"]
        phases = [s["phase"] for s in steps]

        # Check for proper ordering: spec before gen before test, test before verdict
        phase_indices = {"spec": [], "gen": [], "test": [], "verdict": [], "experience": []}

        for i, step in enumerate(steps):
            phase_indices[step["phase"]].append(i)

        # gen steps should not appear before spec steps
        for gen_idx in phase_indices["gen"]:
            for spec_idx in phase_indices["spec"]:
                assert gen_idx > spec_idx, \
                    f"gen step at {gen_idx} appears before spec step at {spec_idx}"

        # test steps should not appear before gen steps
        for test_idx in phase_indices["test"]:
            for gen_idx in phase_indices["gen"]:
                assert test_idx > gen_idx, \
                    f"test step at {test_idx} appears before gen step at {gen_idx}"

    def test_scenario_id_is_string(self, scenario_data: Dict[str, Any]) -> None:
        """Test that scenario_id is a non-empty string."""
        assert "scenario_id" in scenario_data
        assert isinstance(scenario_data["scenario_id"], str)
        assert len(scenario_data["scenario_id"]) > 0, "scenario_id must not be empty"

    def test_subgraph_is_string(self, scenario_data: Dict[str, Any]) -> None:
        """Test that subgraph field is a non-empty string."""
        assert "subgraph" in scenario_data
        assert isinstance(scenario_data["subgraph"], str)
        assert len(scenario_data["subgraph"]) > 0, "subgraph must not be empty"

    def test_critical_step_exists(self, scenario_data: Dict[str, Any]) -> None:
        """Test that at least one critical step exists."""
        critical_steps = [s for s in scenario_data["steps"] if s.get("critical")]

        assert len(critical_steps) > 0, "At least one critical step must exist"
