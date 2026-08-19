# Development skills

These are for coding agents working on this repository (Claude Code,
GitHub Copilot, and anything else that loads Agent Skills from
`.claude/skills/`).

They do **not** ship on the device. Owner-facing skills live in
`brain/skills/` and are copied into the image by
`build/scripts/install-hermes.sh`. Do not move these folders there.

Each skill is one job. Load the matching one before changing that area.
The always-on product rules stay in `CLAUDE.md`.

| Skill | When |
|---|---|
| `taskmd` | Session start, what's next, `tasks/` |
| `golden-image` | Image, packages, first-boot, registry |
| `agentic-ui` | `ui/` screens, tokens, verifying by looking, `make dev` |
| `tauri-nuxt` | Rust host, invoke, window modes, WebKitGTK |
| `product-ux` | Orb, conversation, setup, copy, anti-slop |
| `commercial-hygiene` | Anything that could ship |
| `hermes-gateway-client` | Gateway, model, memory, onboarding chat |
| `brain-contract` | `brain/`, ingest, shipped `brain/skills/` |
