# ADR-0002 — Multi‑Agent Support via ACP

## Status
Proposed

## Context
AgentsManager must orchestrate Codex, Gemini CLI, and Claude Code. Each exposes ACP (stdio or HTTP/WS). We need a unified runner abstraction.

## Decision
Implement a common **AgentRunner + ACP transport** layer that normalizes ACP events and supports stdio + HTTP/WS transports.

## Rationale
- ACP is the shared protocol across target agents.
- Transport abstraction allows local stdio and remote HTTP/WS.
- Normalized events simplify frontend rendering.

## Consequences
- Requires mapping capability flags per runner.
- Needs session reuse policy and reconnect handling.
