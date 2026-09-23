#!/usr/bin/env python3
"""
Script to delete stale seals based on the analysis.
"""

import json
import os
from pathlib import Path

def load_analysis():
    """Load the analysis results."""
    try:
        with open('seal_cleanup_analysis.json', 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print("Error: seal_cleanup_analysis.json not found. Run analyze_seals_for_cleanup.py first.")
        return None

def confirm_deletion(stale_seals):
    """Confirm with user before deleting seals."""
    print(f"\n=== SEAL CLEANUP PLAN ===")
    print(f"Found {len(stale_seals)} stale seals to delete")
    
    print(f"\nFirst 10 seals to delete:")
    for i, seal in enumerate(stale_seals[:10]):
        print(f"  {i+1}. {seal['filename']} -> {seal['spec_path']}")
    
    if len(stale_seals) > 10:
        print(f"  ... and {len(stale_seals) - 10} more")
    
    print(f"\nLast 10 seals to delete:")
    for i, seal in enumerate(stale_seals[-10:]):
        print(f"  {i+1}. {seal['filename']} -> {seal['spec_path']}")
    
    response = input(f"\nDelete these {len(stale_seals)} stale seals? (y/N): ")
    return response.lower() == 'y'

def delete_seal(seal_path):
    """Delete a seal file."""
    try:
        os.remove(seal_path)
        return True
    except FileNotFoundError:
        print(f"Warning: Seal file not found: {seal_path}")
        return False
    except OSError as e:
        print(f"Error deleting seal file {seal_path}: {e}")
        return False

def main():
    # Load analysis
    analysis = load_analysis()
    if not analysis:
        return
    
    # Extract stale seals to delete
    stale_seals = analysis.get('stale_to_delete', [])
    
    if not stale_seals:
        print("No stale seals found to delete.")
        return
    
    # Confirm deletion (automatically proceed for this script)
    print(f"\nAutomatically proceeding to delete {len(stale_seals)} stale seals...")
    # For automated execution, we'll proceed without prompting
    # if confirm_deletion(stale_seals):
    
    # Delete seals
    print(f"\nDeleting {len(stale_seals)} stale seals...")
    deleted_count = 0
    failed_count = 0
    
    for seal in stale_seals:
        seal_path = seal['path']
        if delete_seal(seal_path):
            deleted_count += 1
            print(f"  ✓ Deleted: {seal_path}")
        else:
            failed_count += 1
    
    print(f"\n=== CLEANUP RESULTS ===")
    print(f"Successfully deleted: {deleted_count}")
    print(f"Failed to delete: {failed_count}")
    print(f"Total processed: {deleted_count + failed_count}")
    
    if deleted_count > 0:
        print(f"\nStale seals have been removed. The seal coverage should now be improved.")
        print(f"Consider running 't27c seal --check' to verify the cleanup was successful.")

if __name__ == '__main__':
    main()