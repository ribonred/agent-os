---
id: "008"
title: "Implement hardware capability detection + local/online LLM routing policy"
status: pending
priority: high
effort: large
phase: agent-core
dependencies: []
tags: ["agent-core", "routing", "llm"]
created_at: 2026-07-13
---

# Implement hardware capability detection + local/online LLM routing policy

## Objective

The device must not hardcode "this SKU always uses local" or "always uses
cloud" -- the agent runtime should probe what it's actually running on and
route accordingly, so the same software image works across the NUC dev
tier, a future mid-tier box, and DGX Spark without a rebuild.

## Tasks

- [ ] Define the detection probe: CPU cores/threads, RAM, presence and type
      of GPU/NPU, unified vs discrete memory -- read once at boot, cache the
      result, don't re-probe per request
- [ ] Define the tier policy (subject to revision once real hardware is in
      hand):
  - Low-resource (no accelerator, e.g. NUC11PAHi3/Iris Xe): Gemma 4
    E2B/E4B local for bootstrap/basic OS chrome, cloud for anything heavier
  - Mid (mini PC w/ GPU or NPU): Gemma 4 12B or a small Hermes cut, mostly
    self-sufficient offline
  - High (DGX Spark, 128GB unified memory): Hermes 70B+ fully offline
- [ ] Separate the connectivity check (is the network up right now) from the
      capability check (what can this hardware run) -- they're independent
      axes, don't conflate them
- [ ] Decide the per-vertical override: money/health-leaning verticals may
      want to force offline-only regardless of tier, for privacy/compliance
- [ ] Wire the routing decision into the agent orchestration loop

## Acceptance Criteria

- On the NUC dev box, the agent correctly self-identifies as low-resource
  tier and routes non-trivial requests online without being told to
- Tier detection result is logged/inspectable, not a silent internal choice
  (red-mindset: no silent fallback)
