#!/usr/bin/env python3
"""Run differential testbenches if they exist.

This script looks for differential testbenches in bootstrap/tests/rtl/<module>_diff.v
and runs them using vvp simulation. If no differential testbenches are found,
it exits cleanly without error.

The script is designed to be integrated into CI workflows so that when
differential testbenches are committed, they will be automatically run.
"""

import os
import subprocess
import sys
import glob
import tempfile
import shutil

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
DIFF_TESTBENCH_DIR = os.path.join(ROOT, "bootstrap/tests/rtl")

def run_differential_testbench(module_name, verilog_path, workdir):
    """Run a single differential testbench."""
    print(f"Running differential testbench for {module_name}")
    
    # Look for differential testbench files
    diff_patterns = [
        f"bootstrap/tests/rtl/{module_name}_diff.v",
        f"bootstrap/tests/rtl/{module_name}_diff_tb.v",
        f"bootstrap/tests/rtl/{module_name}_diff_tb.t27",
    ]
    
    diff_tb_v = None
    diff_tb_t27 = None
    
    for pattern in diff_patterns:
        full_path = os.path.join(ROOT, pattern)
        if os.path.exists(full_path):
            if pattern.endswith('.v'):
                diff_tb_v = full_path
            elif pattern.endswith('.t27'):
                diff_tb_t27 = full_path
            print(f"Found differential testbench: {pattern}")
    
    if not diff_tb_v and not diff_tb_t27:
        print(f"No differential testbench found for {module_name}")
        return True
    
    # Compile and run the differential testbench
    try:
        if diff_tb_v:
            # Compile with iverilog
            vvp_cmd = ["iverilog", "-g2012", "-DSIMULATION", diff_tb_v, "-o", f"{workdir}/{module_name}_diff_sim"]
            if subprocess.run(vvp_cmd, check=True, cwd=ROOT, capture_output=True, text=True).returncode != 0:
                print(f"Failed to compile differential testbench for {module_name}")
                return False
            
            # Run with vvp
            vvp_run_cmd = ["vvp", f"{workdir}/{module_name}_diff_sim"]
            result = subprocess.run(vvp_run_cmd, check=True, cwd=ROOT, capture_output=True, text=True)
            print(f"Differential testbench {module_name} output:")
            print(result.stdout)
            if result.stderr:
                print(f"Stderr: {result.stderr}")
        
        elif diff_tb_t27:
            # Convert .t27 to .v if needed, or run directly if supported
            print(f"Found .t27 differential testbench for {module_name} - t27 compilation needed")
            # For now, just report that we found it but need t27 compilation
            print(f"Differential testbench {module_name} found at {diff_tb_t27}")
            print("Note: .t27 testbenches require t27 compilation - not yet implemented")
            return True
            
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"Differential testbench {module_name} failed:")
        print(f"Command: {' '.join(e.cmd)}")
        print(f"Return code: {e.returncode}")
        if e.stdout:
            print(f"Stdout: {e.stdout}")
        if e.stderr:
            print(f"Stderr: {e.stderr}")
        return False

def main():
    """Main function to run all available differential testbenches."""
    print("Running differential testbenches...")
    
    if not os.path.exists(DIFF_TESTBENCH_DIR):
        print(f"No differential testbench directory found at {DIFF_TESTBENCH_DIR}")
        print("Differential testbenches should be placed in bootstrap/tests/rtl/<module>_diff.v")
        return 0
    
    # Find all differential testbenches
    diff_files = glob.glob(os.path.join(DIFF_TESTBENCH_DIR, "*_diff.v"))
    diff_tb_files = glob.glob(os.path.join(DIFF_TESTBENCH_DIR, "*_diff_tb.v"))
    
    all_diff_files = diff_files + diff_tb_files
    
    if not all_diff_files:
        print("No differential testbenches found")
        return 0
    
    print(f"Found {len(all_diff_files)} differential testbenches")
    
    # Create temporary work directory
    with tempfile.TemporaryDirectory() as workdir:
        success_count = 0
        failure_count = 0
        
        for diff_file in all_diff_files:
            # Extract module name from filename
            basename = os.path.basename(diff_file)
            module_name = basename.replace('_diff.v', '').replace('_diff_tb.v', '')
            
            print(f"\nProcessing {module_name}...")
            
            if run_differential_testbench(module_name, diff_file, workdir):
                print(f"✓ Differential testbench {module_name} passed")
                success_count += 1
            else:
                print(f"✗ Differential testbench {module_name} failed")
                failure_count += 1
        
        print(f"\nSummary:")
        print(f"Passed: {success_count}")
        print(f"Failed: {failure_count}")
        
        if failure_count > 0:
            return 1
        else:
            return 0

if __name__ == "__main__":
    sys.exit(main())