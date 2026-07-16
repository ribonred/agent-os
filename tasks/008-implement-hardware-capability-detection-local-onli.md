---
id: "008"
title: "Implement hardware capability detection + local/online LLM routing policy"
status: in-progress
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

- [x] Define the detection probe: CPU cores/threads, RAM, presence and type
      of GPU/NPU, unified vs discrete memory -- read once at boot, cache the
      result, don't re-probe per request
      -- built as agent-core/hw-probe (Rust, sysinfo for CPU/RAM,
      /sys/class/drm + /sys/class/accel for GPU/NPU). CPU/RAM validated
      against ground truth on this dev box (16 cores, 31.3GiB, exact
      match). GPU/NPU sysfs reads are NOT validated against real hardware
      -- this dev sandbox is WSL2, which doesn't expose a real DRM/PCI
      topology (confirmed empty). Must be checked on the actual NUC.
- [x] Define the tier policy (subject to revision once real hardware is in
      hand):
  - Low-resource (no accelerator, e.g. NUC11PAHi3/Iris Xe): Gemma 4
    E2B/E4B local for bootstrap/basic OS chrome, cloud for anything heavier
  - Mid (mini PC w/ GPU or NPU): Gemma 4 12B or a small Hermes cut, mostly
    self-sufficient offline
  - High (DGX Spark, 128GB unified memory): Hermes 70B+ fully offline
      -- classify_tier() implemented and unit-tested with fabricated
      profiles for all three tiers, including a synthetic DGX-Spark-like
      input (128GB/NVIDIA) since we don't have that hardware yet -- same
      honesty pattern as the placeholder hardware-configuration.nix.
      Known gap documented in code: vendor-ID-only heuristic can't tell
      an integrated AMD APU from a discrete AMD GPU, or Intel integrated
      from Intel discrete Arc. Not fixed since neither NUC (Intel
      integrated) nor DGX Spark (NVIDIA discrete) hits that ambiguity --
      revisit if AMD or Intel-discrete hardware enters scope.
- [x] Separate the connectivity check (is the network up right now) from the
      capability check (what can this hardware run) -- they're independent
      axes, don't conflate them
      -- is_online() implemented as a raw TCP connect to 1.1.1.1:443
      (2s timeout, no DNS dependency). Validated live against this
      sandbox's real network (test asserts true, not fabricated). Kept
      as a genuinely separate function/signal from classify_tier -- the
      routing function takes both as independent inputs.
- [~] Decide the per-vertical override: money/health-leaning verticals may
      want to force offline-only regardless of tier, for privacy/compliance
      -- decide_default_routing() takes a vertical_forces_offline bool and
      is tested against it (forces Local even for high-tier+online). Not
      fully wired: no real vertical config exists yet to source this from,
      so it's hardcoded false in main(). Scaffolded for when that config
      exists, not a finished integration.
      Bug caught and fixed while scoping the cloud provider (OpenRouter):
      decide_default_routing() only checked tier + network reachability,
      but a reachable network isn't a usable cloud tier without an API
      key configured -- it would have leaned cloud and then just failed
      at call time. Added has_cloud_credentials as a required input,
      hardcoded false in main() until a real credential store exists.
      This actually changed this box's live output: NUC-class profile
      now correctly reports default_routing: "local" instead of "cloud",
      since no credentials exist here yet -- not a regression, the
      earlier "cloud" result was the bug.
- [~] Wire the routing decision into the agent orchestration loop
      -- the orchestrator now exists: agent-core/orchestrator, a
      standalone Rust daemon (design decided deliberately: separate
      process under systemd so the agent outlives the UI and ships
      unchanged to the DGX tier; HTTP over a Unix socket, chmod 0600, no
      TCP port on a customer device). hw-probe was refactored into a
      library (lib.rs) the daemon links directly -- one implementation of
      tier policy + routing lean, two consumers (CLI stays as a thin JSON
      printer). Same for credentials: the keyring/provisioned-file logic
      moved from the Tauri crate into a shared agent-core/cloud-key
      crate; the daemon resolves the real key, so has_cloud_credentials
      is no longer hardcoded anywhere.
      API: GET /status (all routing inputs + decision, inspectable);
      POST /chat (ndjson token stream; constitution.md is the system
      prompt, loaded at startup, daemon refuses to start without it;
      client-supplied system messages rejected 400).
      Validated live from this sandbox, not simulated: /status correctly
      reported tier=low, online, key_source=provisioned,
      default_routing=cloud, and /chat streamed a real Hermes-70B
      response through OpenRouter end-to-end over the Unix socket. The
      no-key path verified too: routes local, and with no Ollama running
      fails as a loud ndjson error event -- no silent fallback to the
      other backend (deliberate: auto-failover is a product decision,
      not a default).
      Keyring read posture decided while validating: a dead/missing
      Secret Service daemon logs loudly and falls through to the
      provisioned file instead of bricking cloud entirely; writes still
      fail hard. (This also makes WSL dev work.)
      Still open, in order: NixOS packaging (buildRustPackage + systemd
      unit + socket dir), UI chat client on the socket, local-path
      success validation against a real Ollama (this sandbox has none),
      GPU/NPU probe validation on bare metal (WSL2 masks PCI vendor IDs
      -- confirmed concretely, shows Microsoft 1414, not the real
      vendor).

## Acceptance Criteria

- [x] On the NUC dev box, the agent correctly self-identifies as
      low-resource tier and routes non-trivial requests online without
      being told to
      -- verified on this dev box as a stand-in: Low tier, online: true,
      default_routing: Cloud, all without being told. Real NUC run still
      pending (needs on-device execution).
- [x] Tier detection result is logged/inspectable, not a silent internal
      choice (red-mindset: no silent fallback)
      -- hw-probe prints every intermediate fact (cores, memory, GPU
      vendors, NPU, tier, online, routing lean), not just the final
      decision.
