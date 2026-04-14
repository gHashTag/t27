# GitHub SSOT Integration - .t27 Native

## Overview

The Single Source of Truth (SSOT) integration brings GitHub-native support to the `.t27` specification format.

## Features

### .t27 File Format
- Native `.t27` file support in GitHub
- Syntax highlighting
- PR integration
- Issue tracking

### GitHub Actions Integration
- Automatic .t27 validation
- CI pipeline integration
- Automated testing

## Workflow

1. **Edit .t27 spec** - Make changes to `.t27` specification files
2. **Push to GitHub** - CI automatically validates the spec
3. **Review PR** - Changes are shown with proper diff
4. **Merge** - Validated changes merge to master

## Implementation

### Files Modified
- `.github/workflows/phi-loop-ci.yml` - CI pipeline
- `.t27` file handlers - Native GitHub support

## Benefits

- No intermediate files (direct .t27 editing)
- Faster iteration (no regeneration step)
- Better review experience (GitHub native diffs)

## Related Issues

- Issue #338: GitHub SSOT integration - .t27 Native
