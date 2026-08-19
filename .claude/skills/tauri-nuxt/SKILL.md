---
name: tauri-nuxt
description: >
  Change the Tauri 2 Rust host, capabilities, window modes, invoke
  commands, or Nuxt/WebKitGTK wiring. Use for ui/src-tauri/,
  tauri.conf.json, capabilities, make gui/ui-bundle, invoke(), webview
  zoom, kiosk vs pill window — even if the user said desktop app or
  native shell.
---

# Tauri 2 + Nuxt

Official integration: https://v2.tauri.app/start/frontend/nuxt/

This app is a static WebKitGTK webview. Nuxt is `ssr: false`. The
packaged binary loads `ui/dist`. Dev polls `http://localhost:3000`.

## When to Use

- `ui/src-tauri/**`, `ui/tauri.conf.json`, capabilities
- New `#[tauri::command]`, plugins, window geometry
- "invoke failed", blank window, wrong scale, pill vs full
- Mixing frontend and Rust ownership

For visual tokens, orb, and conversation chrome use `agentic-ui` and
`product-ux` instead. For Hermes session/model/memory use
`hermes-gateway-client`.

## Ownership split

| Lives in Rust | Lives in the webview |
|---|---|
| Window geometry (full / pill) | Layout inside the webview |
| Bearer token, gateway HTTP | Tokens of assistant text |
| OS keyring cloud key | Whether the key-status screen shows |
| `settings.json` via plugin-store | Reading those prefs through commands |
| Restore window mode before paint | Pin zoom to 1.0 on mount |

One window, reconfigured. Never a second webview for the pill — that
would fork conversation state. See `ui/src-tauri/src/window_mode.rs`.

The frontend must not gain window-move permissions to drag itself.
Drag/resize commands stay in Rust (`window_drag`, `window_mode_set`).

## Config that must stay in step

- `ui/nuxt.config.ts` `devServer`: host `127.0.0.1`, port `3000`,
  `strictPort: true`
- `ui/src-tauri/tauri.conf.json` `build.devUrl`: `http://localhost:3000`
- Makefile `UI_DEV_URL` / `HERMES_URL`

`ssr: false`. `src-tauri` and `*.test.ts` are ignored by Nuxt typecheck.

Capabilities (`ui/src-tauri/capabilities/default.json`) stay tiny:
`core:default`, opener, log, store, webview zoom, window start-dragging.
Do not add filesystem or HTTP from the webview. Chat and keys go
through `invoke`.

## Commands

Register every new command in `ui/src-tauri/src/lib.rs`
`generate_handler!` and call it from Vue with:

```ts
import { invoke } from "@tauri-apps/api/core";
await invoke("command_name", { camelCaseArgs: value });
```

Existing groups: `agent_*`, `cloud_key_*`, `approval_mode_*`,
`window_mode_*`, `sessions_*`, `shelf_*`, `dev_reset_setup`.

Secrets: OpenRouter key → OS keyring (`cloud-key` crate), never
plugin-store. plugin-store is language, persona, window mode, pill
position only.

## Linux / WebKit gotchas (this product)

On a compositor with no settings daemon, GTK resolution stays `-1`
and WebKit scales the page to garbage. All three are required:

1. `display.default_screen().set_resolution(96.0)` before any webview
2. Nudge `gtk-xft-dpi` to `96*1024` after (change notification)
3. `getCurrentWebview().setZoom(1.0)` in `ui/app/app.vue` on mount

Do not remove one "because the others should be enough." They were
verified by removing them one at a time on a VM.

`make ui-drive` prints `devicePixelRatio` every run for exactly this
reason. A browser always reports a healthy value, so it can never tell
you this is broken.

`make ui-bundle` uses `env -i` + system rustup/bun and
`--no-bundle`. The binary is copied into the image and links
distro GTK/WebKit. Do not rewrite RPATH. Do not build inside Nix.

Window modes (DESIGN.md):

- **Full** = maximized, undecorated — not exclusive fullscreen (that
  hides the system bar the owner may need).
- **Minimized** = floating always-on-top pill, same conversation.
- Unknown stored mode → Full.

Apply geometry in the Tauri `setup` hook so the first frame is
already the right shape.

## Procedure

1. Decide which side of the split owns the change.
2. If adding a command: Rust handler + `generate_handler!` + Vue
   `invoke` + capability only if a new permission is truly required.
3. `cd ui && bun run check` and `cd ui/src-tauri && cargo test`
   (or `make test`).
4. Anything on this page is the native half, so prove it with
   `make ui-drive` rather than a browser: it drives the built binary
   over WebDriver and can read what a command actually returned, what
   `devicePixelRatio` really is, and which shape the window took. A
   browser stubs all of that and will agree with you regardless.
5. For pure layout/scale claims, follow `agentic-ui`'s isolated
   Playwright rule. Never start a second Vite server on this tree.

## Gotchas

- There is one Tauri config. A kiosk overlay config used to exist and
  was deleted with the compositor it served; the device and a dev
  machine now run the same binary and differ only in stored state. Do
  not reintroduce a second config or a second product window — full vs
  pill is `window_mode.rs`.
- `authors = ["you"]` in Cargo.toml is leftover template. Do not
  replace it with a personal name (commercial-hygiene).
- Identifier is `com.agenticos.shell`.
- `dev_reset_setup` is a dev command. Do not expose it on a shipped
  owner path.
---

Official pages used while writing this: Tauri Nuxt guide
(https://v2.tauri.app/start/frontend/nuxt/), IPC
(https://v2.tauri.app/develop/calling-rust/).
