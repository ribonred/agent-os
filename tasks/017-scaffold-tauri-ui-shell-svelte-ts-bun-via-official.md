---
id: "017"
title: "Scaffold Tauri UI shell (Svelte+TS+bun) via official CLI"
status: in-progress
priority: high
effort: medium
phase: ui-shell
dependencies: []
tags: ["tauri", "svelte", "gui"]
created_at: 2026-07-13
---

# Scaffold Tauri UI shell (Svelte+TS+bun) via official CLI

## Objective

Get a real, buildable Tauri UI shell in place -- physically testable by
Red during OS setup, per the actual reason this jumped the queue. Not
building screens yet, just a validated foundation: official scaffold,
system toolchain (not fought through Nix), logging wired in from day one.

## Tasks

- [x] Scaffold via `bunx create-tauri-app` (official CLI, not manual) --
      Svelte+TS+bun, identifier com.agenticos.shell
- [x] Resolved a real architecture question mid-task: develop against the
      system's existing rustup/cargo-tauri toolchain (already installed,
      already has a working WSLg display), not the Nix devShell used for
      hw-probe/ingest -- that Nix environment has no GTK/webkit libs.
      Added ui/.envrc (empty, opts this subtree out of the parent repo's
      `use flake`) -- validated: `cargo` inside ui/ resolves to rustup
      1.97.0, not the Nix store. Packaging the eventual built app for the
      NixOS-based device is real but separate work (see task 018) --
      binaries built with a normal distro toolchain don't run on NixOS
      as-is (no FHS, dynamic linker can't find libraries) without
      autoPatchelfHook-style patching.
- [x] Wired tauri-plugin-log from the start (LogDir + Stdout + Webview
      targets), plus the log:default capability permission -- this is
      the concrete log-scanning tooling requested, in place before
      anything breaks, not added after.
- [x] Validated for real, each layer separately:
      - `cargo check`: clean
      - `bun run check` (svelte-check): 0 errors, 0 warnings, 134 files
      - `bun run build`: frontend builds clean
      - `bun run tauri build --debug`: real binary produced
        (target/debug/ui), .deb and .rpm bundles both complete
        successfully (strong evidence of correct linking/dependencies)
      - AppImage bundling hit a real, diagnosed, non-blocking gap:
        libfuse.so.2 missing on this WSL2/Ubuntu box (confirmed by
        running linuxdeploy directly, not guessed) -- APPIMAGE_EXTRACT_
        AND_RUN=1 didn't propagate through Tauri's nested subprocess
        chain to fix it. Not chased further: neither deb/rpm/AppImage is
        the actual NixOS packaging format anyway (task 018 handles that
        separately), so this doesn't block anything real.
      - Launched the actual compiled binary: process stays alive, no
        crash, log file confirmed at the real Tauri app-log path
        (~/.local/share/com.agenticos.shell/logs/ui.log). Could NOT
        confirm visual window rendering from this sandboxed environment
        -- X11's window tree shows nothing, but WSLg commonly renders via
        native Wayland which an X11-only tool wouldn't see regardless of
        whether it worked. Deliberately did not chase this further into
        WSLg display-stack internals unrelated to the app's own
        correctness -- this is the actual "physically testable" step Red
        asked to do themselves.

## Acceptance Criteria

- [x] Scaffolded via the official CLI, not hand-built
- [x] Builds clean through Rust compile, frontend type-check, frontend
      build, and full Tauri bundle (two of three Linux package formats)
- [x] Logging infrastructure exists before it's needed, not after a bug
- [ ] Visual rendering confirmed -- pending Red running `bun run tauri
      dev` interactively and actually seeing the window
