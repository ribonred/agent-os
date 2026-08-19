---
name: brain-contract
description: >
  Change constitution, onboarding, shipped device skills, or how the
  agent learns the owner. Use for brain/, knowledge-store schema,
  agent-core/ingest, registry when-to-use text, or any prompt the
  device agent will follow.
---

# Brain contract

`brain/` is the device agent's behavior. Coding agents edit it as
product prose, not as a scratchpad.

## When to Use

- `brain/constitution.md`, `brain/onboarding*.md`
- `brain/skills/` (these **do** ship)
- `agent-core/ingest/`, `agent-core/knowledge-store/schema.sql`
- Adding a skill that will be copied onto the unit

## Layers (do not collapse)

| Layer | File | Changes with owner? |
|---|---|---|
| Identity / invariants | `brain/constitution.md` → `SOUL.md` | No. Re-pinned every boot. |
| Overlay | name, language, optional persona | Yes. Shell appends per turn. |
| Owner profile | Hermes `USER.md` via `memory` `user` | Yes, after confirm. |
| Device facts | Hermes `MEMORY.md` via `memory` | Yes, live checks only. |
| Business facts | Postgres `entities/sources/facts` | Yes, confirmed rows. |
| Procedures | `brain/skills/*/SKILL.md` | No, except you ship an update. |

Constitution never holds tool lists, service versions, or the owner
profile. Skills never belong in SOUL.md.

## Onboarding

Specified in `brain/onboarding.md`; executed by the shell step driver in
`ui/src-tauri/src/onboarding.rs` (contract: `onboarding-protocol.md`).

- Deterministic UI first: language, then agent name. Not LLM questions.
- Shell checklist: owner_name, role, needs, vocabulary, boundaries,
  communication, confirm. Hermes only phrases the current open step.
- Name locks after first answer in shell state.
- Never infer what was not said. Unresolved is valid. Guessing is not.
- Fresh devices: Balanced voice, no persona screen. Legacy persona
  overlay is upgrade-only.
- Silent Postgres/Redis checks run once in the shell before turn 1.
- On accept, the shell writes `USER.md` from structured answers.

## Knowledge store

`agent-core/knowledge-store/schema.sql` is append-only facts. Current
value = latest **confirmed** row per entity+field. `value` NULL means
"not yet known." Do not upsert-overwrite.

`agent-core/ingest` parsers exist for csv/docx/xlsx. `main.py` is still
a stub. Extraction is schema-constrained JSON from a local or cloud
model via `hw-probe`'s routing decision — do not reimplement routing
in Python.

## Shipped skills

Only `brain/skills/` is installed onto the unit (all folders ship:
`device-services`, `device-apps`, `owner-files`, `business-records`,
`document-ingest`, `confirm-before-harm`). First-boot copies each
`SKILL.md` into `$HERMES_HOME/skills/<name>/` and into
`/usr/local/share/agentic-os/skills/`. The image installer (`install-hermes.sh`)
glob-copies every `brain/skills/*/` folder, so a new skill must live there
to ship — it needs no per-skill entry once the glob is in place.

A shipped skill must:

- Speak in owner outcomes, not Ubuntu internals
- Use live checks, never versions from yaml or memory alone
- Bundle non-interactive scripts if the steps are fragile
- Stay out of vertical domains (clinic, POS, finance)

`.claude/skills/` is the opposite set: developers only.

## Gotchas

- Small models run onboarding. Compound questions lose answers.
  One question, then stop.
- Redis is ephemeral. Never store the profile or lasting facts there.
- `registry/*.yaml` is metadata, not prompt text.
- `hw-probe` GPU/NPU detection reads sysfs. It is the only routing
  implementation; ingest shells out to the binary.
