# agentic-os — instructions for AI coding agents

Base OS + agent runtime for MSI-bundled AI-assistant devices (NixOS
track). Commercial product code that ships to customers — every file is
something a customer's device could expose or a future contributor could
read with zero shared context. See `CLAUDE.md` at the repo root for the
full context (project goals, repo map, commercial-code hygiene rules,
Playwright-based UI validation workflow) — this file exists so tools that
read `.github/copilot-instructions.md` specifically get the same
grounding without duplicating it here.

## The short version

- Two hardware tiers: mini-PC on full NixOS (primary dev target), DGX
  Spark on stock DGX OS with Nix as an app layer (secondary/stretch) —
  see `CLAUDE.md` for why.
- Device ships generic; users teach it their business via
  `brain/onboarding.md`'s conversation and `agent-core/ingest`'s document
  parsing. Never bake vertical-specific knowledge into the shipped image.
- LLM routing is hardware-tier-detected (`agent-core/hw-probe`), not
  hardcoded — local (Ollama+Hermes) or cloud (OpenRouter+Hermes) depending
  on what the device can actually run.
- `ui/` is developed against the system's own Rust/bun toolchain, not the
  repo's Nix devShell — see `ui/.envrc` and `CLAUDE.md`'s repo map.
- Before writing UI code and claiming it's correct: verify visually with
  Playwright MCP (screenshot + computed-style inspection), not just a
  clean typecheck/build. Full workflow and boundaries are in `CLAUDE.md`.
- No personal names, no internal-tooling references in code/comments —
  see `CLAUDE.md`'s commercial-code hygiene section for the full rule.
- Check `taskmd list` / `taskmd next` before assuming project state;
  `tasks/` is the source of truth for what's done and what's next.
