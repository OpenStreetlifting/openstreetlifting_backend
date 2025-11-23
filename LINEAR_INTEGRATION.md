# Linear + GitHub Integration Guide

## Setup

### Enable Linear GitHub Integration

1. Go to Linear Settings > Integrations > GitHub
2. Connect your GitHub account and select this repository
3. Enable two-way sync
4. Configure auto-labeling: Feature → `feature`, Bug → `bug`, Improvement → `enhancement`
5. Enable auto-close on merge to main

## PR Title Convention

Use this format: `[OSL-123] Add user authentication`

Where `OSL-123` is your Linear issue ID.

This enables:
- Automatic PR linking in Linear
- Issue auto-close on merge
- Linear references in release notes

## Workflow

1. Create issue in Linear (e.g., OSL-45)
2. Create PR with title: `[OSL-45] Description`
3. Linear auto-applies labels based on issue type
4. Merge to main
5. Linear auto-closes issue
6. Next release includes change with Linear reference

## Labels

- `feature` - New features
- `enhancement` - Improvements
- `bug` / `bugfix` / `fix` - Bug fixes
- `breaking-change` - Breaking changes
- `documentation` / `docs` - Documentation
- `chore` / `maintenance` - Maintenance

## Multiple Issues

Format: `[OSL-123][OSL-124] Description`

## No Linear Issue

PRs without Linear IDs still appear in changelog, just without issue reference.
