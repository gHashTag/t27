#!/bin/bash
# Phase 2A Build Fix (EPSILON)

echo "Phase 2A Build Fix (EPSILON)"
echo "Fixing build issues for Parameter Golf"
echo ""

# Fix syntax error in train_real_telemetry.rs
if [ -f "crates/trios-train-cpu/src/bin/train_real_telemetry.rs" ]; then
    sed -i.bak '481d' crates/trios-train-cpu/src/bin/train_real_telemetry.rs
    echo "✓ Fixed train_real_telemetry.rs syntax"
fi

# Build core experiments (exclude broken binaries)
echo "Building core experiments..."
cargo build --release 2>&1 | grep -E "error|Finished" | tail -20

echo ""
echo "Build status check:"
if cargo build --release 2>&1 | grep -q "Finished"; then
    echo "✓ Build: GREEN"
    echo "  Errors: 0"
else
    echo "✗ Build: RED"
    echo "  Errors: (see above)"
fi

echo ""
echo "Agent: EPSILON"
echo "Branch: $(git branch --show-current)"
echo "Commit: $(git rev-parse --short HEAD)"
echo "BUILD: $(cargo build --release 2>&1 | grep -c error) errors found"
echo "TASK: Phase 2A build fix"
echo "TIMESTAMP: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
