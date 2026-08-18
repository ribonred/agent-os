---
name: agentic-ui
description: >
  Change the Tauri + Nuxt assistant shell, setup screens, chat, files,
  orb, or design tokens. Use for ui/, design/DESIGN.md, design/DESKTOP.md,
  visual or layout claims, Playwright checks, and make dev/gui/ui-bundle —
  even if the user only said button, screen, glow, or centered. For the
  Rust host, invoke, and window modes use tauri-nuxt. For taste, copy,
  and DESIGN.md rules use product-ux.
---

# Agentic UI

`ui/` is a Tauri 2 + Nuxt 4 (Vue) shell. It is a frontend to the local
Hermes gateway. The owner talks here; they do not operate Ubuntu.

## When to Use

- Any file under `ui/` or `design/`
- Setup flow, chat pane, file shelf, presence orb, cloud-key screen
- "It doesn't look right", typecheck-is-green-but-layout-is-wrong
- Dev server, kiosk bundle, scoped-CSS canary

## Design first

`design/DESIGN.md` is the spec. Change it, then the implementation.
Do not invent hex values in a component. Tokens live there: `--bg`,
`--surface`, `--accent` (cyan), `--accent-warm` (amber), plus orb-only
colors that must never appear on buttons or text.

`design/DESKTOP.md` is why Ubuntu's desktop sits underneath and how
the agent launches Chrome / `wmctrl` / `gio open`. The assistant
autostarts and is what the owner returns to.

Copy order for setup lists: `brain/onboarding.md` first, then
`ui/app/lib/setupOptions.ts`. Language list is Asia-first, Indonesia
first, not alphabetical.

## Dev loop

```
make hermes-env    # gateway URL + masked key
make dev           # warns if gateway/port 3000 is wrong, then gui
make gui           # bun run tauri dev (needs display + Hermes)
make test          # hw-probe + cloud-key cargo test, then bun run check
cd ui && bun test  # unit tests (agentErrors, etc.)
make ui-bundle     # release binary + scoped-hash canary
```

Dev server is exactly `http://localhost:3000` (`nuxt.config.ts` +
`tauri.conf.json`). `ssr: false`. Build with bun + the system
rustup/cargo, via `env -i` in the Makefile, never a mixed toolchain.

Owner-facing failures go through `ui/app/lib/agentErrors.ts`. Raw
gateway URLs, paths, and errno stay in the log.

## Visual verification (required for layout claims)

`nuxt typecheck` and a successful build do not prove "it's centered"
or "the glow renders."

Use Playwright MCP: navigate, screenshot, `Read` the image. If the
screenshot cannot explain a miss, evaluate `getComputedStyle` on the
node.

**Hard rule:** while the user's `bun run tauri dev` is running, do not
start another Vite/Nuxt server on this tree and do not delete `.nuxt/`,
`dist/`, or `node_modules/.vite/`. A second server shares those caches
and serves mixed compiler generations. That already happened.

When a user server is up: verify against a static `vite`/`nuxt`
generate on a throwaway port, or ask first.

When no user server is up: an isolated throwaway server is fine. Kill
it. Delete screenshots and `.playwright-mcp/` when done. Verification
output is not a commit.

Suspect a stale Vite/Nuxt cache before a CSS bug, especially after
adding a route. Clear caches only when no dev server is running, then
retest on a fresh isolated server.

`make ui-bundle` already wipes caches and fails on a scope-hash
mismatch of `.orb[data-v-*]`. Do not ship a build that fails that
canary.

Device window shape is `window_mode.rs` (full vs pill), not a second
app. `make ui-bundle` may pass a kiosk overlay config; if that file is
missing, do not invent one — ask. Dev stays a normal window.

## Gotchas

- Do not touch the user's interactive Tauri session.
- Test files are excluded from `nuxt typecheck` on purpose
  (`**/*.test.ts`).
- The webview must never see the Hermes bearer token. Chat goes
  through Tauri commands in `ui/src-tauri/src/agent.rs`.
- Fresh setup is language → name → guided onboarding. Do not reintroduce
  a persona screen on new devices. Keep a stored persona only as an
  upgrade path.
