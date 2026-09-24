#!/usr/bin/env python3
"""Detect non-deterministic file generation across backends.

This script runs each of the three non-deterministic backends (gen-c, gen-rust, gen-zig)
twice on the same set of specs and compares the output files to identify which files
differ between runs. This addresses the issue where 1-3 files out of 581 show
non-deterministic behavior across consecutive runs.

Usage:
    python3 tools/check_determinism.py    # local mode, tolerant of missing tools
    python3 tools/check_determinism.py --require  # CI mode, fails if any tool missing
"""
import os
import sys
import shutil
import subprocess
import tempfile
import hashlib
from pathlib import Path


# Import prerequisite utilities
_pq = __import__("_prereq", fromlist=["skip", "broken"])
skip, broken = _pq.skip, _pq.broken

# Configuration from the issue
GEN_SPECS_COUNT = 581  # Number of specs that generate successfully
BACKENDS = ["c", "rust", "zig"]  # The three non-deterministic backends
REQUIRE = "--require" in sys.argv


def find_t27c():
    """Find the t27c binary."""
    for p in ("target/debug/t27c", "target/release/t27c"):
        cand = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), p)
        if os.path.exists(cand):
            return cand
    return shutil.which("t27c")


def get_all_specs():
    """Get all .t27 spec files from the specs directory."""
    specs_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "specs")
    specs = []
    for root, dirs, files in os.walk(specs_dir):
        for file in files:
            if file.endswith(".t27"):
                specs.append(os.path.join(root, file))
    return sorted(specs)


def run_generation(t27c, backend, spec, output_dir):
    """Run t27c gen-<backend> on a spec and return the output files."""
    spec_name = os.path.splitext(os.path.basename(spec))[0]
    backend_output_dir = os.path.join(output_dir, backend)
    os.makedirs(backend_output_dir, exist_ok=True)
    
    # Run the generator
    cmd = [t27c, f"gen-{backend}", spec]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=backend_output_dir)
    
    if result.returncode != 0:
        print(f"  {backend} {spec_name}: generation failed")
        if result.stderr:
            print(f"    Error: {result.stderr.strip()}")
        return None
    
    # Collect all generated files
    generated_files = []
    for root, dirs, files in os.walk(backend_output_dir):
        for file in files:
            if file.startswith('.'):
                continue  # Skip hidden files
            file_path = os.path.join(root, file)
            relative_path = os.path.relpath(file_path, backend_output_dir)
            generated_files.append(relative_path)
    
    return generated_files


def hash_file(file_path):
    """Compute SHA256 hash of a file."""
    hash_sha256 = hashlib.sha256()
    try:
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_sha256.update(chunk)
        return hash_sha256.hexdigest()
    except (IOError, OSError):
        return None


def compare_directories(dir1, dir2, backend):
    """Compare two directories and report differences."""
    files1 = set()
    files2 = set()
    
    # Collect all files from both directories
    for root, dirs, files in os.walk(dir1):
        for file in files:
            if not file.startswith('.'):
                rel_path = os.path.relpath(os.path.join(root, file), dir1)
                files1.add(rel_path)
    
    for root, dirs, files in os.walk(dir2):
        for file in files:
            if not file.startswith('.'):
                rel_path = os.path.relpath(os.path.join(root, file), dir2)
                files2.add(rel_path)
    
    # Find files that exist in both but have different content
    common_files = files1 & files2
    differing_files = []
    
    for file in common_files:
        path1 = os.path.join(dir1, file)
        path2 = os.path.join(dir2, file)
        
        hash1 = hash_file(path1)
        hash2 = hash_file(path2)
        
        if hash1 != hash2:
            differing_files.append(file)
    
    # Files that exist in only one directory
    only_in_1 = files1 - files2
    only_in_2 = files2 - files1
    
    return differing_files, only_in_1, only_in_2


def check_backend_determinism(t27c, backend, specs, workdir):
    """Check determinism for a single backend."""
    print(f"Checking {backend} backend determinism...")
    
    # Run generation twice
    run1_dir = os.path.join(workdir, f"run1_{backend}")
    run2_dir = os.path.join(workdir, f"run2_{backend}")
    
    # First run
    print(f"  Running {backend} generation (first pass)...")
    run1_results = {}
    for spec in specs:
        spec_name = os.path.splitext(os.path.basename(spec))[0]
        result = run_generation(t27c, backend, spec, run1_dir)
        if result is not None:
            run1_results[spec_name] = result
    
    # Second run
    print(f"  Running {backend} generation (second pass)...")
    run2_results = {}
    for spec in specs:
        spec_name = os.path.splitext(os.path.basename(spec))[0]
        result = run_generation(t27c, backend, spec, run2_dir)
        if result is not None:
            run2_results[spec_name] = result
    
    # Compare results
    print(f"  Comparing {backend} results...")
    total_specs_with_output = len(set(run1_results.keys()) | set(run2_results.keys()))
    specs_common = set(run1_results.keys()) & set(run2_results.keys())
    
    print(f"    {backend}: {len(specs_common)}/{total_specs_with_output} specs produced output in both runs")
    
    non_deterministic_specs = []
    
    for spec_name in specs_common:
        spec_run1_dir = os.path.join(run1_dir, spec_name)
        spec_run2_dir = os.path.join(run2_dir, spec_name)
        
        if not os.path.exists(spec_run1_dir) or not os.path.exists(spec_run2_dir):
            continue
            
        differing, only1, only2 = compare_directories(spec_run1_dir, spec_run2_dir, backend)
        
        if differing or only1 or only2:
            non_deterministic_specs.append({
                'spec': spec_name,
                'differing': differing,
                'only_in_run1': only1,
                'only_in_run2': only2
            })
    
    return non_deterministic_specs, total_specs_with_output


def main():
    """Main function."""
    # Check prerequisites
    t27c = find_t27c()
    if not t27c:
        if REQUIRE:
            broken("t27c binary not found (build with `cargo build` first)")
        else:
            skip("t27c binary not found (build with `cargo build` first)")
    
    # Get all specs
    specs = get_all_specs()
    print(f"Found {len(specs)} .t27 spec files")
    
    # Check which backends are available
    available_backends = []
    for backend in BACKENDS:
        if shutil.which(f"t27c"):
            available_backends.append(backend)
        else:
            print(f"Warning: {backend} backend not available (t27c not found)")
    
    if not available_backends:
        if REQUIRE:
            broken("No backends available")
        else:
            skip("No backends available")
    
    # Check determinism for each backend
    with tempfile.TemporaryDirectory() as workdir:
        all_results = {}
        total_backends = len(available_backends)
        
        for i, backend in enumerate(available_backends, 1):
            print(f"\n[{i}/{total_backends}] Checking {backend} backend...")
            
            # Filter specs to only those that might generate with this backend
            # We don't know which ones will work, so try all of them
            non_det_specs, total_specs = check_backend_determinism(t27c, backend, specs, workdir)
            all_results[backend] = {
                'non_deterministic_specs': non_det_specs,
                'total_specs_with_output': total_specs,
                'differing_files_count': sum(len(spec['differing']) for spec in non_det_specs)
            }
        
        # Report results
        print("\n" + "="*60)
        print("NON-DETERMINISM DETECTION RESULTS")
        print("="*60)
        
        overall_non_determinism = False
        total_differing_files = 0
        
        for backend in available_backends:
            result = all_results[backend]
            non_det_specs = result['non_deterministic_specs']
            total_specs = result['total_specs_with_output']
            differing_count = result['differing_files_count']
            
            print(f"\n{backend.upper()} BACKEND:")
            print(f"  Specs with output: {total_specs}")
            print(f"  Non-deterministic specs: {len(non_det_specs)}")
            print(f"  Total differing files: {differing_count}")
            
            if non_det_specs:
                overall_non_determinism = True
                total_differing_files += differing_count
                
                print(f"  Non-deterministic specs:")
                for spec in non_det_specs:
                    print(f"    {spec['spec']}:")
                    if spec['differing']:
                        print(f"      Differing files: {len(spec['differing'])}")
                        for file in spec['differing'][:5]:  # Show first 5
                            print(f"        - {file}")
                        if len(spec['differing']) > 5:
                            print(f"        ... and {len(spec['differing']) - 5} more")
                    if spec['only_in_run1']:
                        print(f"      Only in first run: {list(spec['only_in_run1'])[:3]}")
                    if spec['only_in_run2']:
                        print(f"      Only in second run: {list(spec['only_in_run2'])[:3]}")
        
        print(f"\nSUMMARY:")
        print(f"  Backends checked: {len(available_backends)}")
        print(f"  Total non-deterministic specs: {sum(len(r['non_deterministic_specs']) for r in all_results.values())}")
        print(f"  Total differing files: {total_differing_files}")
        
        if total_differing_files > 0:
            print(f"  ⚠️  NON-DETERMINISM DETECTED: {total_differing_files} files differ between runs")
            if REQUIRE:
                print("  ❌ FAIL: Non-deterministic generation detected")
                sys.exit(1)
            else:
                print("  ⚠️  WARNING: Non-deterministic generation detected (continuing in tolerant mode)")
        else:
            print(f"  ✅ PASS: No non-determinism detected")
        
        sys.exit(0)


if __name__ == "__main__":
    main()