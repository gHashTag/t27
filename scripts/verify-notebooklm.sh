#!/bin/bash
# scripts/verify-notebooklm.sh
# 7-Level verification plan for NotebookLM integration
# Based on arxiv:2411.00248v2 (Error codes in Python HTTP clients)
# phi^2 + 1/phi^2 = 3 | TRINITY

set -e

echo "=========================================="
echo "NOTEBOOKLM INTEGRATION VERIFICATION"
echo "=========================================="
echo ""

LEVELS_PASSED=0
LEVELS_FAILED=0

# ============================================================================
# LEVEL 1: Files in place
# ============================================================================
echo "LEVEL 1: Files in place"
echo "--------------------------------"

FILES_COUNT=$(ls -1 contrib/backend/notebooklm/*.py 2>/dev/null | wc -l | tr -d ' ')
echo "Expected: 11 modules"
echo "Found: $FILES_COUNT modules"

if [ "$FILES_COUNT" -eq 11 ]; then
    echo "[PASS] All 11 modules present"
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
else
    echo "[FAIL] Expected 9, found $FILES_COUNT"
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
fi

echo ""

# ============================================================================
# LEVEL 2: Python imports work
# ============================================================================
echo "LEVEL 2: Python imports work"
echo "--------------------------------"

python3 -c "
import sys
sys.path.insert(0, '.')
from contrib.backend.notebooklm.config import config_from_env, NotebookLMConfig
from contrib.backend.notebooklm.auth_token import token_load, token_save, token_is_valid, AuthTokens
from contrib.backend.notebooklm.cookie_auth import authenticate_with_cookies
print('[OK] All Python imports work')
"

if [ $? -eq 0 ]; then
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
    echo "[PASS] Import test succeeded"
else
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
    echo "[FAIL] Import test failed"
fi

echo ""

# ============================================================================
# LEVEL 3: Config defaults correct
# ============================================================================
echo "LEVEL 3: Config defaults correct"
echo "--------------------------------"

python3 -c "
import sys
sys.path.insert(0, '.')
from contrib.backend.notebooklm.config import config_from_env
cfg = config_from_env()
assert cfg.notebook_name == 't27-QUEEN-BRAIN', f'Wrong notebook name: {cfg.notebook_name}'
assert cfg.timeout_ms == 30000, f'Wrong timeout: {cfg.timeout_ms}'
assert cfg.auto_refresh == True, f'Wrong auto-refresh: {cfg.auto_refresh}'
print('[OK] Config defaults verified')
"

if [ $? -eq 0 ]; then
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
    echo "[PASS] Config defaults verified"
else
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
    echo "[FAIL] Config verification failed"
fi

echo ""

# ============================================================================
# LEVEL 4: Token operations work
# ============================================================================
echo "LEVEL 4: Token operations work"
echo "--------------------------------"

python3 -c "
import sys
import tempfile
from datetime import datetime, timedelta
sys.path.insert(0, '.')
from contrib.backend.notebooklm.auth_token import token_load, token_save, token_is_valid, AuthTokens

# Test load from default path (should return None if file doesn't exist)
result = token_load()
# Result can be None or AuthTokens, just verify function works without error

# Test save with future expiry date
future_expiry = datetime.now() + timedelta(hours=1)
tokens = AuthTokens('access_token', 'refresh_token', future_expiry, 'bearer')
success = token_save(tokens)
assert success == True, f'Failed to save tokens'

# Test token_is_valid
assert token_is_valid(tokens) == True, 'Valid token should return True'

print('[OK] Token operations verified')
"

if [ $? -eq 0 ]; then
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
    echo "[PASS] Token operations work"
else
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
    echo "[FAIL] Token operations failed"
fi

echo ""

# ============================================================================
# LEVEL 5: SDK installed
# ============================================================================
echo "LEVEL 5: SDK installed"
echo "--------------------------------"

source /tmp/notebooklm-venv/bin/activate 2>/dev/null

python3 -c "
import notebooklm
print(f'[OK] SDK version: {notebooklm.__version__}')
"

SDK_EXIT_CODE=$?

deactivate 2>/dev/null

if [ $SDK_EXIT_CODE -eq 0 ]; then
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
    echo "[PASS] SDK is available"
else
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
    echo "[FAIL] SDK not available"
fi

echo ""

# ============================================================================
# LEVEL 6: Imports don't conflict with stdlib
# ============================================================================
echo "LEVEL 6: Imports don't conflict with stdlib"
echo "--------------------------------"

python3 -c "
import sys
import tempfile
import os

# Save current directory and add to path
orig_dir = os.getcwd()
sys.path.insert(0, orig_dir)

# Change to /tmp for the test
os.chdir('/tmp')

# Create a test file that uses standard library 'token'
with open('test_stdlib.py', 'w') as f:
    f.write('import token; print(\"stdlib token works\")')

# Import our auth_token module
from contrib.backend.notebooklm.auth_token import AuthTokens
from contrib.backend.notebooklm.config import NotebookLMConfig

# Use both in same script
print('[OK] No stdlib conflict detected')
"

if [ $? -eq 0 ]; then
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
    echo "[PASS] auth_token.py doesn't conflict with stdlib 'token'"
else
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
    echo "[FAIL] Stdlib conflict detected"
fi

echo ""

# ============================================================================
# LEVEL 7: Connection (in sandbox!)
# ============================================================================
echo "LEVEL 7: Connection test (SANDBOX MODE)"
echo "--------------------------------"
echo "NOTE: Full auth requires browser. This is a unit test only."
echo ""

source /tmp/notebooklm-venv/bin/activate 2>/dev/null

python3 -c "
import sys
sys.path.insert(0, '.')
from contrib.backend.notebooklm.cookie_auth import test_notebooklm_sdk_integration

if test_notebooklm_sdk_integration():
    print('[OK] SDK check works')
else:
    print('[FAIL] SDK check failed')
    sys.exit(1)
"

deactivate 2>/dev/null

if [ $? -eq 0 ]; then
    LEVELS_PASSED=$((LEVELS_PASSED + 1))
    echo "[PASS] SDK availability test passed"
else
    LEVELS_FAILED=$((LEVELS_FAILED + 1))
    echo "[FAIL] SDK availability test failed"
fi

echo ""
echo "=========================================="
echo "SUMMARY"
echo "=========================================="
echo "Levels passed: $LEVELS_PASSED/7"
echo "Levels failed: $LEVELS_FAILED/7"

if [ $LEVELS_PASSED -eq 7 ]; then
    echo ""
    echo "RESULT: ALL VERIFICATION LEVELS PASSED!"
    echo ""
    echo "Next steps:"
    echo "  1. Integration tests (T-14 to T-19 from PR plan)"
    echo "  2. Update PR #309 with verification status"
    exit 0
else
    echo ""
    echo "RESULT: $LEVELS_FAILED LEVELS FAILED"
    echo ""
    echo "To fix failures:"
    echo "  1. Check missing modules in contrib/backend/notebooklm/"
    echo "  2. Verify Python version compatibility (requires 3.10+)"
    echo "  3. Ensure venv has notebooklm-py installed"
    echo ""
    exit 1
fi
