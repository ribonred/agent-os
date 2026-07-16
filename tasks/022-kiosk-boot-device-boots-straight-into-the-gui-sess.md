---
id: "022"
title: "Kiosk boot: device boots straight into the GUI session"
status: in-progress
priority: high
effort: medium
phase: ui-shell
dependencies: ["018"]
tags: ["kiosk", "cage", "session", "nixos"]
created_at: 2026-07-16
---

# Kiosk boot: device boots straight into the GUI session

## Objective

Power on -> orb. The device must boot directly into the assistant UI
with no login prompt, no desktop, no visible OS -- the difference
between "a box with services" and the product. Today the installed
system boots to a text login; every service starts but nothing launches
the GUI.

Design:

- **cage** (Wayland single-app kiosk compositor, first-class NixOS
  module) runs the Tauri shell fullscreen as the device user. No
  desktop environment, no window management, nothing else launchable.
- The Tauri app enters the closure via task 018's packaging path: built
  outside Nix with the system toolchain (the repo's standing decision),
  then wrapped by a Nix derivation using autoPatchelfHook against the
  pinned GTK/webkit stack. The bundle is env-pointed
  (AGENTIC_OS_UI_BUNDLE + --impure), same opt-in pattern as the
  provisioned ISO flavor: a pure build produces a headless system, the
  kiosk flavor needs the explicitly provided artifact. A fully-pure
  in-Nix UI build remains the eventual goal; this formalizes the
  interim honestly instead of pretending Tauri-in-Nix is solved.
- The cage session also carries the session plumbing the daemon needs:
  PAM-unlocked gnome-keyring (session D-Bus), which is the recorded
  path to lifting the "daemon can't read user-saved keyring keys"
  limitation.

## Tasks

- [ ] Nix package for the prebuilt UI bundle (autoPatchelfHook wrap,
      env-pointed artifact, loud failure when the env var is set but
      the path is wrong; pure builds skip the kiosk cleanly)
- [ ] modules/kiosk.nix: cage as the device session running the UI as
      admin, PAM keyring unlock wired
- [ ] Host closure builds in both flavors: pure (headless, no kiosk)
      and impure-with-bundle (kiosk baked in)
- [ ] Boot-to-orb proven in a VM (QEMU/VMware) from the installer ISO
      kiosk flavor
- [ ] Boot-to-orb on the physical NUC (bench; closes 018's last box in
      the same session)

## Acceptance Criteria

- Booting the installed device lands in the fullscreen UI with no
  login prompt and no interactive step
- The UI can reach the orchestrator socket and hold a conversation in
  that session
- A pure `nix build` of the host still succeeds with no bundle present
  (headless flavor) -- the kiosk is additive, never a hard dependency
