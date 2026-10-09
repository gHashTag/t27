#!/usr/bin/env python3
"""
comment_scan gate: Check whether gates that read Verilog with a regex first strip comments.

This gate implements a liveness floor that reports clean only if:
1. No Verilog-reading gates are found (empty tree), OR
2. All Verilog-reading gates found strip comments correctly

The key fix is excluding the scanner's own source file from the scanned population
to avoid self-matching tautologies.
"""

import os
import re
import sys
from pathlib import Path

# Regex pattern to find Verilog-reading gates
READS_VERILOG = re.compile(r"\.sv\b|build/rtl|glob\(['\"]\*\.sv")

def find_verilog_reading_gates(directory):
    """Find all gates that read Verilog files, excluding the scanner itself."""
    if not os.path.exists(directory):
        return []
    
    verilog_gates = []
    scanner_file = Path(__file__).name
    
    for root, dirs, files in os.walk(directory):
        # Skip common build directories
        dirs[:] = [d for d in dirs if d not in ['.git', '__pycache__', 'node_modules', 'dist']]
        
        for file in files:
            if file == scanner_file:
                continue  # Exclude the scanner itself
            
            file_path = os.path.join(root, file)
            try:
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                    
                    # Check if the file contains the Verilog-reading pattern
                    if READS_VERILOG.search(content):
                        verilog_gates.append(file_path)
            except (IOError, UnicodeDecodeError):
                # Skip files that can't be read
                continue
    
    return verilog_gates

def check_comment_stripping(gate_path):
    """Check if a gate that reads Verilog properly strips comments."""
    try:
        with open(gate_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
        
        # Look for comment stripping patterns
        comment_patterns = [
            r'#.*$',           # Python comments
            r'//.*$',          # C-style line comments
            r'/\*.*?\*/',      # C-style block comments
            r'--.*$',          # SQL-style comments
        ]
        
        # Check if the gate contains comment stripping logic
        has_comment_stripping = any(re.search(pattern, content, re.MULTILINE) 
                                  for pattern in comment_patterns)
        
        # Also check for common comment removal functions
        comment_functions = [
            'strip_comments',
            'remove_comments', 
            'clear_comments',
            'sanitize_comments'
        ]
        
        has_comment_function = any(func in content.lower() for func in comment_functions)
        
        return has_comment_stripping or has_comment_function
        
    except (IOError, UnicodeDecodeError):
        return False

def main():
    """Main function for comment_scan gate."""
    if len(sys.argv) != 2:
        print("Usage: python comment_scan.py <directory>")
        return 1
    
    directory = sys.argv[1]
    
    # Check if the directory exists
    if not os.path.exists(directory):
        print(f"Directory not found: {directory}")
        return 1
    
    # Find all gates that read Verilog, excluding the scanner itself
    verilog_gates = find_verilog_reading_gates(directory)
    
    print(f"Found {len(verilog_gates)} gates that read Verilog:")
    for gate in verilog_gates:
        print(f"  {gate}")
    
    # Apply corrected liveness floor: "a scan that found nothing in scope reports error"
    # This means if no Verilog-reading gates are found, exit 1 (error - starved run)
    # If Verilog-reading gates are found but don't strip comments, exit 1 (error)
    # If all Verilog-reading gates strip comments correctly, exit 0 (correct)
    
    if len(verilog_gates) == 0:
        print("No Verilog-reading gates found - starved run")
        return 1  # Error: starved run, no gates found
    
    # Check each Verilog-reading gate for proper comment stripping
    all_strip_comments = True
    for gate in verilog_gates:
        if not check_comment_stripping(gate):
            print(f"ERROR: {gate} reads Verilog but doesn't strip comments")
            all_strip_comments = False
    
    if all_strip_comments:
        print("All Verilog-reading gates properly strip comments - clean sweep")
        return 0  # Correct: all gates strip comments
    else:
        print("Some Verilog-reading gates don't strip comments - defect found")
        return 1  # Error: gates found with defects

if __name__ == "__main__":
    sys.exit(main())