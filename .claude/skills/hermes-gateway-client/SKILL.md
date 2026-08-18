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

**Prompts.** Prose lives in `brain/` and is `include_str!`'d at
compile time:

- `brain/onboarding-protocol.md`
- `brain/onboarding-start.md`
- `brain/onboarding-resume.md`

A missing file is a build error. Edit those files, then the Rust only
if the contract changed. Overlay texts (name, language, persona) are
specified in `brain/onboarding.md` and mirrored in `agent.rs`. Change
the markdown first.

**SOUL.md.** `brain/constitution.md` is installed as
`$HERMES_HOME/SOUL.md` (directly in Hermes home, not a nested
`.hermes/`). First-boot re-pins it every boot. Do not put the owner's
name, persona, or service versions in the constitution.

**Onboarding completion.** The shell owns it. After the owner accepts
the profile, the agent must land a successful `memory` write
(`user` target). Only then set `onboardingComplete`. Start normal chat
in a **new** Hermes session so the new USER.md / MEMORY.md snapshot is
injected.

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
