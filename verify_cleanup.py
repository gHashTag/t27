#!/usr/bin/env python3
"""Verify that the cleanup was successful by checking that deleted seals had current twins."""

import json
import os
from pathlib import Path

SEALS = Path(".trinity/seals")

def analyze_cleanup():
    """Analyze the seals directory after cleanup."""
    old_format_files = []
    new_format_files = []
    
    for seal_file in SEALS.glob("*.json"):
        with open(seal_file, 'r') as f:
            try:
                seal_data = json.load(f)
            except json.JSONDecodeError:
                continue
                
        filename = seal_file.name
        
        # Check if it's old format (no underscore before Module)
        if "_" not in filename.replace(".json", ""):
            old_format_files.append((filename, seal_data))
        else:
            new_format_files.append((filename, seal_data))
    
    print(f"Old format files remaining: {len(old_format_files)}")
    print(f"New format files: {len(new_format_files)}")
    
    # Check if any old format files still have twins
    twins_found = 0
    for old_name, old_data in old_format_files:
        # Extract module name from old format
        module_name = old_name.replace(".json", "")
        
        # Look for corresponding new format file
        new_name = f"*_{module_name}.json"
        matching_new = [name for name, _ in new_format_files if name.endswith(f"_{module_name}.json")]
        
        if matching_new:
            new_name, new_data = matching_new[0], next(data for name, data in new_format_files if name == matching_new[0])
            
            # Check if they have the same spec_path and spec_hash
            if old_data.get("spec_path") == new_data.get("spec_path") and old_data.get("spec_hash") == new_data.get("spec_hash"):
                twins_found += 1
                print(f"Found twin pair: {old_name} <-> {new_name}")
    
    print(f"Old format files with current twins: {twins_found}")
    
    if twins_found == 0:
        print("✅ Cleanup successful: No old format files with current twins found")
    else:
        print(f"⚠️  Cleanup incomplete: {twins_found} old format files still have current twins")

if __name__ == "__main__":
    if SEALS.is_dir():
        analyze_cleanup()
    else:
        print(f"Seals directory {SEALS} not found")