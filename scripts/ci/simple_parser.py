#!/usr/bin/env python3
"""Simple pattern matcher for workflow files."""

import re

def load(path):
    """Simple pattern-based loader for workflow files."""
    with open(path) as fh:
        content = fh.read()
    
    # Simple pattern matching to find pull_request with branches
    doc = {}
    
    # Find on: block
    on_match = re.search(r'on:\s*\n(.*?)(?=\n\w+|\Z)', content, re.DOTALL)
    if on_match:
        on_content = on_match.group(1)
        doc['on'] = {}
        
        # Find pull_request and pull_request_target
        pr_pattern = r'(pull_request(_target)?)\s*:\s*\n(.*?)(?=\n\w+:\s*\n|\Z)'
        for pr_match in re.finditer(pr_pattern, on_content, re.DOTALL):
            pr_type = pr_match.group(1)
            pr_content = pr_match.group(3)
            
            # Check for branches filter
            branches_match = re.search(r'branches:\s*\[\s*([^]]+)\s*\]', pr_content, re.DOTALL)
            if branches_match:
                branches = branches_match.group(1)
                if pr_type not in doc['on']:
                    doc['on'][pr_type] = {}
                # Parse branches list
                branch_list = []
                for branch in branches.split(','):
                    branch = branch.strip().strip('"\'')
                    if branch:
                        branch_list.append(branch)
                doc['on'][pr_type]['branches'] = branch_list
    
    return doc