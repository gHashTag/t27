#!/bin/bash

# Test script to verify exit code changes for gHashTag/t27#7334
# This script should be run from the cli/tri directory after building with cargo build --release

set -euo pipefail

TRI_BINARY="./target/release/tri"

if [ ! -f "$TRI_BINARY" ]; then
    echo "Error: $TRI_BINARY not found. Run 'cargo build --release' first."
    exit 1
fi

echo "Testing exit codes for tri binary..."

# Test 1: --help should exit with code 0
echo "Test 1: tri --help should exit 0"
if "$TRI_BINARY" --help >/dev/null 2>&1; then
    echo "PASS: tri --help exited 0"
elif [ $? -eq 0 ]; then
    echo "PASS: tri --help exited 0"
else
    echo "FAIL: tri --help did not exit 0"
    exit 1
fi

# Test 2: Unknown flag should exit with code 1 (not 2)
echo "Test 2: tri mutate spec --no-such-flag should exit 1"
if "$TRI_BINARY" mutate spec --no-such-flag >/dev/null 2>&1; then
    echo "FAIL: tri mutate spec --no-such-flag should have failed"
    exit 1
else
    exit_code=$?
    if [ $exit_code -eq 1 ]; then
        echo "PASS: tri mutate spec --no-such-flag exited 1"
    else
        echo "FAIL: tri mutate spec --no-such-flag exited $exit_code, expected 1"
        exit 1
    fi
fi

# Test 3: Unknown subcommand should exit with code 1 (not 2)
echo "Test 3: tri mutate no-such-subcommand should exit 1"
if "$TRI_BINARY" mutate no-such-subcommand >/dev/null 2>&1; then
    echo "FAIL: tri mutate no-such-subcommand should have failed"
    exit 1
else
    exit_code=$?
    if [ $exit_code -eq 1 ]; then
        echo "PASS: tri mutate no-such-subcommand exited 1"
    else
        echo "FAIL: tri mutate no-such-subcommand exited $exit_code, expected 1"
        exit 1
    fi
fi

echo "All tests passed!"