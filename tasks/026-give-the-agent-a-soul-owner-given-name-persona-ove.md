---
id: "026"
title: "Give the agent a soul: owner-given name + persona overlays"
status: in-progress
priority: high
effort: medium
phase: agent-core
dependencies: ["025"]
tags: ["identity", "persona", "onboarding", "hermes-agent", "ui"]
created_at: 2026-07-18
---

# Give the agent a soul: owner-given name + persona overlays

## Objective

Before this task, the setup screens' persona and language choices were
stored and never read again -- a user picking "Formal & Precise"
changed nothing -- and the agent had no name anywhere. Product
decisions made 2026-07-18: the device ships nameless and the **owner
christens it during onboarding** (no canonical product name), and
`brain/constitution.md` stays the invariant base soul with the pre-UI
persona selection applied as a real voice overlay.

## Design (implemented)

- **Injection point**: per-turn `system_message` in the gateway's
  `/api/sessions/{id}/chat/stream` body. Verified against hermes-agent
  0.18.2 source: it is applied as an ephemeral system prompt appended
  *after* the assembled core prompt (SOUL.md + memory), which is
  exactly "overlay on the constitution" -- and per-turn means a
  changed name/persona applies on the next send with no
  session-recreate logic. Session-creation `system_prompt` was
  rejected: 0.18.2 stores it but never feeds it into chat runs.
- **Rust owns composition** (ui/src-tauri/src/agent.rs): the webview
  never passes prompt text over IPC. Persona and language are matched
  against known ids (unknown -> baseline); the owner-chosen name is
  the only free text and is sanitized (control chars -> space,
  whitespace collapsed, 60-char cap). Overlay texts mirror
  brain/onboarding.md's "Persona voice overlays" section verbatim --
  change the doc first.
- **Third setup step**: ui/app/pages/setup/name.vue ("What will you
  call me?"), spec'd in design/DESIGN.md "Naming screen". The name
  lives in the agent's voice only -- the no-name-badge rule of the
  conversation surface stands.
- **Migration**: app.vue's gate now routes to
  `firstIncompleteSetupStep()` -- an already-set-up device (language +
  persona, no name) lands directly on /setup/name once, then home.

## Tasks

- [x] Docs first: onboarding.md third mandatory question + canonical
      overlay texts + real mechanism description; constitution.md
      identity paragraph (nameless until christened, never claim a
      different identity); DESIGN.md naming-screen spec.
- [x] Store + flow: agentName key, firstIncompleteSetupStep(),
      persona -> name -> home; gate migration.
- [x] agent.rs: compose_overlay + sanitize_name + store read
      (tauri-plugin-store StoreExt) + conditional system_message.
      8 unit tests (id rejection, name sanitization incl.
      newline-collapse injection guard, partial composition).
- [x] Live verification: `cargo test -- --ignored` --
      live_gateway_named_identity streams "Kirana" back when asked its
      name with the overlay attached (dev gateway, real model).
- [x] Naming screen visually verified against the static build
      (Playwright screenshot: orb/h1/eyebrow/input/disabled-Continue
      per DESIGN.md).
- [ ] Hands-on pass (`make dev`): full three-step flow, then chat in
      persona + name + language and feel the register shift.
- [ ] Device pass: rebuild ui-bundle + image after this lands so the
      device VM/NUC carries the naming step.

## Acceptance Criteria

- A fresh device asks language -> persona -> name before any
  conversation; an upgraded device asks only for the name once
- The agent introduces itself by the owner-given name, keeps the
  selected persona's register, and replies in the selected language
- Core Behavior rules are untouched by any persona (overlay shifts
  register only)
- The webview cannot inject arbitrary system-prompt text through the
  store (unknown ids ignored; name sanitized)
