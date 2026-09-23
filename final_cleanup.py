#!/usr/bin/env python3
"""Final cleanup of remaining stale seals with current twins."""

import json
import os
from pathlib import Path

SEALS = Path(".trinity/seals")

def final_cleanup():
    """Remove remaining old format files that have current twins."""
    old_format_files = []
    
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
    
    print(f"Found {len(old_format_files)} old format files")
    
    files_to_delete = []
    
    for old_name, old_data in old_format_files:
        # Extract module name from old format
        module_name = old_name.replace(".json", "")
        
        # Look for corresponding new format file
        matching_new = [name for name in SEALS.glob("*.json") if name.name.endswith(f"_{module_name}.json")]
        
        if matching_new:
            new_file = matching_new[0]
            with open(new_file, 'r') as f:
                try:
                    new_data = json.load(f)
                except json.JSONDecodeError:
                    continue
            
            # Check if they have the same spec_path and spec_hash
            if old_data.get("spec_path") == new_data.get("spec_path") and old_data.get("spec_hash") == new_data.get("spec_hash"):
                files_to_delete.append(old_name)
                print(f"Marked for deletion: {old_name} (twin: {new_file.name})")
    
    print(f"\nWill delete {len(files_to_delete)} files:")
    for filename in files_to_delete:
        print(f"  {filename}")
    
    # Actually delete the files
    deleted_count = 0
    for filename in files_to_delete:
        file_path = SEALS / filename
        if file_path.exists():
            file_path.unlink()
            deleted_count += 1
            print(f"Deleted: {filename}")
    
    print(f"\nSuccessfully deleted {deleted_count} files")
    return deleted_count

if __name__ == "__main__":
    if SEALS.is_dir():
        final_cleanup()
    else:
        print(f"Seals directory {SEALS} not found")