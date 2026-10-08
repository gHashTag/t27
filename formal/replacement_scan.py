#!/usr/bin/env python3
"""
Scanner to detect the re-baseline idiom used by gates.

This scanner detects when gates write a fresh baseline over a possibly-broken
tree and exit 0, which is the defect Prop. 209 exists for. It looks for
patterns where baselines are written without proper validation.
"""

import os
import sys
import re
from pathlib import Path
from typing import List, Dict, Set, Optional

def find_baseline_files(root: Path) -> List[Path]:
    """Find all baseline files in the repository."""
    baseline_files = []
    baseline_patterns = [
        "corpus_size_baseline.txt",
        "specs_generate_baseline.txt", 
        "vector_data_baseline.txt",
        "withdrawn_live_baseline.txt",
        "seal_baseline.txt",
        "duplicate_declarations_baseline.txt",
        "lowered_collisions_baseline.txt"
    ]
    
    for pattern in baseline_patterns:
        baseline_file = root / pattern
        if baseline_file.exists():
            baseline_files.append(baseline_file)
    
    return baseline_files

def scan_for_rebaseline_patterns(root: Path) -> Dict[str, List[str]]:
    """Scan for re-baseline idiom patterns in Python files."""
    findings = {}
    
    # Look for patterns where baselines are written
    baseline_write_patterns = [
        r'write_baseline\s*\(',
        r'baseline.*updated',
        r'--update-baseline',
        r'baseline.*written',
        r'record.*baseline'
    ]
    
    # Look for potential validation patterns that should be present
    validation_patterns = [
        r'validate.*before.*write',
        r'check.*before.*baseline',
        r'ensure.*valid.*write',
        r'if.*fail.*exit',
        r'raise.*error.*invalid'
    ]
    
    # Scan Python files
    python_files = list(root.rglob("*.py"))
    
    for file_path in python_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            # Check for baseline writing patterns
            baseline_matches = []
            for pattern in baseline_write_patterns:
                matches = re.findall(pattern, content, re.IGNORECASE)
                if matches:
                    baseline_matches.extend(matches)
            
            if baseline_matches:
                # Check if validation patterns are present
                validation_found = False
                for pattern in validation_patterns:
                    if re.search(pattern, content, re.IGNORECASE):
                        validation_found = True
                        break
                
                if not validation_found:
                    findings[str(file_path)] = baseline_matches
                    
        except (OSError, UnicodeDecodeError):
            # Skip files that can't be read
            continue
    
    return findings

def check_baseline_files(root: Path) -> Dict[str, Dict]:
    """Check baseline files for potential issues."""
    results = {}
    
    baseline_files = find_baseline_files(root)
    
    for baseline_file in baseline_files:
        try:
            with open(baseline_file, 'r', encoding='utf-8') as f:
                content = f.read().strip()
            
            # Check if file is empty or contains zero values
            is_empty = not content
            contains_zero = '0' in content if content else False
            
            results[str(baseline_file)] = {
                'exists': True,
                'empty': is_empty,
                'contains_zero': contains_zero,
                'size': len(content)
            }
            
        except (OSError, UnicodeDecodeError):
            results[str(baseline_file)] = {
                'exists': False,
                'error': 'Could not read file'
            }
    
    return results

def main():
    """Main scanner function."""
    root = Path.cwd()
    
    print("=== Re-baseline Idiom Scanner ===")
    print(f"Scanning repository: {root}")
    print()
    
    # Scan for re-baseline patterns
    print("1. Scanning for re-baseline patterns...")
    rebaseline_findings = scan_for_rebaseline_patterns(root)
    
    if rebaseline_findings:
        print("   POTENTIAL ISSUES FOUND:")
        for file_path, patterns in rebaseline_findings.items():
            print(f"   File: {file_path}")
            for pattern in patterns:
                print(f"     - {pattern}")
            print()
    else:
        print("   No re-baseline patterns found without validation.")
        print()
    
    # Check baseline files
    print("2. Checking baseline files...")
    baseline_results = check_baseline_files(root)
    
    for file_path, info in baseline_results.items():
        if info['exists']:
            status = []
            if info['empty']:
                status.append("EMPTY")
            if info['contains_zero']:
                status.append("CONTAINS_ZERO")
            
            if status:
                print(f"   WARNING: {file_path} - {', '.join(status)}")
            else:
                print(f"   OK: {file_path} ({info['size']} bytes)")
        else:
            print(f"   ERROR: {file_path} - {info.get('error', 'File not found')}")
    
    print()
    
    # Summary
    total_issues = len(rebaseline_findings)
    total_baseline_files = len([r for r in baseline_results.values() if r['exists']])
    problematic_baselines = len([r for r in baseline_results.values() 
                                if r.get('empty') or r.get('contains_zero')])
    
    print("=== SUMMARY ===")
    print(f"Files with re-baseline patterns without validation: {total_issues}")
    print(f"Baseline files found: {total_baseline_files}")
    print(f"Baseline files with issues: {problematic_baselines}")
    
    if total_issues > 0 or problematic_baselines > 0:
        print("\n⚠️  POTENTIAL ISSUES DETECTED")
        print("Consider reviewing the identified files for proper validation before writing baselines.")
        return 1
    else:
        print("\n✅ NO ISSUES DETECTED")
        return 0

if __name__ == "__main__":
    sys.exit(main())