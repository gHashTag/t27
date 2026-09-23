#!/usr/bin/env python3
"""
Analyze seals to identify duplicates and cleanup candidates.
"""

import json
import os
from pathlib import Path
from collections import defaultdict
import hashlib

def get_seal_info(seal_file):
    """Extract information from a seal file."""
    try:
        with open(seal_file, 'r') as f:
            data = json.load(f)
        
        # Get the filename without extension for the module name
        filename = os.path.basename(seal_file)
        module_name = filename.replace('.json', '')
        
        return {
            'file': seal_file,
            'filename': filename,
            'module_name': module_name,
            'spec_path': data.get('spec_path', ''),
            'spec_hash': data.get('spec_hash', ''),
            'sealed_at': data.get('sealed_at', ''),
            'sealed_by': data.get('sealed_by', ''),
            'is_old_format': not '_' in module_name,
            'is_new_format': '_' in module_name
        }
    except Exception as e:
        print(f"Error reading {seal_file}: {e}")
        return None

def get_current_spec_hash(spec_path):
    """Get the current hash of a spec file."""
    try:
        with open(spec_path, 'rb') as f:
            content = f.read()
            return hashlib.sha256(content).hexdigest()
    except Exception as e:
        print(f"Error reading {spec_path}: {e}")
        return None

def main():
    seals_dir = Path('.trinity/seals')
    
    if not seals_dir.exists():
        print(f"Seals directory {seals_dir} not found")
        return
    
    # Read all seals
    seals = []
    for seal_file in seals_dir.glob('*.json'):
        seal_info = get_seal_info(seal_file)
        if seal_info:
            seals.append(seal_info)
    
    print(f"Found {len(seals)} seal files")
    
    # Group by spec_path
    by_spec_path = defaultdict(list)
    for seal in seals:
        by_spec_path[seal['spec_path']].append(seal)
    
    print(f"\nSeals by spec_path:")
    for spec_path, seal_list in by_spec_path.items():
        if len(seal_list) > 1:
            print(f"{spec_path}: {len(seal_list)} seals")
    
    # Find duplicates
    duplicates = []
    cleanup_candidates = []
    
    for spec_path, seal_list in by_spec_path.items():
        if len(seal_list) <= 1:
            continue
            
        # Get current spec hash
        current_hash = get_current_spec_hash(spec_path)
        
        # Separate old and new format seals
        old_format = [s for s in seal_list if s['is_old_format']]
        new_format = [s for s in seal_list if s['is_new_format']]
        
        print(f"\n{spec_path}:")
        print(f"  Current spec hash: {current_hash}")
        print(f"  Old format seals: {len(old_format)}")
        print(f"  New format seals: {len(new_format)}")
        
        for seal in seal_list:
            is_current = seal['spec_hash'] == current_hash if current_hash else False
            print(f"    {seal['filename']}: {'CURRENT' if is_current else 'STALE'} (sealed: {seal['sealed_at']})")
        
        # Identify cleanup candidates
        # Prefer to keep new format seals if they're current
        current_seals = [s for s in seal_list if s['spec_hash'] == current_hash] if current_hash else []
        stale_seals = [s for s in seal_list if s['spec_hash'] != current_hash] if current_hash else seal_list
        
        if len(current_seals) == 1 and len(stale_seals) > 0:
            # We have one current seal and some stale ones
            print(f"  → Cleanup candidates: {len(stale_seals)} stale seals")
            cleanup_candidates.extend(stale_seals)
        elif len(current_seals) == 0 and len(seal_list) > 1:
            # All seals are stale, but we have duplicates - remove the older ones
            print(f"  → All seals stale, keeping newest")
            seal_list.sort(key=lambda s: s['sealed_at'], reverse=True)
            cleanup_candidates.extend(seal_list[1:])
    
    print(f"\n=== SUMMARY ===")
    print(f"Total cleanup candidates: {len(cleanup_candidates)}")
    
    # Show cleanup candidates by filename
    by_filename = defaultdict(list)
    for seal in cleanup_candidates:
        by_filename[seal['filename']].append(seal)
    
    print(f"\nCleanup candidates by filename:")
    for filename, seals in by_filename.items():
        print(f"  {filename}: {len(seals)} candidates")
    
    # Generate cleanup script
    cleanup_script = "#!/bin/bash\n"
    cleanup_script += "# Cleanup script for duplicate seals\n"
    cleanup_script += "# Generated automatically\n\n"
    
    for seal in cleanup_candidates:
        cleanup_script += f"# {seal['spec_path']}\n"
        cleanup_script += f"rm '{seal['file']}'\n"
        cleanup_script += f"# Ledger cleanup needed for: {seal['filename']}\n\n"
    
    with open('cleanup_seals.sh', 'w') as f:
        f.write(cleanup_script)
    
    print(f"\nCleanup script written to: cleanup_seals.sh")
    print(f"Would remove {len(cleanup_candidates)} seal files")

if __name__ == '__main__':
    main()