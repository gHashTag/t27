#!/usr/bin/env bash
# scripts/bulk-create-notebooks.sh — Create NotebookLM notebooks for all open issues
# phi^2 + 1/phi^2 = 3 | TRINITY

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Fetching open issues from gHashTag/t27..."

gh issue list --repo gHashTag/t27 --state open --json number,title --limit 100 | \
  python3.10 -c "
import json, sys, subprocess

issues = json.load(sys.stdin)
print(f'Found {len(issues)} open issues')

if len(issues) == 0:
    sys.exit(0)

for issue in issues:
    num = issue['number']
    title = issue['title']
    print(f'\\nCreating notebook for Issue #{num}: {title}')
    result = subprocess.run([
        'python3.10', 'contrib/backend/notebooklm/create_notebook.py',
        '--title', f'Issue #{num}: {title}',
        '--issue', str(num)
    ], capture_output=True, text=True)
    
    if result.returncode == 0:
        notebook_id = result.stdout.strip()
        print(f'  ✅ Notebook ID: {notebook_id}')
        # Comment to issue
        comment = f'📓 **NotebookLM Notebook created**\\n\\nNotebook ID: \`{notebook_id}\`\\nURL: https://notebooklm.google.com/notebook/{notebook_id}'
        subprocess.run([
            'gh', 'issue', 'comment', str(num),
            '--repo', 'gHashTag/t27',
            '--body', comment
        ], capture_output=True)
        print(f'  💬 Commented on issue #{num}')
    else:
        print(f'  ❌ Error: {result.stderr}')
        sys.exit(1)

print(f'\\n✅ All notebooks created for {len(issues)} issues')
"
