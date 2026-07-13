---
id: "019"
title: "Implement design/DESIGN.md tokens in Svelte (colors, AI presence indicator, motion)"
status: completed
priority: high
effort: large
phase: ui-shell
dependencies: ["017"]
tags: ["svelte", "design"]
created_at: 2026-07-13
completed_at: 2026-07-13
---

# Implement design/DESIGN.md tokens in Svelte (colors, AI presence indicator, motion)

## Objective

First real pass at design/DESIGN.md in code -- tokens as CSS custom
properties, and the first component that uses them (the idle-state AI
presence indicator), replacing the default Tauri+Svelte tutorial template
entirely. Not the full motion spec (listening/thinking/speaking states)
yet -- per "shape it incrementally later," this is the foundation those
build on, not the finished thing.

## Tasks

- [x] src/app.css -- all color tokens from DESIGN.md as CSS custom
      properties, system font stack, dark background applied globally
- [x] src/routes/+layout.svelte created (didn't exist before) to import
      the global CSS -- SvelteKit needs this as the entry point
- [x] Replaced +page.svelte entirely -- removed the default tutorial
      content (Vite/Tauri/Svelte logos, the greet demo) for a minimal
      centered presence indicator: a radial-gradient glow circle in
      --accent, with a slow breathing animation (opacity/scale pulse)
      matching DESIGN.md's "idle: slow breathe" description
- [x] Validated: svelte-check clean (0 errors, 0 warnings, 134 files),
      full frontend build clean
- [x] Red caught a real bug: not actually centered when viewed live.
      Playwright MCP got installed mid-task specifically to check this --
      screenshotted the running page and confirmed visually: the orb sat
      pinned top-left. Root cause diagnosed via getComputedStyle, not
      guessed: `main` was computed as `display: block`/`height: 120px`,
      not the flex/100vh actually written -- the long-running dev server
      never properly picked up +layout.svelte (a brand-new file) into
      SvelteKit's route manifest, and a scoped style tag showed corrupted
      content on inspection, both pointing at a stale Vite cache rather
      than a real CSS bug. Confirmed by spinning up an isolated, freshly-
      cached dev server on a separate port (cleared .svelte-kit and
      node_modules/.vite first) -- screenshotted that one too, orb
      perfectly centered. Cleaned up the throwaway server, screenshots,
      and stray .playwright-mcp/ output afterward (added to .gitignore).
      Durable capability now available for future UI work in this
      project: Playwright MCP lets visual/layout claims actually get
      checked against a real rendered page, not just typecheck/build
      success.

## Acceptance Criteria

- [x] Design tokens exist as real CSS variables, not hardcoded hex values
      scattered across components
- [x] Default tutorial template fully removed
- [ ] Listening/thinking/speaking states, and anything beyond the idle
      breathe -- explicitly deferred, not a gap in this pass
