#!/usr/bin/env python3
"""
Simple test runner for CLARA-Bridge tests.
No external dependencies required (no pytest needed).
"""

import json
import sys
import traceback
from pathlib import Path
from typing import Any, Callable, Dict, List, Tuple


class TestResult:
    """Result of a single test."""
    def __init__(self, name: str, passed: bool, error: str = ""):
        self.name = name
        self.passed = passed
        self.error = error


class TestRunner:
    """Simple test runner."""

    def __init__(self):
        self.results: List[TestResult] = []

    def run_test(self, func: Callable) -> None:
        """Run a single test function."""
        test_name = func.__name__
        try:
            func()
            self.results.append(TestResult(test_name, True))
            print(f"✓ {test_name}")
        except AssertionError as e:
            self.results.append(TestResult(test_name, False, str(e)))
            print(f"✗ {test_name}: {e}")
        except Exception as e:
            self.results.append(TestResult(test_name, False, traceback.format_exc()))
            print(f"✗ {test_name}: {e}")

    def summary(self) -> int:
        """Print summary and return exit code."""
        passed = sum(1 for r in self.results if r.passed)
        failed = len(self.results) - passed

        print(f"\n{'='*60}")
        print(f"Tests run: {len(self.results)}")
        print(f"Passed: {passed}")
        print(f"Failed: {failed}")
        print(f"{'='*60}")

        return 0 if failed == 0 else 1


# ==================== Vetted Blocks Tests ====================

def load_vetted_blocks() -> Dict[str, Any]:
    """Load vetted blocks JSON."""
    path = Path(__file__).parent.parent / "vetted-blocks" / "math-constants-sacred-chain.json"
    with open(path) as f:
        return json.load(f)


def test_vetted_blocks_root_structure() -> None:
    """Test required root-level fields."""
    data = load_vetted_blocks()
    required_fields = [
        "catalog_name", "description", "subgraph", "nodes",
        "composition_chain", "downstream_phi_critical", "metadata",
    ]
    for field in required_fields:
        assert field in data, f"Missing field: {field}"


def test_vetted_blocks_nodes_structure() -> None:
    """Test nodes array structure."""
    data = load_vetted_blocks()
    assert isinstance(data["nodes"], list)
    assert len(data["nodes"]) > 0

    for node in data["nodes"]:
        required = ["name", "node_id", "path", "tier", "exports", "test_invariant"]
        for field in required:
            assert field in node, f"Node missing {field}"


def test_vetted_blocks_composition_chain() -> None:
    """Test composition chain structure."""
    data = load_vetted_blocks()
    chain = data["composition_chain"]
    assert "path" in chain
    assert "critical_invariants" in chain
    assert isinstance(chain["critical_invariants"], list)


def test_vetted_blocks_metadata() -> None:
    """Test metadata structure."""
    data = load_vetted_blocks()
    metadata = data["metadata"]
    assert "catalog_version" in metadata
    assert "generated_from" in metadata
    assert "claera_domain" in metadata


# ==================== Scenarios Tests ====================

def load_scenario() -> Dict[str, Any]:
    """Load scenario JSON."""
    path = Path(__file__).parent.parent / "scenarios" / "chern-simons-phi-verification.json"
    with open(path) as f:
        return json.load(f)


def test_scenario_root_structure() -> None:
    """Test required root-level fields."""
    data = load_scenario()
    required = ["scenario_id", "name", "description", "steps", "invariants", "metadata"]
    for field in required:
        assert field in data, f"Missing field: {field}"


def test_scenario_steps_structure() -> None:
    """Test steps array structure."""
    data = load_scenario()
    assert isinstance(data["steps"], list)
    assert len(data["steps"]) > 0

    for step in data["steps"]:
        required = ["name", "phase", "command", "expected_outcome"]
        for field in required:
            assert field in step, f"Step missing {field}"


def test_scenario_verdict_step_critical() -> None:
    """Test that verdict step is marked critical."""
    data = load_scenario()
    verdict_steps = [s for s in data["steps"] if s["phase"] == "verdict"]
    assert len(verdict_steps) > 0
    assert verdict_steps[0].get("critical") == True


def test_scenario_step_names_unique() -> None:
    """Test that step names are unique."""
    data = load_scenario()
    names = [s["name"] for s in data["steps"]]
    assert len(names) == len(set(names)), "Duplicate step names found"


# ==================== Experience Schema Tests ====================

def load_experience_schema() -> Dict[str, Any]:
    """Load experience schema JSON."""
    path = Path(__file__).parent.parent / "audit-trail" / "experience-schema.json"
    with open(path) as f:
        return json.load(f)


def test_experience_schema_root() -> None:
    """Test root structure."""
    data = load_experience_schema()
    assert "schema_name" in data
    assert "schema_version" in data
    assert "description" in data


def test_experience_episode_entry() -> None:
    """Test episode_entry structure."""
    data = load_experience_schema()
    episode = data["episode_entry"]
    assert "episode_id" in episode
    assert "verdict" in episode
    assert "invariants_checked" in episode


def test_experience_mistakes_entry() -> None:
    """Test mistakes_entry structure."""
    data = load_experience_schema()
    mistakes = data["mistakes_entry"]
    # Verdict should mention toxic
    assert "toxic" in mistakes["verdict"].lower()
    assert isinstance(mistakes["blocked_modules"], list)
    assert "quarantine_timestamp" in mistakes


# ==================== Main ====================

def main() -> int:
    """Run all tests."""
    runner = TestRunner()

    print("="*60)
    print("CLARA-Bridge Tests")
    print("="*60)
    print("\n--- Vetted Blocks ---")
    runner.run_test(test_vetted_blocks_root_structure)
    runner.run_test(test_vetted_blocks_nodes_structure)
    runner.run_test(test_vetted_blocks_composition_chain)
    runner.run_test(test_vetted_blocks_metadata)

    print("\n--- Scenarios ---")
    runner.run_test(test_scenario_root_structure)
    runner.run_test(test_scenario_steps_structure)
    runner.run_test(test_scenario_verdict_step_critical)
    runner.run_test(test_scenario_step_names_unique)

    print("\n--- Experience Schema ---")
    runner.run_test(test_experience_schema_root)
    runner.run_test(test_experience_episode_entry)
    runner.run_test(test_experience_mistakes_entry)

    return runner.summary()


if __name__ == "__main__":
    sys.exit(main())
