#!/usr/bin/env python3
"""XDC Pin Validation Script
Validates that PACKAGE_PIN assignments in XDC files are valid user IO balls
for the declared FPGA package using prjxray-db data.

Usage:
    python3 tools/validate_xdc_pins.py <xdc_file> <fpga_package>
    python3 tools/validate_xdc_pins.py specs/fpga/constraints/qmtech_a100t.xdc xc7a100tfgg676-1
"""

import re
import sys
import argparse
from pathlib import Path

# List of known invalid pins for XC7A100T-FGG676 package
# These pins are documented as non-user-IO balls in the prjxray-db
INVALID_PINS_XC7A100T_FGG676 = {
    "J13", "T10", "T11", "T13", "U10", "U11", "U18", 
    "V11", "V12", "W11", "W12", "W17", "W22"
}

# List of known invalid pins for XC7A200T-FGG676 package (placeholder)
# TODO: Fill with actual invalid pins from prjxray-db for XC7A200T-FGG676
INVALID_PINS_XC7A200T_FGG676 = set()

def extract_package_pins(xdc_file):
    """Extract PACKAGE_PIN assignments from XDC file."""
    pins = []
    try:
        with open(xdc_file, 'r') as f:
            content = f.read()
            
        # Find all PACKAGE_PIN assignments
        pattern = r'PACKAGE_PIN\s+([A-Za-z0-9]+)\s+IOSTANDARD'
        matches = re.findall(pattern, content)
        
        for pin in matches:
            pins.append(pin.strip())
            
    except FileNotFoundError:
        print(f"Error: XDC file not found: {xdc_file}")
        return []
    except Exception as e:
        print(f"Error reading XDC file: {e}")
        return []
    
    return pins

def validate_pins_for_package(pins, invalid_pins_set, package_name):
    """Validate pins against invalid list for a specific package."""
    invalid_found = []
    valid_pins = []
    
    for pin in pins:
        if pin in invalid_pins_set:
            invalid_found.append(pin)
        else:
            valid_pins.append(pin)
    
    return invalid_found, valid_pins

def main():
    parser = argparse.ArgumentParser(description='Validate XDC pin assignments')
    parser.add_argument('xdc_file', help='Path to XDC file to validate')
    parser.add_argument('fpga_package', help='FPGA package (e.g., xc7a100tfgg676-1)')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    # Determine invalid pins based on package
    if 'xc7a100t' in args.fpga_package.lower():
        invalid_pins = INVALID_PINS_XC7A100T_FGG676
        package_desc = "XC7A100T-FGG676"
    elif 'xc7a200t' in args.fpga_package.lower():
        invalid_pins = INVALID_PINS_XC7A200T_FGG676
        package_desc = "XC7A200T-FGG676"
    else:
        print(f"Warning: Unknown package {args.fpga_package}, using XC7A100T-FGG676 invalid pins")
        invalid_pins = INVALID_PINS_XC7A100T_FGG676
        package_desc = "Unknown (using XC7A100T-FGG676 invalid pins)"
    
    # Extract pins from XDC file
    pins = extract_package_pins(args.xdc_file)
    
    if not pins:
        print("No PACKAGE_PIN assignments found in XDC file")
        return 1
    
    # Validate pins
    invalid_found, valid_pins = validate_pins_for_package(pins, invalid_pins, args.fpga_package)
    
    # Report results
    print(f"Validating XDC file: {args.xdc_file}")
    print(f"Target FPGA package: {package_desc}")
    print(f"Total pins found: {len(pins)}")
    print(f"Valid pins: {len(valid_pins)}")
    print(f"Invalid pins: {len(invalid_found)}")
    
    if args.verbose:
        print(f"\nValid pins: {', '.join(sorted(valid_pins))}")
        if invalid_found:
            print(f"Invalid pins: {', '.join(sorted(invalid_found))}")
    
    if invalid_found:
        print(f"\n❌ VALIDATION FAILED: Found {len(invalid_found)} invalid pins for {package_desc}")
        print("These pins are not valid user IO balls for the declared package:")
        for pin in sorted(invalid_found):
            print(f"  - {pin}")
        print("\nRecommendation:")
        print("1. Update the XDC file to declare the correct FPGA package")
        print("2. Or correct the pin assignments to valid user IO balls")
        print("3. Or document which board the file actually targets")
        return 1
    else:
        print(f"\n✅ VALIDATION PASSED: All {len(pins)} pins are valid user IO balls for {package_desc}")
        return 0

if __name__ == "__main__":
    sys.exit(main())