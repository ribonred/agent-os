---
id: "016"
title: "Tauri GUI: OpenRouter API key entry / cloud-connect screen"
status: in-progress
priority: medium
effort: large
phase: ui-shell
dependencies: ["012"]
tags: ["tauri", "gui", "openrouter"]
created_at: 2026-07-13
---

# Tauri GUI: OpenRouter API key entry / cloud-connect screen

## Objective

Let the user connect (and disconnect) an OpenRouter API key from the UI,
stored in the OS keyring per DESIGN.md -- never in the plain-file store.
This is the missing input behind hw-probe's has_cloud_credentials, which
is hardcoded false until something real exists to read.

## Tasks

- [x] Storage decision, researched not assumed: Stronghold (the old
      official Tauri answer) is deprecated for removal in v3; OS keyring
      is the current recommended path. Chose the `keyring` Rust crate
      (v4, zbus Secret Service backend on Linux) via three custom
      commands instead of a community plugin, for one deliberate reason:
      community plugins expose a generic get_password to the webview --
      exactly backwards. Our posture: the key goes in and never comes
      back out to JS. There is no command that returns the secret; the
      frontend only learns configured yes/no. Consumers that need the key
      (agent runtime) read the keyring directly on the Rust side.
- [x] keyring v4's API verified against the actual vendored crate source
      (v4 renamed features and restructured modules vs v3 -- Entry::new/
      set_password/get_password/delete_credential under the v1 feature,
      Error::NoEntry from keyring-core).
- [x] src-tauri/src/cloud_key.rs -- cloud_key_save (rejects empty),
      cloud_key_exists (NoEntry -> false, other errors loud),
      cloud_key_delete (idempotent: deleting an absent key is success).
      cargo check clean.
- [x] NixOS host: services.gnome.gnome-keyring.enable = true -- Secret
      Service needs a daemon on the device or saving fails. Full host
      closure rebuilt and confirmed (real exit code, not piped away).
      PAM auto-unlock on login is a noted follow-up once the device's
      session flow is decided.
- [x] /settings/cloud screen in the established visual world (orb,
      bilingual eyebrow "Hubungkan cloud · Optional", honest copy: works
      fully offline, cloud is optional reach). Both states screenshotted
      via the browser-side IPC fake: unconfigured (password input +
      Connect, disabled until non-empty) and connected (green status dot,
      "Disconnect and remove the key"). Errors surface loudly in the UI
      via role=alert, not swallowed.
- [x] Quiet "Cloud settings" link on the home screen (bottom corner, no
      nav chrome yet). Also confirmed during validation: the mandatory
      setup gate correctly guards /settings/cloud too -- an unconfigured
      device redirects to language setup even when deep-linked.
- [x] svelte-check clean (146 files, 0 errors).
- [x] Vendor-provisioned key file (added on request): the device can ship
      with cloud pre-configured so the buyer never creates an API
      account. /etc/agentic-os/cloud-keys.toml (root-owned 0600, per-
      provider TOML sections so future providers slot in), read as a
      fallback -- keyring (explicit user action) always wins; UI
      "disconnect" removes the keyring entry and falls back to the
      provisioned key, it cannot delete the file. Loud-error rules:
      missing file = normal (nothing provisioned); existing-but-unreadable
      or malformed file = error, never silently "no key"; blank api_key =
      provisioning mistake, counts as absent; group/other-readable file
      logs a permissions warning. Dev override via
      AGENTIC_OS_CLOUD_KEYS_FILE env var -- which also gives Red a working
      cloud-key path on WSL where no keyring daemon runs.
      resolve_openrouter_key() is the Rust-side accessor for the future
      orchestrator; deliberately NOT a Tauri command.
      UI now three states via cloud_key_status (none | keyring |
      provisioned) -- provisioned state screenshotted ("Cloud came
      pre-configured on this device." + use-your-own-key override).
      5 Rust unit tests, all passing: valid parse, missing section,
      blank key, malformed TOML errors loudly, real file read through
      the env override. Host config documents the provisioning
      expectation (sops-nix/agenix to own writing the file once factory
      flow is designed) -- the key itself never committed.

## Acceptance Criteria

- [x] Key stored in OS keyring, not tauri-plugin-store or any file
      (user-entered path; vendor-provisioned file is a deliberate,
      documented second source with keyring precedence)
- [x] Key can never round-trip to the webview -- no command returns it
- [x] All three UI states visually verified by screenshot
- [ ] Real keyring round-trip (save -> exists -> delete) against a live
      Secret Service daemon -- not validatable in this sandbox (no
      gnome-keyring running, and real Tauri IPC needs the actual app
      window). Needs Red's `bun run tauri dev` pass: save a key, restart,
      confirm still connected, disconnect.
- [x] Follow-up (belonged to task 008's orchestrator wiring): done there.
      The key logic moved into the shared agent-core/cloud-key crate;
      the orchestrator daemon resolves real key presence per request, so
      has_cloud_credentials is no longer hardcoded anywhere. The Tauri
      commands here became thin wrappers over the same crate.
