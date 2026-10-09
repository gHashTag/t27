#!/usr/bin/env python3

import os
import re
import glob

def get_gen_w_files_with_ports():
    """Get all gen_w*.py files that have corresponding .t27 files"""
    scripts_dir = "scripts"
    port_dir = "specs/port/scripts"
    
    # Get all gen_w*.py files
    gen_w_py_files = []
    for file in glob.glob(f"{scripts_dir}/gen_w*.py"):
        filename = os.path.basename(file)
        gen_w_py_files.append(filename)
    
    # Get all gen_w*.t27 files
    gen_w_t27_files = []
    for file in glob.glob(f"{port_dir}/gen_w*.t27"):
        filename = os.path.basename(file)
        # Convert .t27 to .py to check correspondence
        py_filename = filename.replace('.t27', '.py')
        gen_w_t27_files.append(py_filename)
    
    # Find intersection - files that have both
    files_with_ports = set(gen_w_py_files) & set(gen_w_t27_files)
    
    return sorted(files_with_ports)

def is_file_referenced(filepath, exclude_dirs=['docs/', 'specs/', '.trinity/', '.claude/', 'docs/now/', 'docs/reports/', '.git/']):
    """Check if a file is referenced anywhere in the repository (excluding specified dirs)"""
    # Convert filepath to relative path for searching
    rel_path = filepath
    if filepath.startswith('./'):
        rel_path = filepath[2:]
    
    # Search for references to the file
    try:
        # Use grep to search for the file path
        result = os.popen(f'grep -r "{rel_path}" . {" ".join(f"--exclude-dir={d}" for d in exclude_dirs)} 2>/dev/null').read()
        
        # Also search for just the filename without extension
        filename = os.path.basename(rel_path)
        name_without_ext = filename.replace('.py', '')
        
        # Check for module references (e.g., import statements)
        result2 = os.popen(f'grep -r "{name_without_ext}" . {" ".join(f"--exclude-dir={d}" for d in exclude_dirs)} 2>/dev/null').read()
        
        # Count unique lines that aren't just the file itself
        lines = set(result.strip().split('\n'))
        lines2 = set(result2.strip().split('\n'))
        
        # Filter out empty lines and the file path itself
        relevant_lines = [line for line in lines if line and rel_path not in line]
        relevant_lines2 = [line for line in lines2 if line and name_without_ext in line and rel_path not in line]
        
        return len(relevant_lines) > 0 or len(relevant_lines2) > 0
        
    except Exception as e:
        print(f"Error checking {filepath}: {e}")
        return False

def main():
    print("Finding gen_w*.py files with ports...")
    files_with_ports = get_gen_w_files_with_ports()
    print(f"Found {len(files_with_ports)} files with ports")
    
    print("\nChecking for references...")
    unreferenced_files = []
    
    for file in files_with_ports:
        filepath = f"scripts/{file}"
        if not is_file_referenced(filepath):
            unreferenced_files.append(file)
            print(f"UNREFERENCED: {file}")
        else:
            print(f"REFERENCED: {file}")
    
    print(f"\nSUMMARY:")
    print(f"Total files with ports: {len(files_with_ports)}")
    print(f"Unreferenced files: {len(unreferenced_files)}")
    print(f"Files to delete: {unreferenced_files}")
    
    # Write list to file for deletion
    with open('files_to_delete.txt', 'w') as f:
        for file in unreferenced_files:
            f.write(f"scripts/{file}\n")
    
    return unreferenced_files

if __name__ == "__main__":
    main()