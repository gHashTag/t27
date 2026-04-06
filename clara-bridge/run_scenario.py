#!/usr/bin/env python3
"""
CLARA-Bridge Scenario Runner

Reads scenario JSON and executes steps atomically.
Each step validates success before proceeding to next step.

Usage:
    python clara-bridge/run_scenario.py [--dry-run] [--step N] [--verbose] <scenario_file>

Arguments:
    --dry-run        Only print commands without executing
    --step N         Run only step N (0-indexed)
    --verbose         Show detailed output
    scenario_file    Path to scenario JSON file

Exit codes:
    0 - All steps passed
    1 - Scenario file not found or invalid
    2 - Command execution failed
    3 - Verification failed
    4 - tri/t27c not found in PATH
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any, List


class ScenarioError(Exception):
    """Base exception for scenario errors."""
    pass


class CommandNotFoundError(ScenarioError):
    """Raised when tri/t27c is not available."""
    pass


def load_scenario(scenario_path: str) -> Dict[str, Any]:
    """Load and validate scenario JSON file."""
    path = Path(scenario_path)
    if not path.exists():
        raise ScenarioError(f"Scenario file not found: {scenario_path}")

    with open(path, 'r') as f:
        data = json.load(f)

    required = ['scenario_id', 'name', 'steps']
    missing = [k for k in required if k not in data]
    if missing:
        raise ScenarioError(f"Invalid scenario: missing keys: {missing}")

    return data


def check_tri_available() -> None:
    """Verify tri/t27c is in PATH."""
    try:
        subprocess.run(['tri', '--version'],
                          capture_output=True,
                          text=True,
                          check=True)
    except FileNotFoundError:
        raise CommandNotFoundError(
            "tri/t27c not found in PATH.\n"
            "Build with: cd bootstrap && cargo build --release\n"
            "Or add to PATH: export PATH=$PATH:$(pwd)/bootstrap/target/release:$PATH"
        )
    except subprocess.CalledProcessError as e:
        raise CommandNotFoundError(
            f"tri command failed: {e}\n"
            "Ensure Trinity toolchain is properly installed."
        )


def execute_command(command: str,
                 step_name: str,
                 dry_run: bool = False,
                 step_num: int = 0) -> subprocess.CompletedProcess:
    """Execute a single command and return result."""
    if dry_run:
        print(f"[DRY-RUN] [{step_num}] {step_name}: {command}")
        # Mock success for dry-run
        result = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='[dry-run]',
            stderr=''
        )
        return result

    print(f"[{step_num}] Executing: {step_name}")
    print(f"  Command: {command}")

    try:
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            check=False  # Don't raise on non-zero exit
        )

        print(f"  Exit code: {result.returncode}")
        if result.stdout:
            print(f"  Output: {result.stdout[:200]}{'...' if len(result.stdout) > 200 else ''}")

        if result.stderr and result.stderr.strip():
            print(f"  Errors: {result.stderr[:200]}{'...' if len(result.stderr) > 200 else ''}")

        return result

    except FileNotFoundError as e:
        print(f"  Error: {e}")
        # Return failure exit code
        return subprocess.CompletedProcess(
            args=[],
            returncode=127,  # Command not found
            stdout='',
            stderr=str(e)
        )
    except Exception as e:
        print(f"  Error: {e}")
        return subprocess.CompletedProcess(
            args=[],
            returncode=1,
            stdout='',
            stderr=str(e)
        )


def verify_step_outcome(step: Dict[str, Any],
                   result: subprocess.CompletedProcess,
                   verbose: bool = False) -> bool:
    """Check if step execution met expected outcome."""
    expected_outcome = step.get('expected_outcome', '')
    verify_by = step.get('verify_by', '')

    if verbose:
        print(f"  Expected: {expected_outcome}")
        print(f"  Verify by: {verify_by}")

    # For MVP, we rely on exit codes
    # Non-zero exit = failure for tri/t27c commands
    # Python scripts (kepler_newton_tests.py) = 0 for success
    success = result.returncode == 0

    if not success:
        print(f"  FAILED: Step '{step['name']}' did not complete successfully")
        return False

    # If verify_by mentions specific output pattern, we could check stdout here
    # But for MVP, we trust exit codes and manual inspection
    if verbose:
        print(f"  PASSED: Step '{step['name']}' completed successfully")

    return True


def check_dependencies(steps: List[Dict[str, Any]],
                    current_step_num: int,
                    completed: set) -> bool:
    """Check if all dependencies have been completed."""
    step = steps[current_step_num]
    depends_on = step.get('depends_on', [])

    for dep in depends_on:
        if dep not in completed:
            print(f"  Dependency '{dep}' not completed. Cannot proceed.")
            return False

    return True


def run_scenario(scenario: Dict[str, Any],
               dry_run: bool = False,
               single_step: int = None,
               verbose: bool = False) -> int:
    """Run all steps in the scenario."""
    print(f"\n{'='*60}")
    print(f"Scenario: {scenario['name']}")
    print(f"ID: {scenario['scenario_id']}")
    print(f"{'='*60}\n")

    steps = scenario.get('steps', [])
    total_steps = len(steps)

    if single_step is not None:
        if single_step < 0 or single_step >= total_steps:
            print(f"  Invalid step number: {single_step}. Must be 0-{total_steps-1}")
            return 1
        steps_to_run = [steps[single_step]]
        print(f"  Running single step {single_step}/{total_steps-1}: {steps[single_step]['name']}")
    else:
        steps_to_run = steps
        print(f"  Running all {total_steps} step(s)...")

    completed = set()
    failed = False

    for i, step in enumerate(steps_to_run):
        step_num = i + 1

        # Check dependencies
        if not check_dependencies(steps, i, completed):
            failed = True
            break

        # Execute command
        result = execute_command(
            step['command'],
            step['name'],
            dry_run=dry_run,
            step_num=step_num
        )

        # Verify outcome
        if not verify_step_outcome(step, result, verbose):
            failed = True
            break

        # Mark as completed
        completed.add(step['name'])

    # Summary
    print(f"\n{'='*60}")
    if failed:
        print("FAILED: Scenario did not complete successfully")
        print("Review output above for details")
        return 2  # Command execution failed
    else:
        print(f"SUCCESS: All {len(steps_to_run)} step(s) completed")
        return 0  # All passed


def main():
    parser = argparse.ArgumentParser(
        description='CLARA-Bridge Scenario Runner',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        'scenario_file',
        help='Path to scenario JSON file'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Only print commands without executing'
    )
    parser.add_argument(
        '--step', '-s',
        type=int,
        metavar='N',
        help='Run only step N (0-indexed)'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show detailed output'
    )

    args = parser.parse_args()

    # Check tri/t27c availability
    try:
        check_tri_available()
    except CommandNotFoundError as e:
        print(f"\n{str(e)}")
        return 4  # tri not found

    # Load scenario
    try:
        scenario = load_scenario(args.scenario_file)
    except ScenarioError as e:
        print(f"\nError loading scenario: {e}")
        return 1  # Scenario file error

    # Run scenario
    exit_code = run_scenario(
        scenario,
        dry_run=args.dry_run,
        single_step=args.step,
        verbose=args.verbose
    )

    return exit_code


if __name__ == '__main__':
    sys.exit(main())
