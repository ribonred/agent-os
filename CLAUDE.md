# agentic-os

Base OS + agent runtime for MSI-bundled AI-assistant devices (NixOS track).
This is commercial product code that ships to customers, not a personal
dev project — treat every file as something a customer's device could
expose or a future team member could read with no shared context.

## Project goals & context

Sold hardware bundling an AI assistant, targeting non-technical daily
users (a doctor's front desk, a POS counter, a skincare shop, personal
money management) — the product is "buy simplicity," not "buy an AI
platform." Two hardware tiers: a mini-PC tier running full NixOS (primary
dev target, currently a SWNUC11PAHi3000), and DGX Spark as a secondary/
stretch tier where Nix sits as an app layer on stock DGX OS rather than
replacing it (DGX OS is vendor-validated for that hardware; NixOS is not).

The device ships with no domain knowledge — a generic base image, not a
per-vertical build. Users teach it their own business after purchase,
through a guided onboarding conversation (`brain/onboarding.md`) and by
handing it documents (`agent-core/ingest`). LLM strategy is hardware-tier
routing, not a fixed choice: `agent-core/hw-probe` detects what the
device can actually run and leans local (Ollama + Hermes) or cloud
(OpenRouter, defaulting to Hermes too, for behavioral consistency across
the switch) accordingly — never hardcoded per SKU.

## Repo map

- `hosts/`, `modules/tool-registry/`, `registry/` — the NixOS host config
  and declarative infra (Postgres, Redis, Ollama+Hermes)
- `brain/constitution.md` — the shipped agent's actual system prompt/
  behavior spec; `brain/onboarding.md` — how it learns a new user's
  business, including the mandatory language/persona setup questions
- `agent-core/` — real runtime code: `hw-probe` (Rust, hardware/routing
  detection), `ingest` (Python/uv, document parsing + extraction),
  `knowledge-store` (Postgres schema)
- `design/DESIGN.md` — the UI's color/typography/motion system; change
  this file first, implementation follows it, never the other way round
- `ui/` — the Tauri + Nuxt (Vue) shell. Developed against the system's own
  rustup/cargo-tauri/bun toolchain, not the Nix devShell (`ui/.envrc`
  opts this subtree out of the parent repo's `use flake` on purpose —
  Tauri's GTK/webkit deps aren't worth fighting into Nix for local dev;
  packaging the built app for the actual NixOS device is separate,
  later work)
- `tasks/` — taskmd. Check `taskmd list` / `taskmd next` at the start of
  a session before doing anything else; don't re-derive project state
  that's already tracked there.

## Commercial-code hygiene (non-negotiable)

- **No personal names, usernames, or identifying info** anywhere in code,
  config, or comments. System accounts get generic names (`admin`, not a
  developer's username). No email addresses, no "my machine" references.
- **No references to internal tooling/process by name** in code comments
  — e.g. don't cite an internal task-tracker's ticket IDs, or name an
  internal engineering-process/skill doc as the source of a rule. If a
  rule matters enough to justify in a comment, restate the reasoning
  itself in plain terms so it stands on its own to an outside reader.
- **Explaining the "why" is encouraged, not discouraged.** The rule above
  is about who/what gets named, not about removing rationale. A comment
  like "unix socket only -- nothing on this box needs network pg yet" is
  exactly right. A comment like "see red-mindset" or "tracked as task 002"
  is not, because it leaks internal context and won't mean anything once
  this ships.
- Project-management artifacts (the `tasks/` directory, taskmd) are
  internal and not part of what ships — task IDs are fine *within that
  directory*, just don't bleed them into source comments elsewhere.

Apply this on every edit, not just when asked.

## UI validation — Playwright MCP

`nuxt typecheck` and a successful build catch type/syntax errors. They do
not catch layout or visual bugs — a component can typecheck cleanly and
still render nothing like what the CSS says it should. For any change to
`ui/` where correctness is a visual/layout claim ("it's centered," "the
glow renders," "the button is visible"), verify it by actually looking,
not by inferring from a clean build.

**What it is**: Playwright MCP — browser automation tools (navigate,
screenshot, evaluate JS/computed styles, click, fill forms). Tool names
are deferred; load them first with
`ToolSearch("select:mcp__plugin_playwright_playwright__browser_navigate,...")`
before first use each session — they don't appear in the default tool
list.

**Workflow**: navigate to the running page, take a screenshot, `Read` it
as an image. If something looks wrong and the screenshot alone doesn't
explain why, use `browser_evaluate` with `getComputedStyle` on the
relevant element to see what actually applied versus what the source
says — the mismatch tells you whether it's a real code bug or something
else entirely (see the cache note below).

**Boundaries**:

- **Never touch the user's own interactive `bun run tauri dev` session**
  — don't kill it, don't rely on it as the thing you're validating
  against. And "touch" includes the caches: a second vite dev server on
  this tree shares `.nuxt/`, `dist/` and `node_modules/.vite/` with the
  user's running one, and concurrent servers clobber each other's module
  graphs — the user's app then randomly serves components whose markup
  and scoped CSS come from different compiler generations (layout
  collapses, styles match nothing, "sometimes broken sometimes not").
  This happened for real and cost a long debugging session. Therefore:
  **while the user's dev server is running, never start another vite dev
  server on this tree and never delete its caches** — verify against a
  static `vite build` output on a throwaway port, or ask first. When no
  user server is running, an isolated throwaway server is fine — kill it
  and delete any screenshots/`.playwright-mcp/` output when done.
  Nothing from a verification pass belongs in a commit.
- **Suspect a stale Vite/Nuxt cache before a real bug** if a fresh
  change doesn't render as expected, especially after adding a new route
  file (e.g. a new page) — a long-running dev server can fail to
  pick up structural changes via HMR. Clear `.nuxt/` and
  `node_modules/.vite/` (only when no dev server is running — see
  above), retest on a freshly started, isolated server, *then* trust
  the result either way. This already happened once: a new
  `+layout.svelte` wasn't picked up by a running server, and what looked
  like a centering bug in the CSS was actually a stale style transform.
  Release builds are immune since `make ui-bundle` cleans caches and
  fails on a scope-hash mismatch (the canary check).
- Read-only checks (navigate/screenshot/evaluate) are the default use.
  Interaction (`click`/`fill_form`/etc.) is fine when a task specifically
  requires testing a flow, not as a default habit.
