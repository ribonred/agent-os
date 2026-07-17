---
id: "024"
title: "Migrate UI shell from Svelte to Nuxt"
status: in-progress
priority: high
effort: large
phase: ui-shell
dependencies: ["017"]
tags: ["tauri", "nuxt", "vue", "gui"]
created_at: 2026-07-17
---

# Migrate UI shell from Svelte to Nuxt

## Objective

Rebuild the Tauri shell's frontend on Nuxt (Vue), replacing SvelteKit,
after the intermittent scoped-CSS/markup split-generation failures made
the Svelte toolchain untrustworthy for this product. Clean-slate
scaffold per the official Tauri v2 + Nuxt integration guide (ssr:
false, fixed dev port 3000, `nuxi generate` -> ../dist), not a
file-by-file translation of the old tree.

Everything the Svelte UI had carries over: same design system
(DESIGN.md unchanged -- tokens, presence orb spec, conversation
surface), same Rust command surface (cloud_key, agent chat streaming
over the Unix socket -- restored from git history verbatim, it was
framework-agnostic), same store-gated mandatory setup flow.

## Tasks

- [x] Scaffold: nuxi minimal template + official Tauri v2 Nuxt config
      (ssr off, devServer host 0, strictPort, TAURI_ env prefix,
      src-tauri ignored). Tauri deps: @tauri-apps/api, plugin-store JS;
      typescript pinned to 5.x (v7/tsgo breaks vue-tsc).
- [x] src-tauri restored from git history: cloud_key.rs, agent.rs,
      lib.rs, capabilities, icons, Cargo manifest -- only the build
      section of tauri.conf.json changed (Nuxt dev URL/commands,
      frontendDist ../dist). cargo check clean.
- [x] Kiosk window config done properly this time:
      tauri.kiosk.conf.json overlay (fullscreen, no decorations) used
      by `make ui-bundle` -- the device gets a chromeless fullscreen
      window, dev keeps a normal one. Fixes the visible titlebar from
      the first VM kiosk boot.
- [x] All screens ported to Vue pages: home (wake orb), setup/language
      (greeting cycle), setup/persona, settings/cloud (three key
      states), chat (Channel-streamed ndjson, orb rhythm as status).
      PresenceOrb.vue keeps the 5-layer spec and rhythm states (prop
      named orbState). Setup gate lives in app.vue, fails closed as
      before.
- [x] `nuxt typecheck` clean; static build verified in-browser against
      dist/ with the IPC fake: orb exactly 180px, zero horizontal
      overflow, all four screens screenshotted, chat stream exercised.
      Makefile canary rewritten for Vue scoping (.orb[data-v-*] in CSS
      must pair with the attr in built JS).
- [ ] Release bundle (`make ui-bundle`) + kiosk ISO rebuild + VM
      boot-to-orb re-verification on the Nuxt shell.
- [ ] Hands-on pass: `make dev` daily-driver feel, and confirm the
      intermittent layout breakage does not recur on the new toolchain.

## Acceptance Criteria

- [x] Feature parity with the Svelte shell on all five screens,
      verified visually against the static build
- [x] No Svelte/SvelteKit dependency remains in ui/
- [ ] Device image boots to the fullscreen undecorated orb (VM check)
- [ ] The "sometimes broken sometimes not" layout failure does not
      reproduce on the new toolchain in normal dev use
