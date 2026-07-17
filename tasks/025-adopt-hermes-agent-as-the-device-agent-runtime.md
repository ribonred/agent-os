---
id: "025"
title: "Adopt Hermes Agent as the device agent runtime"
status: in-progress
priority: high
effort: large
phase: agent-core
dependencies: ["008"]
tags: ["hermes-agent", "nixos", "api", "messaging"]
created_at: 2026-07-17
---

# Adopt Hermes Agent as the device agent runtime

## Objective

Run Nous Research's Hermes Agent (hermes-agent.nousresearch.com) as the
device's agent runtime -- the original product idea. It brings what the
hand-rolled orchestrator was slowly growing toward, already built:
persistent cross-session memory, a skill system that improves with use,
60+ tools + MCP, and first-class messaging bridges (Slack, Telegram,
WhatsApp, 20+ platforms) for reaching the assistant away from the
device. OpenRouter remains the model provider inside it, keeping the
Hermes-4 model lockstep.

## What the docs research established (2026-07-17)

- **Runtime**: self-hosted Python app; `hermes gateway` is the daemon.
  Official NixOS flake + module (`github:NousResearch/hermes-agent`,
  `nixosModules.default`) with hardened systemd service, declarative
  `settings` (deep-merged into config.yaml), `documents` (SOUL.md
  identity file), `environmentFiles` for secrets. Upstream calls the
  flake "Tier 2" / best-effort -- pin via flake.lock, update
  deliberately, never casually. Default package adds ~700 MB closure.
- **Interaction surface** (the answer to "how to create sessions"):
  API server on 127.0.0.1:8642, bearer-auth (`API_SERVER_KEY`),
  enabled via env (`API_SERVER_ENABLED=true`). Three tiers:
  - `POST /v1/chat/completions` -- stateless OpenAI-compatible, SSE
    streaming plus custom `hermes.tool.progress` events
  - `POST /v1/responses` -- server-side history via
    `previous_response_id` or named `conversation`
  - Sessions API: `POST /api/sessions` (create), `POST
    /api/sessions/{id}/chat/stream` (SSE turn: `assistant.delta`,
    `tool.started`, `tool.completed`, `run.completed`), fork, list.
    Long tasks: Runs API (`/v1/runs` + events stream + approvals).
  - Headers `X-Hermes-Session-Id` / `X-Hermes-Session-Key` scope
    transcript vs long-term memory.
- **Config**: model as `provider: openrouter`, `default:
  openrouter/nousresearch/hermes-4-70b`; secrets in .env
  (`OPENROUTER_API_KEY`); SOUL.md is system-prompt slot #1.
- **Security note from upstream**: the API grants the full toolset
  *including terminal commands* -- loopback-only binding and per-unit
  bearer tokens are load-bearing, and toolset lockdown needs a
  deliberate pass before shipping.

## Tasks

- [x] Scaffold (inert until enabled): flake input pinned;
      modules/hermes-agent.nix wires the upstream module -- constitution
      as SOUL.md, OpenRouter + hermes-4-70b lockstep, memory on, API
      server on loopback:8642; installer renders
      /etc/agentic-os/hermes.env (shared vendor OpenRouter key + a
      per-unit API_SERVER_KEY generated at install time); tmpfiles
      handoff so the UI user can read the bearer token.
- [ ] First light: enable on the dev VM, confirm the gateway comes up,
      create a session and stream a turn with curl against
      /api/sessions. Measure real closure growth.
- [ ] Toolset/terminal lockdown decision: what may the shipped agent
      execute on the device? (upstream default is everything)
- [ ] UI cutover: Tauri shell speaks the sessions API (SSE) instead of
      the orchestrator socket; orb rhythm driven by
      tool.started/completed events.
- [ ] Decide the orchestrator's fate: hardware-tier local/cloud routing
      must survive -- either hw-probe feeds hermes' provider/fallback
      config (it supports custom endpoints, e.g. local Ollama), or the
      orchestrator shrinks to a router in front of it.
- [ ] Messaging bridges (Slack, Telegram) -- after the device story is
      solid; each needs its own credentials flow.

## Acceptance Criteria

- Device boots with hermes gateway up, UI holds a session-scoped
  conversation through it, memory persists across restarts
- API server unreachable from off-device; bearer token per unit
- A pure build with the scaffold disabled remains byte-identical in
  behavior to today's system
