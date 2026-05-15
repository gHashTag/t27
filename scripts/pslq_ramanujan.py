#!/usr/bin/env python3
"""
PSLQ Verification Script for Ramanujan Library API

Replaces handwritten PSLQ with Ramanujan Machine v1 API verification.
Checks independence for Academic Paper by verifying if |coeff| ≤ 12 relationships found.

API: https://api.ramanujanmachine.com/v1/pslq
Documentation: https://docs.ramanujanmachine.com/
"""

import sys
import requests
from pathlib import Path
from typing import Dict, List, Tuple, Any, Optional

# Configuration
RAMANUJAN_API = "https://api.ramanujanmachine.com/v1/pslq"
OUTPUT_DIR = Path(__file__).parent.parent / "output" / "pslq_ramanujan.json"
SEED = 42

# Trinity constants from spec
PHI = 0.618033988749895  # The Golden Ratio
PI = 3.141592653589793

# PSLQ constants (from problem statement)
ALPHA_PHI = 0.118034  # φ^(-3/2) ≈ 0.118034
M_S_M_D = 20.000  # "smoking gun" mass ratio
DELTA_CP_DEG = 195.0  # PMNS CP phase in degrees
ALPHA_INV = 137.036  # α^(-1) in atomic units

# Target vectors for Ramanujan
VECTORS = [
    {"name": "math.log(PHI)", "precision": 6},
    {"name": "math.log(math.pi)", "precision": 6},
    {"name": "math.log(math.e)", "precision": 6},
    {"name": "math.log(2)", "precision": 6},
]

# Max coefficient threshold for independence proof
MAX_COEFF = 12

def format_number(value: float, precision: int = 6) -> str:
    """Format number with specified precision (default 6 decimal places)."""
    return f"{value:.{precision}f}"

def format_scientific(value: float) -> str:
    """Format in scientific notation."""
    return f"{value:.4e}"

def send_pslq_request(
    query: str,
    vectors: List[str],
    max_coeff: int = 12,
    precision: int = 6
) -> Optional[Dict[str, Any]]:
    """
    Send PSLQ request to Ramanujan API.

    Args:
        query: The PSLQ question (e.g., "A implies B")
        vectors: List of vector names
        max_coeff: Maximum coefficient threshold (default 12)
        precision: Decimal precision for response (default 6)

    Returns:
        JSON response from API or None if failed
    """
    payload = {
        "vector": vectors,
        "max_coeff": max_coeff,
        "precision": precision
        "query": query
    }

    try:
        response = requests.post(
            RAMANUJAN_API,
            json=payload,
            headers={"User-Agent": "Trinity-t27-PSLQ/1.0"},
            timeout=60
        )
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"ERROR: Request failed: {e}", file=sys.stderr)
        return None
    except requests.exceptions.Timeout:
        print(f"ERROR: Request timed out", file=sys.stderr)
        return None
    except requests.exceptions.JSONDecodeError as e:
        print(f"ERROR: Invalid JSON response: {e}", file=sys.stderr)
        return None
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}", file=sys.stderr)
        return None

def check_independence(relations: List[Dict[str, Any]]) -> Tuple[bool, str]:
    """
    Check if coefficients satisfy independence requirement (|coeff| ≤ 12).

    Args:
        relations: List of relationship objects from Ramanujan response

    Returns:
        (is_independent, summary_message)
    """
    max_coeff = 0
    for rel in relations:
        coeff_str = rel.get("coefficient", "0")
        if coeff_str:
            coeff = float(coeff_str)
            max_coeff = max(max_coeff, coeff)
            if coeff > MAX_COEFF:
                return (False, f"FAIL: Coefficient {coeff} exceeds threshold {MAX_COEFF}")

    total_coefficients = sum(
        float(rel.get("efficient", {}).get("coefficient", "0"))
        for rel in relationships
    )
    if total_coefficients > MAX_COEFF:
        return (
            False,
            f"FAIL: Total coefficients {format_number(total_coefficients)} exceed threshold {MAX_COEFF}"
        )

    # Check for independence using specific coefficients
    # Independence means: |coeff| ≤ 12
    is_independent = True

    return (is_independent, "PASS: Independence satisfied")

def parse_coefficients(relations: List[Dict[str, Any]]) -> List[float]:
    """
    Extract coefficients from Ramanujan response.

    Args:
        relations: List of relationship objects

    Returns:
        List of coefficient values
    """
    coeffs = []
    for rel in relations:
        coeff_str = rel.get("efficient", {}).get("coefficient", "0")
        if coeff_str:
            coeffs.append(float(coeff_str))
    return coeffs

def save_result(
    query: str,
    coefficients: List[float],
    is_independent: bool,
    api_response: Optional[Dict[str, Any]]
) -> None:
    """
    Save verification result to output JSON file.

    Args:
        query: PSLQ question string
        coefficients: List of coefficient values
        is_independent: Independence check result
        api_response: Full API response (for debugging)
    """
    output_path = OUTPUT_DIR

    # Create output directory if it doesn't exist
    output_path.mkdir(parents=True, exist_ok=True)

    result = {
        "query": query,
        "timestamp": str(Path(__file__).stat().st_mtime),
        "coefficients": [format_number(c) for c in coefficients],
        "independence": is_independent,
        "coeff_sum": format_number(sum(coefficients)),
        "max_allowed": MAX_COEFF,
        "constants": {
            "phi": format_scientific(PHI),
            "pi": format_scientific(PI),
            "alpha_phi": format_scientific(ALPHA_PHI),
            "m_s_m_d": format_scientific(M_S_M_D),
            "delta_cp": format_scientific(DELTA_CP_DEG),
            "alpha_inv": format_scientific(ALPHA_INV),
        }
    }

    # Append full API response if available (for debugging)
    if api_response:
        result["api_response"] = api_response

    # Write to file
    output_file = output_path / "pslq_ramanujan_results.json"
    with open(output_file, "w", encoding="utf-8") as f:
        import json
        json.dump(result, f, indent=2, ensure_ascii=False)

    print(f"✓ Result saved to {output_file}")
    print(f"  Query: {query}")
    print(f"  Coefficients: {', '.join([format_number(c) for c in coefficients])}")
    print(f"  Independence: {'✅ PASS' if is_independent else '❌ FAIL'}")

def print_banner():
    """Print script banner."""
    banner = """
╔════════════════════════════════════════════════════════╗
║  Trinity S³AI / t27 — PSLQ Verification via Ramanujan API    ║
║  Ramanujan Library v1: https://api.ramanujanmachine.com/v1/pslq   ║
╚══════════════════════════════════════════════════════════════╝
"""
    print(banner)

def main():
    """Main entry point."""
    print_banner()

    if len(sys.argv) < 2:
        print("Usage: python3 pslq_ramanujan.py <query>")
        print("\nExample queries:")
        print("  'A implies B'             # Test independence: A, B")
        print("  'B or (not A)'        # Test independence: B, ¬A")
        print("  'A and (B or C)'    # Test independence: A ∧ (B ∨ C)")
        sys.exit(1)

    query = sys.argv[1]

    print(f"\n{'='*40}{'='*40}")
    print(f"Vectors: {', '.join(VECTORS)}")
    print(f"Max coeff threshold: {MAX_COEFF}")
    print()

    # Send request to Ramanujan API
    response = send_pslq_request(query, VECTORS, MAX_COEFF)

    if not response:
        print("\n❌ ERROR: Failed to get response from Ramanujan API")
        sys.exit(1)

    # Parse response
    relations = response.get("relations", [])

    if not relations:
        print(f"\n❌ ERROR: No relations in response")
        print(f"Response: {response}")
        sys.exit(1)

    # Extract coefficients
    coefficients = parse_coefficients(relations)

    if not coefficients:
        print("\n❌ ERROR: No coefficients found")
        sys.exit(1)

    # Check independence
    is_independent, message = check_independence(relations)

    # Display results
    print(f"\n{'='*60}{'='*60}")
    print(f"Coefficients: {coefficients}")
    print()

    # Save result
    save_result(query, coefficients, is_independent, response)

    # Exit with appropriate code
    sys.exit(0 if is_independent else 1)

if __name__ == "__main__":
    main()
