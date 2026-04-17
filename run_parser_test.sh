#!/bin/bash
# Simple test for the parser implementation
cd /Users/playra/t27

# Compile the parser as a library
rustc --edition 2021 --crate-name t27_bootstrap_parser \
  -L target/debug/deps \
  -o target/debug/libt27_bootstrap_parser.rlib \
  bootstrap/src/parser.rs

# Create test binary
rustc --edition 2021 -L target/debug/deps \
  --extern t27_bootstrap_parser \
  -o target/debug/parser_test \
  tests/parser_test.rs

# Run the test
echo "Running parser test..."
./target/debug/parser_test specs/00-gf-family-foundation.tri
