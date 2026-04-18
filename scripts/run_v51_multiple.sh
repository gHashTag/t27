#!/bin/bash
# Запустить v51 N=10 раз для получения >1000 формул
echo "Running v51 multiple times..."
for i in {1..10}; do
    echo "Run $i/10"
    python3 /Users/playra/t27/scripts/ultra_engine_v51.py --all --threshold 0.05 --quiet
    echo ""
done
echo "All runs completed!"
chmod +x /Users/playra/t27/scripts/run_v51_multiple.sh
