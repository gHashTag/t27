"""
Test vetted-blocks JSON schema validation.
Ensures math-constants-sacred-chain.json follows expected structure.
"""

import json
import pytest
from pathlib import Path

from typing import Any, Dict


VETTED_BLOCKS_PATH = Path(__file__).parent.parent / "vetted-blocks" / "math-constants-sacred-chain.json"


class TestVettedBlocksSchema:
    """Test vetted-blocks JSON schema."""

    @pytest.fixture
    def vetted_blocks_data(self) -> Dict[str, Any]:
        """Load vetted blocks JSON."""
        with open(VETTED_BLOCKS_PATH) as f:
            return json.load(f)

    def test_root_structure(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test required root-level fields."""
        required_fields = [
            "catalog_name",
            "description",
            "subgraph",
            "nodes",
            "composition_chain",
            "downstream_phi_critical",
            "metadata",
        ]

        for field in required_fields:
            assert field in vetted_blocks_data, f"Missing required field: {field}"

    def test_nodes_structure(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test nodes array structure."""
        assert "nodes" in vetted_blocks_data
        assert isinstance(vetted_blocks_data["nodes"], list), "nodes must be an array"
        assert len(vetted_blocks_data["nodes"]) > 0, "nodes must not be empty"

        for node in vetted_blocks_data["nodes"]:
            required_node_fields = [
                "name",
                "node_id",
                "path",
                "tier",
                "kind",
                "strand",
                "exports",
                "sacred_level",
                "test_invariant",
            ]
            for field in required_node_fields:
                assert field in node, f"Node missing field: {field}"

            # Check exports is array
            assert isinstance(node["exports"], list), f"Node exports must be array: {node['name']}"
            assert len(node["exports"]) > 0, f"Node exports must not be empty: {node['name']}"

    def test_composition_chain_structure(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test composition chain structure."""
        assert "composition_chain" in vetted_blocks_data
        chain = vetted_blocks_data["composition_chain"]

        required_chain_fields = ["path", "total_tiers", "critical_invariants", "verification_method"]
        for field in required_chain_fields:
            assert field in chain, f"Composition chain missing field: {field}"

        assert isinstance(chain["path"], list), "composition_chain.path must be array"
        assert isinstance(chain["critical_invariants"], list), "critical_invariants must be array"

    def test_subgraph_format(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test subgraph field format."""
        assert "subgraph" in vetted_blocks_data
        subgraph = vetted_blocks_data["subgraph"]

        # Should contain arrow notation or readable description
        assert isinstance(subgraph, str), "subgraph must be string"
        assert len(subgraph) > 0, "subgraph must not be empty"

        # Should reference expected nodes
        assert "node 4" in subgraph or "math/constants" in subgraph, \
            f"subgraph should reference known nodes: {subgraph}"

    def test_metadata_structure(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test metadata fields."""
        assert "metadata" in vetted_blocks_data
        metadata = vetted_blocks_data["metadata"]

        required_metadata = [
            "catalog_version",
            "generated_from",
            "generated_date",
            "claera_domain",
        ]
        for field in required_metadata:
            assert field in metadata, f"Metadata missing field: {field}"

    def test_phi_critical_nodes_exist(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test downstream_phi_critical references actual nodes."""
        assert "downstream_phi_critical" in vetted_blocks_data
        downstream = vetted_blocks_data["downstream_phi_critical"]

        # Get all node paths
        node_paths = {node["path"] for node in vetted_blocks_data["nodes"]}

        for module in downstream:
            # Check if referenced module exists in nodes
            found = any(module in path for path in node_paths)
            assert found, f"downstream_phi_critical references unknown module: {module}"

    def test_tiers_are_positive(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test that all tier values are positive."""
        for node in vetted_blocks_data["nodes"]:
            assert "tier" in node, f"Node missing tier: {node['name']}"
            assert isinstance(node["tier"], int) or isinstance(node["tier"], float), \
                f"Node tier must be numeric: {node['name']}"
            assert node["tier"] > 0, f"Node tier must be positive: {node['name']}"

    def test_critical_invariants_not_empty(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test that critical invariants are defined."""
        chain = vetted_blocks_data["composition_chain"]
        assert "critical_invariants" in chain
        invariants = chain["critical_invariants"]

        assert isinstance(invariants, list), "critical_invariants must be array"
        assert len(invariants) > 0, "critical_invariants must not be empty"

        # Each invariant should be a non-empty string
        for inv in invariants:
            assert isinstance(inv, str), f"Invariant must be string: {inv}"
            assert len(inv) > 0, f"Invariant must not be empty"

    def test_node_ids_are_unique(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test that node IDs are unique."""
        node_ids = [node["node_id"] for node in vetted_blocks_data["nodes"]]
        assert len(node_ids) == len(set(node_ids)), \
            f"Node IDs must be unique: duplicates found"

    def test_expected_subgraph_nodes(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test that subgraph references actual nodes in data."""
        subgraph = vetted_blocks_data["subgraph"]

        # Expected nodes based on subgraph string
        expected_nodes = ["math/constants", "physics/chern-simons", "math/sacred_physics"]
        actual_names = [node["name"] for node in vetted_blocks_data["nodes"]]

        for expected in expected_nodes:
            assert expected in actual_names, f"Subgraph references {expected} but not found in nodes"

    def test_verification_method_is_string(self, vetted_blocks_data: Dict[str, Any]) -> None:
        """Test that verification_method is a non-empty string."""
        chain = vetted_blocks_data["composition_chain"]
        assert "verification_method" in chain
        method = chain["verification_method"]

        assert isinstance(method, str), "verification_method must be string"
        assert len(method) > 0, "verification_method must not be empty"
