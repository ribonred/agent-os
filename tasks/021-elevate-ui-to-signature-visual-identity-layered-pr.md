---
id: "021"
title: "Elevate UI to signature visual identity (layered presence orb, greeting cycle, staged motion)"
status: completed
priority: high
effort: medium
phase: ui-shell
dependencies: []
tags: ["svelte", "design", "gui"]
created_at: 2026-07-13
completed_at: 2026-07-14
---

# Elevate UI to signature visual identity (layered presence orb, greeting cycle, staged motion)

## Objective

The first-pass UI (flat single-color breathing dot on dark) was exactly
the generic "near-black + one bright accent" template look -- called out
as not stunning enough. Researched current AI-interface references
(premium orbs are layered: multiple radial gradients for depth, rotating
conic gradient for iridescence, blur for atmosphere -- never one flat
glow) and rebuilt the identity around a real signature element.

## Tasks

- [x] DESIGN.md updated first (source-of-truth rule): orb-only color
      tokens (cyan/violet/warm/deep -- violet exists only inside the orb,
      never as standalone UI color), the 5-layer orb spec (atmosphere,
      conic core rotating on transform for webview compat, specular
      shading, aperture ring, light pool), and the first-boot greeting
      cycle spec
- [x] PresenceOrb.svelte -- shared component, size + grounded props, all
      five layers, prefers-reduced-motion respected
- [x] Home screen: 180px grounded orb on a light pool (the device as a
      physical object on the counter it ships to), vignette, staged
      entrance, "Ready when you are." invitation line
- [x] Language screen: cycling multilingual greeting (Halo first --
      Indonesian-first is the product's actual identity, not a generic AI
      trick), bilingual eyebrow (Pilih bahasa · Choose your language),
      staggered list entrance, focus-visible states
- [x] Persona screen: same world -- small orb, "How should I be with
      you?" heading, bilingual eyebrow, staggered cards, RECOMMENDED
      badge in accent-warm
- [x] Visual critique loop via Playwright screenshots on an isolated
      fresh-cache server: first pass read as a "beach ball" (conic
      segments too legible at 180px, soft-light highlight invisible on
      near-black) -- fixed with heavier size-relative blur (segments melt
      into continuous iridescence) and normal-blend shading. Re-shot and
      confirmed: reads as a luminous sphere with real volume.
- [x] Home screen screenshotted despite the fail-closed setup gate by
      faking the Tauri IPC bridge in the browser only (addInitScript,
      contract read from the installed plugin source: store get returns
      [value, exists]) -- zero product code touched for testing. This
      also incidentally validated the gate's "setup complete -> stay"
      path, which the earlier plain-browser check couldn't reach.

## Acceptance Criteria

- [x] No flat single-accent orb -- layered, iridescent, volumetric,
      confirmed by screenshot at both 72px and 180px
- [x] Setup flow and home screen read as one coherent world (orb present
      from first boot onward)
- [x] svelte-check clean, production build clean
- [ ] Judged stunning by the person who asked -- needs Red's eyes on
      `bun run tauri dev`, and the greeting-cycle motion can only be
      felt live, not in a static screenshot
