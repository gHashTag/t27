#!/usr/bin/env python3
"""
Script to identify stale seals for cleanup based on the issue description.
"""

import json
import os
import glob
from pathlib import Path

def load_seal_file(seal_path):
    """Load and parse a seal file."""
    try:
        with open(seal_path, 'r') as f:
            return json.load(f)
    except (json.JSONDecodeError, FileNotFoundError):
        return None

def get_seal_info(seal_path):
    """Extract key information from a seal file."""
    seal_data = load_seal_file(seal_path)
    if not seal_data:
        return None
    
    # Extract directory prefix from filename
    filename = os.path.basename(seal_path)
    if '_' in filename and not filename.endswith('.json'):
        # This is the new format: <dir>_<Module>.json
        dir_prefix = filename.split('_')[0]
    else:
        # This is the old format: <Module>.json
        dir_prefix = None
    
    return {
        'path': seal_path,
        'filename': filename,
        'module': seal_data.get('module', ''),
        'spec_path': seal_data.get('spec_path', ''),
        'spec_hash': seal_data.get('spec_hash', ''),
        'sealed_at': seal_data.get('sealed_at', ''),
        'dir_prefix': dir_prefix,
        'has_sealed_by': 'sealed_by' in seal_data
    }

def find_seal_pairs(seals_info):
    """Find pairs of seals that represent the same module."""
    pairs = []
    processed = set()
    
    for i, seal1 in enumerate(seals_info):
        if seal1 is None or seal1['path'] in processed:
            continue
            
        # Find potential twin
        twin = None
        for j, seal2 in enumerate(seals_info):
            if i == j or seal2 is None or seal2['path'] in processed:
                continue
                
            # Check if they have the same module and spec_path
            if (seal1['module'] == seal2['module'] and 
                seal1['spec_path'] == seal2['spec_path']):
                twin = seal2
                break
        
        if twin:
            pairs.append((seal1, twin))
            processed.add(seal1['path'])
            processed.add(twin['path'])
        else:
            # No twin found, mark as single
            processed.add(seal1['path'])
            pairs.append((seal1, None))
    
    return pairs

def classify_seal_pair(seal1, seal2):
    """Classify a pair of seals to determine which one is stale."""
    if seal2 is None:
        return 'single', seal1
    
    # Determine which one is newer/newer format
    if seal1['has_sealed_by'] and not seal2['has_sealed_by']:
        # seal1 is newer format, seal2 is older
        return 'stale_old', seal2, seal1
    elif seal2['has_sealed_by'] and not seal1['has_sealed_by']:
        # seal2 is newer format, seal1 is older  
        return 'stale_old', seal1, seal2
    elif seal1['has_sealed_by'] and seal2['has_sealed_by']:
        # Both are new format, compare timestamps
        if seal1['sealed_at'] > seal2['sealed_at']:
            return 'stale_older', seal2, seal1
        else:
            return 'stale_older', seal1, seal2
    else:
        # Both are old format, compare timestamps
        if seal1['sealed_at'] > seal2['sealed_at']:
            return 'stale_older', seal2, seal1
        else:
            return 'stale_older', seal1, seal2

def main():
    seals_dir = Path('.trinity/seals')
    
    # Load all seal files
    seal_files = glob.glob(str(seals_dir / '*.json'))
    seals_info = []
    
    print(f"Found {len(seal_files)} seal files")
    
    for seal_file in seal_files:
        info = get_seal_info(seal_file)
        if info:
            seals_info.append(info)
    
    print(f"Successfully loaded {len(seals_info)} seal files")
    
    # Find pairs
    pairs = find_seal_pairs(seals_info)
    print(f"Found {len(pairs)} seal pairs/groups")
    
    # Classify pairs
    stale_to_delete = []
    current_kept = []
    singles = []
    
    for pair in pairs:
        classification = classify_seal_pair(*pair)
        
        if classification[0] == 'single':
            singles.append(classification[1])
        elif classification[0] == 'stale_old':
            stale, current = classification[1], classification[2]
            stale_to_delete.append(stale)
            current_kept.append(current)
        elif classification[0] == 'stale_older':
            stale, current = classification[1], classification[2]
            stale_to_delete.append(stale)
            current_kept.append(current)
    
    print(f"\n=== ANALYSIS RESULTS ===")
    print(f"Total seals analyzed: {len(seals_info)}")
    print(f"Pairs found: {len([p for p in pairs if p[1] is not None])}")
    print(f"Singles found: {len(singles)}")
    print(f"Stale seals to delete: {len(stale_to_delete)}")
    print(f"Current seals to keep: {len(current_kept)}")
    
    print(f"\n=== SEALS TO DELETE ===")
    for seal in stale_to_delete:
        print(f"  {seal['filename']} -> {seal['spec_path']}")
    
    print(f"\n=== SEALS TO KEEP ===")
    for seal in current_kept:
        print(f"  {seal['filename']} -> {seal['spec_path']}")
    
    print(f"\n=== SINGLES (NO TWIN) ===")
    for seal in singles:
        print(f"  {seal['filename']} -> {seal['spec_path']}")
    
    # Save results for later use
    results = {
        'total_seals': len(seals_info),
        'pairs': len([p for p in pairs if p[1] is not None]),
        'singles': len(singles),
        'stale_to_delete': [{'path': s['path'], 'filename': s['filename'], 'spec_path': s['spec_path']} for s in stale_to_delete],
        'current_kept': [{'path': s['path'], 'filename': s['filename'], 'spec_path': s['spec_path']} for s in current_kept],
        'singles': [{'path': s['path'], 'filename': s['filename'], 'spec_path': s['spec_path']} for s in singles]
    }
    
    with open('seal_cleanup_analysis.json', 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\nAnalysis saved to seal_cleanup_analysis.json")
    
    return stale_to_delete, current_kept, singles

if __name__ == '__main__':
    main()