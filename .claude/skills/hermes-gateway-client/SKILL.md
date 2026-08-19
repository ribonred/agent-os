---
name: hermes-gateway-client
description: >
  Change how the shell talks to Hermes: sessions, streaming chat,
  onboarding turns, memory scope, model id, overlays, or the gateway
  service. Use for ui/src-tauri/src/agent.rs, install-hermes.sh,
  hermes-gateway.service, SOUL.md pinning, or "chat is broken."
---

# Hermes gateway client

The UI is a thin client of `hermes gateway` on loopback
`http://127.0.0.1:8642`. The webview never calls the gateway. Tauri
commands proxy it so the bearer token never enters the web context.

## When to Use

- `ui/src-tauri/src/agent.rs` and related Rust
- `build/scripts/install-hermes.sh` config, unit, identity copies
- Onboarding chat protocol wiring
- Model / memory / approval settings for the device runtime

## Invariants

**Model id.** Sessions must be created with a real model id. A session
opened without one is stamped `hermes-agent` (a profile label). The
provider then rejects every turn. Default is duplicated in:

- `build/scripts/install-hermes.sh` → `config.yaml` `model.default`
- `ui/src-tauri/src/agent.rs` → `model_id()` fallback

Keep them identical. Override later via `AGENTIC_OS_HERMES_MODEL`.
Current fallback: `deepseek/deepseek-v4-flash-0731`. Bare OpenRouter
id, no `openrouter/` prefix.

**Memory scope.** `agentic-os:device:main` — stable across app
restarts. Owner profile → Hermes `memory` tool, `user` target
(`USER.md`). Device facts → `memory` target (`MEMORY.md`). Do not
duplicate the profile in Postgres or `settings.json`.

**Session resume / history.** Hermes capabilities matter here:

- `GET /api/sessions/{id}/messages` — durable transcript for the UI
- `POST /api/sessions/{id}/chat` and `.../chat/stream` — **load DB
  history** via `get_messages_as_conversation`, then run one turn
- `POST /v1/runs` — `session_id` is correlation only unless the client
  also sends `conversation_history` (or `previous_response_id`)

The shell therefore runs owner chat turns on
`/api/sessions/{id}/chat/stream`, not bare `/v1/runs`. Reopening a
conversation only sets the active session id and hydrates the pane;
the next turn must hit the session chat path or Hermes answers as if
the chat were empty. Stop/approval still use `/v1/runs/{run_id}/…`
using the `run_id` from the session stream's `run.started` event.

**Prompts.** Chat answer-offering lives in `brain/chat-protocol.md` and
is `include_str!`'d. Onboarding is **not** a long free-form protocol
injection anymore: `ui/src-tauri/src/onboarding.rs` owns the checklist
and builds a per-turn step brief. Product contract prose stays in
`brain/onboarding-protocol.md` / `brain/onboarding.md`. Overlay texts
(name, language, persona) are specified in `brain/onboarding.md` and
mirrored in `agent.rs`. Change the markdown first for those.

**SOUL.md.** `brain/constitution.md` is installed as
`$HERMES_HOME/SOUL.md` (directly in Hermes home, not a nested
`.hermes/`). First-boot re-pins it every boot. Do not put the owner's
name, persona, or service versions in the constitution.

**Onboarding completion.** The shell owns it end-to-end. Steps and
answers live in `onboardingState`. Silent device checks and the final
`USER.md` write are shell-side. After the owner accepts confirm, the
shell writes `USER.md`, sets `onboardingComplete`, and drops the
onboarding session so normal chat starts fresh with the new snapshot.

**Approvals** (device `config.yaml`): `mode: "off"` with a tiny `deny`
list — `/etc/agentic-os/hermes.env` and `cloud-keys.toml`. Do not
broaden deny to config.yaml; the owner has no other admin surface.

**Network.** Gateway is loopback only. Full toolset including terminal
is behind that bearer token.

## Dev

```
make hermes-env
# AGENTIC_OS_HERMES_URL default http://127.0.0.1:8642
# key from AGENTIC_OS_HERMES_KEY, /etc/agentic-os/hermes.env, or ~/.hermes/.env
```

`make dev` warns if `/health` fails; chat will not work until
`hermes gateway` is up.

## Gotchas

- Config path is `$HERMES_HOME/config.yaml`. An `/etc/...yaml` copy is
  not read.
- `HERMES_ENVIRONMENT_HINT` on the systemd unit states the appliance
  boundary: use Ubuntu silently, talk in device/task language.
- Agent working directory is the owner's home. Agent state stays in
  `$HERMES_HOME`. Do not clutter Documents/Downloads with runtime files.
- The gateway service is a **system** unit (must be up before login),
  not upstream's user messaging service. No systemd sandbox: sudo and
  the owner's files must remain reachable. Supplementary groups
  `render` `video` so GPU is not silently CPU-only.
