---
id: "020"
title: "Build mandatory language + persona selection screens (onboarding.md)"
status: completed
priority: high
effort: medium
phase: ui-shell
dependencies: ["017"]
tags: ["svelte", "onboarding"]
created_at: 2026-07-13
completed_at: 2026-07-14
---

# Build mandatory language + persona selection screens (onboarding.md)

## Objective

Build the two mandatory device-setup screens onboarding.md specifies:
language (10 options, Indonesia first) and agent persona (4 presets,
Balanced default). Direct UI selection, not LLM-generated questions --
these gate the root screen until both are chosen.

## Tasks

- [x] Add tauri-plugin-store -- Rust + JS sides, log:default-style
      permission (store:default). Compiled clean before any Svelte work
      started.
- [x] Language selection screen -- src/routes/setup/language, 10 options
      from onboarding.md via a shared src/lib/setupOptions.ts (source of
      truth is the doc, this mirrors it, same pattern as design/DESIGN.md
      -> app.css)
- [x] Persona selection screen -- src/routes/setup/persona, 4 presets,
      Balanced visually marked recommended (--accent-warm border)
- [x] Root layout redirects to setup if language/persona unset -- fails
      closed, not open: a real bug caught during validation (store read
      throws with TypeError outside an actual Tauri window -- no IPC
      bridge exists in a plain browser) meant the gate would silently let
      the user through unconfigured on ANY store-read failure, not just
      the browser-context case. Fixed with try/catch defaulting to
      "incomplete" on error, then re-verified the redirect actually fires.
- [x] Validated via Playwright: screenshotted both screens on a freshly-
      cached isolated dev server (learned that lesson already). Language
      list renders correctly, Indonesia first; CJK/Thai/Hindi entries
      showed as tofu boxes in the screenshot -- checked whether that was
      a real product bug or a sandbox artifact (fc-list: zero CJK/Thai/
      Devanagari fonts in this sandbox) and found a genuine gap: the
      NixOS host config had zero font packages declared at all, so
      design/DESIGN.md's system-font-stack decision wasn't actually
      backed by fonts on the real device either. Added noto-fonts +
      noto-fonts-cjk-sans to hosts/host/configuration.nix, verified the
      package names against real docs (not guessed), and confirmed the
      full host closure still builds with them -- real validation, not
      assumed. Persona screen also screenshotted, correct layout.
      Real limitation surfaced and documented: Playwright hitting the
      Vite dev server directly can validate layout/CSS but not
      Tauri-plugin-backed behavior (no IPC bridge outside the real Tauri
      window) -- that needs Red testing inside `bun run tauri dev`.

## Acceptance Criteria

- [x] A fresh launch (no stored prefs) lands on language selection, not
      the main screen -- verified the fail-closed redirect fires
- [~] Selections persist across a reload (real store, not just in-memory
      component state) -- code path is correct and store:default
      permission is granted, but the actual write/persist behavior can
      only be exercised inside a real Tauri window per the limitation
      above, not from this sandbox
- [x] Both screens visually verified via screenshot, not just typecheck
