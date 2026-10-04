#!/usr/bin/env bash
# Test script for pre-commit hook behavior with different tri scenarios
# This validates the fix for gHashTag/t27#6009

set -euo pipefail

# Create temporary directory for test
TEMP_DIR=$(mktemp -d)
echo "=== Testing pre-commit hook behavior ==="
echo "Working in: $TEMP_DIR"

# Copy repository structure to test directory
REPO_ROOT="/workspace/t27/.worktrees/queen-6009"
cp -r "$REPO_ROOT/.githooks" "$TEMP_DIR/"
cp -r "$REPO_ROOT/tools" "$TEMP_DIR/"
cd "$TEMP_DIR"

# Set up test repository
git init -q
git config user.email "test@example.com"
git config user.name "Test User"

# Create a simple file to commit
echo "test content" > test.txt
git add test.txt

echo ""
echo "=== Test 1: No tri binary available ==="
echo "Testing with no tri binary on PATH..."

# Ensure no tri is available by setting minimal PATH
export PATH="/usr/bin:/bin"
git commit -m "test commit no tri" 2>&1 || true
echo "Exit code: $?"

echo ""
echo "=== Test 2: Fake foreign tri (fails hooks --help) ==="
echo "Testing with fake tri that doesn't support hooks..."

# Create fake tri that fails hooks --help
mkdir -p bin
cat > bin/tri << 'EOF'
#!/bin/bash
if [[ "$1" == "hooks" && "$2" == "--help" ]]; then
    exit 1
fi
echo "fake tri: unknown command: $1"
exit 1
EOF
chmod +x bin/tri
export PATH="$PATH:$(pwd)/bin"

# Copy our hook to test
git config core.hooksPath .githooks

echo "Creating staged file..."
echo "content with potential conflict <<<<<<< HEAD" > conflict.txt
git add conflict.txt

git commit -m "test commit fake tri" 2>&1 || true
echo "Exit code: $?"

echo ""
echo "=== Test 3: Fake genuine tri (passes hooks --help) ==="
echo "Testing with fake tri that supports hooks..."

# Create fake tri that passes hooks --help
cat > bin/tri << 'EOF'
#!/bin/bash
if [[ "$1" == "hooks" && "$2" == "--help" ]]; then
    echo "Usage: tri hooks <command>"
    echo "Commands:"
    echo "  pre-commit    Run pre-commit hooks"
    exit 0
fi
if [[ "$1" == "hooks" && "$2" == "pre-commit" ]]; then
    echo "fake tri: pre-commit hooks not implemented"
    exit 2
fi
echo "fake tri: unknown command: $1"
exit 1
EOF
chmod +x bin/tri

git commit -m "test commit fake genuine tri" 2>&1 || true
echo "Exit code: $?"

echo ""
echo "=== Test 4: Real conflict marker detection ==="
echo "Testing conflict marker detection with no tri..."

# Remove fake tri to test no tri scenario
rm bin/tri
export PATH="/usr/bin:/bin"

# Create a file with actual conflict markers
cat > conflicted_file.py << 'EOF'
def test_function():
    x = 1
    <<<<<<< HEAD
    x = 2
    =======
    x = 3
    >>>>>>> branch
    return x
EOF
git add conflicted_file.py

git commit -m "test commit with conflict" 2>&1 || true
echo "Exit code: $?"

echo ""
echo "=== Test 5: Clean file (should pass conflict check) ==="
echo "Testing clean file with no tri..."

# Create a clean file
cat > clean_file.py << 'EOF'
def clean_function():
    return "no conflicts here"
EOF
git add clean_file.py

git commit -m "test commit clean file" 2>&1 || true
echo "Exit code: $?"

echo ""
echo "=== Test completed ==="
echo "Cleaning up..."
cd /
rm -rf "$TEMP_DIR"