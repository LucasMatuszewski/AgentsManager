# ADR-0001 — Upstream Tracking Strategy

## Status
Accepted

## Context
AgentsManager is a fork of CodexMonitor. We want to keep the original upstream accessible for reference and selective cherry‑picks, while letting the fork evolve independently.

## Decision
- Add `upstream` remote pointing to `https://github.com/Dimillian/CodexMonitor`.
- Create a dedicated branch `codex-monitor` that **tracks** `upstream/main`.
- Keep `main` tracking `origin/main` (our fork) for all product work.
- When needed, cherry‑pick commits from `codex-monitor` into `main` or feature branches.

## Rationale
- Preserves a clean upstream reference branch.
- Avoids accidental merges of upstream into product main.
- Enables selective adoption of upstream fixes.

## Commands
```bash
# One‑time setup
git remote add upstream https://github.com/Dimillian/CodexMonitor.git
git fetch upstream
git checkout -b codex-monitor --track upstream/main

# Update upstream tracking branch
git checkout codex-monitor
git merge upstream/main --ff-only

# Bring changes into main
git checkout main
git cherry-pick <commit>
```

## Consequences
- Upstream changes must be reviewed and cherry‑picked intentionally.
- Developers should avoid committing on `codex-monitor`.
