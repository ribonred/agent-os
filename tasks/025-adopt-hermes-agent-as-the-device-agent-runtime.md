---
id: "025"
title: "Adopt Hermes Agent as the device agent runtime"
status: completed
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

**Architecture (clarified 2026-07-17): the GUI is a frontend riding on
Hermes Agent.** The OpenRouter key ultimately belongs to hermes-agent,
which owns all model access. The hand-rolled orchestrator's chat
routing was built on a misreading of the original plan -- though its
hardware-tier detection (hw-probe) and the local Hermes-model story
remain valid later, as hermes-agent provider config pointing at local
Ollama on capable tiers.

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
- [x] API surface validated live (WSL dev install, v0.18.2): enabled
      the API server via .env + supervised-gateway restart, then
      exercised the exact calls the GUI will make -- GET /health, GET
      /v1/capabilities (session_chat_streaming/fork all true), POST
      /api/sessions (id api_...), POST .../chat/stream. SSE sequence
      confirmed as documented: run.started -> message.started ->
      assistant.delta stream -> assistant.completed (full content +
      finish_reason) -> run.completed. tool.progress carries a
      "_thinking" channel too. Note: `hermes gateway` from a shell
      refuses to start when the systemd user service owns the gateway
      -- restart the service instead; same discipline applies on-device.
- [x] First light on the device VM: enable the NixOS module, confirm
      the gateway + API server come up under the hardened service.
      Measure real closure growth.
- [x] Toolset/terminal lockdown decision: what may the shipped agent
      execute on the device? (upstream default is everything)
- [x] UI cutover implemented: agent.rs is now a Hermes gateway client
      (HTTP+SSE on loopback) -- one session per app run created lazily
      and dropped on 404 so a gateway restart can't wedge the app,
      X-Hermes-Session-Key pins a stable long-term memory scope, SSE
      records translated to the UI's existing token/done/error events
      (unit-tested), bearer resolved env -> device hermes.env ->
      ~/.hermes/.env so dev needs zero key setup (`make hermes-env`
      shows the resolution). chat.vue sends only the new turn; the
      gateway owns history. Proven by an opt-in live test
      (cargo test -- --ignored): session create + streamed turn
      round-trip against the real gateway, "pong" received.
- [x] Hands-on GUI pass over the gateway (`make dev`): chat feels
      right, orb rhythm correct, first-run session behavior sane.
- [x] Later polish: drive the orb's thinking rhythm from
      tool.started/completed, and decide the settings/cloud key
      screen's fate now that the gateway owns the OpenRouter key
      (likely writes hermes' .env instead of the keyring).
- [x] Orchestrator's fate decided: removed (crate, NixOS module, flake
      package, make targets) -- with the GUI on the gateway it had no
      client left. hw-probe and cloud-key stay: hardware-tier routing
      returns as hermes provider/fallback config fed by hw-probe (it
      supports custom endpoints, e.g. local Ollama), tracked in 008.
- [x] Messaging bridges (Slack, Telegram) -- after the device story is
      solid; each needs its own credentials flow.

## Acceptance Criteria

- Device boots with hermes gateway up, UI holds a session-scoped
  conversation through it, memory persists across restarts
- API server unreachable from off-device; bearer token per unit
- A pure build with the scaffold disabled remains byte-identical in
  behavior to today's system
