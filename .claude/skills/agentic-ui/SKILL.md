---
name: agentic-ui
description: >
  Change the Tauri + Nuxt assistant shell, setup screens, chat, files,
  orb, or design tokens. Use for ui/, design/DESIGN.md, design/DESKTOP.md,
  visual or layout claims, make ui-drive or Playwright checks, and
  make dev/gui/ui-bundle — even if the user only said button, screen,
  glow, or centered. For the Rust host, invoke, and window modes use
  tauri-nuxt. For taste, copy, and DESIGN.md rules use product-ux.
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

Two loops. Picking the wrong one wastes the pass.

**Does the claim depend on the native half?** A command's real return
value, window modes, the store on disk, a gateway round-trip, or how
WebKitGTK lays the page out — use `make ui-drive`. Anything else, the
browser is quicker.

### `make ui-drive` — the real app

Drives the built binary over WebDriver (`ui/dev/drive.py`). The app runs
unmodified; nothing is added to the shipped binary for it. Import `App`
for a scenario: `js()`, `click()`, `type_into()`, `screenshot()`,
`settle()`, `log()`, and `seed()` to write the store before launch so a
run does not sit through the setup conversation.

One-off installs, neither of which ships: `sudo apt install
webkitgtk-webdriver` (**not** the `webkit2gtk-driver` the Tauri docs
name) and `cargo install tauri-driver`.

- Writes to a throwaway `XDG_DATA_HOME` by default. Leave it. An
  automated pass once wrote window geometry into the real settings and
  the app opened wrong afterwards.
- A scenario that talks to the gateway leaves **real conversations** in
  the owner's list. Delete the ones you created, and only those.
- The window opens on the real display, so it is visible to whoever is
  at the machine.
- `log()` returns both halves labelled: `[app]` from the shell's log
  file, `[page]` from the console, captured in-page because WebKitGTK
  does not expose it. Only what happens after start is in `[page]`.

Chromium is not the engine the device ships. It agreeing with the design
is evidence about Chromium — use `make ui-drive` before claiming the
device renders something correctly.

### The browser — layout alone

Playwright MCP: navigate, screenshot, `Read` the image. If the
screenshot cannot explain a miss, evaluate `getComputedStyle` on the
node. The MCP is not always registered; a local Playwright driving
Chromium does the same job.

Nothing native exists here, so `invoke` fails and the app redirects to
setup or renders empty. Stub `window.__TAURI_INTERNALS__.invoke` before
the app's scripts run. Two that cost real time: `plugin:store|get`
returns a `[value, exists]` **tuple**, not a bare value, and
`plugin:store|load` must return a resource id, not null. Both fail by
silently redirecting to the language screen.

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
