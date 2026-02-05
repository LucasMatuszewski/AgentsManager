# PRD — AgentsManager (CodexMonitor++)

## Objective
Build a cross‑platform Tauri app that orchestrates **multiple CLI agents** (Codex, Gemini CLI, Claude Code) via **ACP**, supporting local + remote workflows with clear UX for runner, workspace, and sandbox state.

## Problem
Current CodexMonitor is Codex‑only and hides critical context (which runner, which workspace, which sandbox). This causes wrong‑agent and wrong‑repo mistakes, reduces trust, and limits expansion to other agents.

## Goals
1. **Multi‑agent support** via ACP with a consistent runner abstraction.
2. **Clear context UI**: always show active runner + workspace + sandbox mode.
3. **Robust recovery**: reconnect / resume sessions cleanly with visible status.
4. **CI‑first dev flow**: avoid heavy local builds on low‑RAM machines.
5. **Support of cloud agents**: to allow delegation via CLI to e.g. Codex Cloud agent, Claude Code Web agent, Cursor Cloud agent etc.
6. **Separate stats by provider**: on dashboard we should show clearly data by provider and accumulated (original CodexMonitor only shows data for Codex)

## Non‑Goals (for v1)
- Full parity with all Codex App features
- Advanced multi‑user collaboration
- Auto‑merging upstream CodexMonitor changes

## Target Users
- Power developers running multiple agents across multiple repos
- Edukey internal use and demos

## Core Features (v1)
- Runner abstraction (Codex/Gemini/Claude via ACP)
- Runner selector in **New Thread** and **Workspace Settings**
- Persist runner + command/env per workspace
- ACP normalization layer (stdio + HTTP/WS)
- Session lifecycle handling (reconnect, reuse policy)
- Context header: runner + workspace + sandbox mode
- Data / Stats separation by tool / provider / model

## UX Principles
- **Make state obvious** (active runner/workspace/sandbox)
- **Make recovery boring** (clear reconnect and resume status)

## Success Metrics
- Start threads with different runners in same app
- Resume threads without losing context
- Fewer wrong‑repo / wrong‑runner mistakes
- CI pipeline runs lint/typecheck/tests reliably

## Dependencies
- Codex CLI (app-server) available in PATH (already implemented in CodexMonitor)
- Gemini CLI / Claude Code ACP adapters
- GH auth configured for PRs/CI checks

## Risks & Mitigations
- ACP differences between agents → normalize events, detect capabilities
- Low‑RAM dev machine → CI‑first, avoid local heavy builds
- Auth complexity → fine‑grained PAT + GH_TOKEN

## Open Questions
- Exact runner capability flags for each agent
- Remote runner default transport (stdio vs HTTP/WS)
- Best UI placement for runner context badge
