#!/bin/bash
# Test script for math_compare_gamma_nodeps.rs

echo "=== Testing math_compare_gamma_nodeps.rs (nodeps version) ==="
cd bootstrap
cargo run --manifest-path bootstrap/Cargo.toml --manifest-path bootstrap/src/Cargo.toml -- math compare --gamma 2>&1
echo "Exit code: $?"
