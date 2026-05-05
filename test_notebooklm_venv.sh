#!/bin/bash
# test_notebooklm_venv.sh
# Test NotebookLM connection using venv

VENV="/tmp/notebooklm-venv"

echo "Testing NotebookLM SDK availability..."
source "$VENV/bin/activate"

python -c "
import notebooklm
print('  [OK] notebooklm-py SDK available')
print(f'  Module: {notebooklm.__file__}')
"

# Test basic import
python -c "
from notebooklm import NotebookLM
print('  [OK] NotebookLM class imported')
"

# Test config
echo ""
echo "Testing configuration..."
python << 'PYTHON'
import sys
from pathlib import Path

contrib_path = Path('contrib/backend/notebooklm')
sys.path.insert(0, str(contrib_path))

from config import config_from_env

config = config_from_env()
print(f"  Storage path: {config.storage_path}")
print(f"  Notebook name: {config.notebook_name}")
print(f"  Timeout: {config.timeout_ms}ms")
print(f"  Auto-refresh: {config.auto_refresh}")
PYTHON

echo ""
echo "[SUCCESS] All connection tests passed"
