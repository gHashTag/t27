#!/usr/bin/env python3
"""tri unrun-checks -- find check:* scripts that no CI workflow runs"""

import argparse
import json
import os
import re
import sys
from pathlib import Path


def parse_arguments():
    parser = argparse.ArgumentParser(description='Find check:* scripts that no CI workflow runs')
    parser.add_argument('--package', default='package.json', help='Path to package.json file')
    parser.add_argument('--workflows', default='.github', help='Directory containing workflow files')
    parser.add_argument('--prefix', default='check:', help='Script prefix to check')
    return parser.parse_args()


def load_package_json(package_path):
    """Load and parse package.json file"""
    try:
        with open(package_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: package file '{package_path}' not found", file=sys.stderr)
        return None
    except json.JSONDecodeError as e:
        print(f"Error: cannot parse package.json: {e}", file=sys.stderr)
        return None


def extract_script_names(package_data, prefix):
    """Extract script names that start with the given prefix"""
    scripts = package_data.get('scripts', {})
    return [name for name in scripts.keys() if name.startswith(prefix)]


def find_script_references(workflows_dir, prefix):
    """Find all references to scripts in workflow files"""
    references = set()
    pattern = re.compile(rf'npm\s+run\s+([a-zA-Z0-9_:-]+)')
    
    workflows_path = Path(workflows_dir)
    if not workflows_path.exists():
        return references
    
    for yml_file in workflows_path.rglob('*.yml'):
        try:
            with open(yml_file, 'r', encoding='utf-8') as f:
                content = f.read()
                # Find all matches and check if they're exact matches
                for match in pattern.finditer(content):
                    script_name = match.group(1)
                    # Check if this is an exact match (not a prefix of another script name)
                    start_pos = match.start()
                    end_pos = match.end()
                    
                    # Check character after the script name
                    if end_pos < len(content):
                        next_char = content[end_pos]
                        if next_char not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_:-':
                            references.add(script_name)
                    else:
                        # End of file, counts as exact match
                        references.add(script_name)
        except (IOError, UnicodeDecodeError):
            continue
    
    return references


def find_script_references_in_scripts(package_data, prefix):
    """Find references to scripts in other scripts' commands"""
    references = set()
    scripts = package_data.get('scripts', {})
    
    for script_name, command in scripts.items():
        if isinstance(command, str):
            pattern = re.compile(rf'npm\s+run\s+([a-zA-Z0-9_:-]+)')
            for match in pattern.finditer(command):
                referenced_script = match.group(1)
                # Check if this is an exact match
                start_pos = match.start()
                end_pos = match.end()
                
                if end_pos < len(command):
                    next_char = command[end_pos]
                    if next_char not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_:-':
                        references.add(referenced_script)
                else:
                    references.add(referenced_script)
    
    return references


def main():
    args = parse_arguments()
    
    # Load package.json
    package_data = load_package_json(args.package)
    if package_data is None:
        sys.exit(2)
    
    # Extract script names with the given prefix
    script_names = extract_script_names(package_data, args.prefix)
    if not script_names:
        print("unrun: 0 of 0 check:* scripts")
        sys.exit(0)
    
    # Find references in workflow files
    workflow_references = find_script_references(args.workflows, args.prefix)
    
    # Find references in other scripts
    script_references = find_script_references_in_scripts(package_data, args.prefix)
    
    # Combine all references
    all_references = workflow_references.union(script_references)
    
    # Find unrun scripts
    unrun_scripts = [name for name in script_names if name not in all_references]
    unrun_scripts.sort()
    
    # Print results
    for script in unrun_scripts:
        print(script)
    
    if unrun_scripts:
        print(f"unrun: {len(unrun_scripts)} of {len(script_names)} {args.prefix}* scripts")
        print("NOT ESTABLISHED: A script run by some other route (a Makefile, a shell script, a person) is still listed as unrun.")
        sys.exit(1)
    else:
        print(f"unrun: 0 of {len(script_names)} {args.prefix}* scripts")
        sys.exit(0)


if __name__ == '__main__':
    main()