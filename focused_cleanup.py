#!/usr/bin/env python3
"""
Focused script to delete only the stale seals mentioned in the issue.
Based on the issue description, we should only delete old format files that have current new format twins.
"""

import json
import os
from pathlib import Path

def get_seal_info(seal_path):
    """Extract key information from a seal file."""
    try:
        with open(seal_path, 'r') as f:
            seal_data = json.load(f)
        
        filename = os.path.basename(seal_path)
        
        # Determine if this is old format (no directory prefix) or new format
        # New format: <dir>_<Module>.json
        # Old format: <Module>.json
        if '_' in filename and filename.endswith('.json'):
            # Check if there's exactly one underscore and it's not at the beginning or end
            parts = filename.split('_')
            if len(parts) == 2 and parts[1].endswith('.json'):
                # New format: <dir>_<Module>.json
                dir_prefix = parts[0]
                module_name = parts[1][:-5]  # Remove .json extension
                format_type = 'new'
            else:
                # Multiple underscores or other pattern - treat as old format
                format_type = 'old'
        else:
            # No underscore or doesn't end with .json - old format
            format_type = 'old'
        
        return {
            'path': seal_path,
            'filename': filename,
            'module': seal_data.get('module', ''),
            'spec_path': seal_data.get('spec_path', ''),
            'spec_hash': seal_data.get('spec_hash', ''),
            'sealed_at': seal_data.get('sealed_at', ''),
            'has_sealed_by': 'sealed_by' in seal_data,
            'format_type': format_type
        }
    except (json.JSONDecodeError, FileNotFoundError):
        return None

def find_old_format_with_new_twin(old_seal, all_seals):
    """Check if an old format seal has a new format twin with the same spec_path."""
    for seal in all_seals:
        if (seal['format_type'] == 'new' and 
            seal['spec_path'] == old_seal['spec_path'] and
            seal['module'] == old_seal['module']):
            return seal
    return None

def main():
    seals_dir = Path('.trinity/seals')
    seal_files = list(seals_dir.glob('*.json'))
    
    print(f"Found {len(seal_files)} seal files")
    
    # Load all seal info
    all_seals = []
    for seal_file in seal_files:
        info = get_seal_info(str(seal_file))
        if info:
            all_seals.append(info)
    
    print(f"Successfully loaded {len(all_seals)} seal files")
    
    # Count format types
    old_format = [s for s in all_seals if s['format_type'] == 'old']
    new_format = [s for s in all_seals if s['format_type'] == 'new']
    print(f"Old format seals: {len(old_format)}")
    print(f"New format seals: {len(new_format)}")
    
    # Show examples of each format
    print(f"\n=== OLD FORMAT EXAMPLES ===")
    for seal in old_format[:5]:
        print(f"  {seal['filename']} -> module: {seal['module']}, spec_path: {seal['spec_path']}")
    
    print(f"\n=== NEW FORMAT EXAMPLES ===")
    for seal in new_format[:5]:
        print(f"  {seal['filename']} -> module: {seal['module']}, spec_path: {seal['spec_path']}")
    
    # Find old format seals that have new format twins
    stale_to_delete = []
    current_kept = []
    
    print(f"\n=== ANALYZING PAIRS ===")
    for old_seal in old_format:
        twin = find_old_format_with_new_twin(old_seal, all_seals)
        if twin:
            # Check if the twin is current (has sealed_by)
            if twin['has_sealed_by']:
                stale_to_delete.append(old_seal)
                current_kept.append(twin)
                print(f"  ✓ Found pair: {old_seal['filename']} (stale) -> {twin['filename']} (current)")
            else:
                # Both are old format, keep the newer one
                if old_seal['sealed_at'] > twin['sealed_at']:
                    stale_to_delete.append(twin)
                    current_kept.append(old_seal)
                    print(f"  ✓ Found pair: {twin['filename']} (older) -> {old_seal['filename']} (newer)")
                else:
                    stale_to_delete.append(old_seal)
                    current_kept.append(twin)
                    print(f"  ✓ Found pair: {old_seal['filename']} (older) -> {twin['filename']} (newer)")
    
    print(f"\n=== FOCUSED CLEANUP RESULTS ===")
    print(f"Old format seals found: {len(old_format)}")
    print(f"New format seals found: {len(new_format)}")
    print(f"Stale seals to delete: {len(stale_to_delete)}")
    print(f"Current seals to keep: {len(current_kept)}")
    
    if len(stale_to_delete) == 0:
        print("No stale seals found that match the criteria.")
        return
    
    print(f"\n=== SEALS TO DELETE ===")
    for seal in stale_to_delete:
        print(f"  {seal['filename']} -> {seal['spec_path']}")
    
    # Proceed with deletion
    print(f"\nDeleting {len(stale_to_delete)} stale seals...")
    deleted_count = 0
    failed_count = 0
    
    for seal in stale_to_delete:
        try:
            os.remove(seal['path'])
            deleted_count += 1
            print(f"  ✓ Deleted: {seal['path']}")
        except OSError as e:
            failed_count += 1
            print(f"  ✗ Failed to delete {seal['path']}: {e}")
    
    print(f"\n=== FINAL RESULTS ===")
    print(f"Successfully deleted: {deleted_count}")
    print(f"Failed to delete: {failed_count}")
    print(f"Total processed: {deleted_count + failed_count}")
    
    if deleted_count > 0:
        print(f"\nStale seals have been removed. The seal coverage should now be improved.")
        print(f"Consider running 't27c seal --check' to verify the cleanup was successful.")

if __name__ == '__main__':
    main()