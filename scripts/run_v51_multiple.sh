#!/bin/bash
# Repository root from this script's location, not from one machine.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Запустить v51 N=10 раз для получения >1000 формул
echo "Running v51 multiple times..."
for i in {1..10}; do
    echo "Run $i/10"
    python3 "$ROOT"/scripts/ultra_engine_v51.py --all --threshold 0.05 --quiet
    echo ""
done
echo "All runs completed!"
chmod +x "$ROOT"/scripts/run_v51_multiple.sh
