---
id: "023"
title: "GUI-driven local model download during onboarding"
status: pending
priority: medium
effort: medium
phase: ui-shell
dependencies: ["008", "022"]
tags: ["ollama", "onboarding", "gui"]
created_at: 2026-07-17
---

# GUI-driven local model download during onboarding

## Objective

The local model (hermes3:3b on the low tier) is NOT downloaded at boot
-- that was removed deliberately: a silent ~2GB background download at
first networked boot is invisible to the person setting up the device
and fails invisibly offline. Instead the download becomes a visible,
consented onboarding step in the GUI: the device explains what it's
fetching and why, shows real progress, and works via cloud in the
meantime (a fresh provisioned unit chats through OpenRouter from the
first minute; the local tier arrives when the pull completes).

Likely shape (decide at implementation, not now): the orchestrator
daemon owns the pull (it already owns routing and talks to Ollama),
exposing start/progress over its socket -- Ollama's /api/pull streams
progress JSON that can be forwarded to the UI the same way chat tokens
are. Which model to pull comes from the hardware tier (hw-probe), not
hardcoded in the UI.

## Tasks

- [ ] Orchestrator endpoint to start/observe a model pull (streamed
      progress, loud failure, resumable -- Ollama pulls resume natively)
- [ ] Onboarding step in the UI: consent + progress + "you can keep
      using me meanwhile" (cloud routing keeps working during the pull)
- [ ] Routing behavior during/after: local tier becomes available the
      moment the model lands, without a restart
- [ ] Offline-only devices: decide the story (ship-with-model factory
      flavor? require one networked session?) -- currently an offline
      fresh device has no local model and no cloud, which must at least
      be surfaced honestly in the UI

## Acceptance Criteria

- Fresh device never downloads models silently; every byte of model
  download happens behind explicit onboarding consent with visible
  progress
- Cloud-provisioned units are fully usable during the pull
- Local routing activates when the pull completes, no reboot needed
