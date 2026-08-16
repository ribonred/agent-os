---
id: "027"
title: "Onboard the owner and agent together"
status: pending
priority: high
effort: large
type: feature
phase: agent-core
dependencies: []
tags: [onboarding, hermes-agent, memory, postgres, redis, ui]
created_at: 2026-07-19
context:
  - brain/onboarding.md
  - brain/constitution.md
  - modules/hermes-agent.nix
  - registry/postgres.yaml
  - registry/redis.yaml
  - ui/app/lib/setupStore.ts
  - ui/src-tauri/src/agent.rs
---

# Onboard the owner and agent together

## Objective

Turn first boot into one continuous setup in which the owner configures the
agent while the agent learns the minimum reliable context needed to help the
owner. The completed flow is:

1. Select language in deterministic UI.
2. Give the agent a name in deterministic UI.
3. Enter a dedicated guided conversation where the agent resolves the five
   unknowns in `brain/onboarding.md`, confirms the resulting profile, learns
   its available device services, and persists both sides of that knowledge.
4. Reach the normal home screen only after the owner accepts the profile and
   every required write succeeds.

Fresh devices skip the separate persona screen. Balanced is the initial voice;
the conversation's communication-style unknown personalizes it from there.
Existing devices keep an already-selected persona so an upgrade does not
silently discard an owner's choice.

## Architecture decisions

### Keep identity, profile, environment, and procedures separate

- `SOUL.md` remains the immutable identity and behavior contract. Do not put a
  changing tool list, service versions, or owner profile into it.
- Hermes `USER.md` is the compact, always-available owner profile: identity,
  role, recurring needs, vocabulary, boundaries, and communication preference.
  Write it with Hermes' built-in `memory` tool using the `user` target.
- Hermes `MEMORY.md` holds stable device facts the agent needs while operating:
  available services, verified versions, and durable environment constraints.
  Write it with the `memory` target.
- Redis remains ephemeral infrastructure for caches, queues, and session state.
  Never store the owner profile or any other durable onboarding result in it.
- Ship device-service operating instructions as a Hermes skill. A skill is the
  upstream-supported home for procedures built from existing terminal tools;
  it should explain when and how to use Postgres and Redis and how to verify an
  operation. The registry YAML remains product metadata, not prompt text.

### Learn the device before promising capabilities

At the start of guided onboarding, run deterministic checks through the Hermes
terminal tool and capture command, parsed version, health, and check time:

- Postgres health: `pg_isready -h /run/postgresql`
- Postgres server version: `sudo -u postgres psql -Atqc 'SHOW server_version'`
- Redis health: `redis-cli -h 127.0.0.1 ping`
- Redis server version: `redis-cli -h 127.0.0.1 INFO server`, parsing the single
  `redis_version:` field rather than storing the full response

The implementation must prove these commands work as the `hermes` service user
on the device image. Save only verified results to `MEMORY.md`; never infer a
version from Nix source or a registry declaration. Re-check before a task if a
service reports unhealthy or its remembered version may be stale.

### One orchestrated onboarding flow

The shell owns progress and completion; the model does not decide whether setup
is complete. Fresh devices follow:

`language -> name -> guided conversation -> complete`

The interview remains model-generated within the existing 5-to-15-question
bound. The shell tracks question count and supplies a narrow onboarding system
instruction. The agent asks one unknown at a time and adapts only from confirmed
answers already in the conversation.

After at least five questions, or at the fifteen-question ceiling, the agent
shows one plain-language profile summary. The owner can correct it in the same
conversation. Only explicit acceptance permits the agent to write the profile.

### Native Hermes persistence

The final confirmed turn uses Hermes' built-in `memory` tool:

1. Write one compact atomic batch to the `user` target for the five owner-profile
  fields. This is the canonical profile; do not duplicate it in Postgres or the
  shell's settings store.
2. Write successful service probes to the `memory` target with their check time.
  Unavailable services remain unsaved rather than guessed.
3. Persist `onboardingComplete` only after the session transcript proves the
  `user` write returned `success: true` and was not staged for approval.

Hermes memory is injected as a frozen snapshot when a session starts. Normal
chat therefore begins in a fresh session after onboarding completes.

## Tasks

- [ ] Update `brain/onboarding.md` first with the two-screen setup order,
      mutual-onboarding state machine, and persistence targets.
- [ ] Update `design/DESIGN.md` with the dedicated interview, profile review,
      and service-check status before writing UI code.
- [ ] Change fresh-device setup routing to language -> name -> onboarding;
      preserve existing persona preferences as an upgrade compatibility path.
- [ ] Add fresh-device onboarding-started/completed markers and a shell-owned
  question count without persisting a second profile copy.
- [ ] Add a constrained onboarding protocol around Hermes session chat; the
      shell, not free-form model output, owns transitions and completion.
- [ ] Enable both `memory.memory_enabled` and
      `memory.user_profile_enabled` in Hermes configuration and verify the
      `memory` tool is exposed to sessions used by the UI.
- [ ] Add a device-services Hermes skill covering Postgres/Redis purpose,
      boundaries, health/version checks, safe usage, and verification.
- [ ] Run the service probes as the real `hermes` system user and persist only
      successful parsed results to the `memory` target.
- [ ] Add focused unit/integration tests for bounds, one-unknown-at-a-time,
  confirmation, transcript-proven memory completion, and legacy migration.
- [ ] Visually verify every first-boot phase with the repository's isolated
  static-build Playwright workflow, then run one device-level pass.

## Acceptance Criteria

- A fresh device follows language -> name -> guided onboarding -> review -> home,
  while an upgraded device retains any prior persona choice.
- The agent asks 5 to 15 generated questions, one unknown at a time, and never
  stores an unconfirmed inference as fact.
- After restart, Hermes recalls the compact confirmed owner profile from
  `USER.md` without a duplicate profile in Postgres or the settings store.
- The agent can name Postgres and Redis as available only after live health
  checks, and `MEMORY.md` contains their live server versions and check time.
- Redis contains no durable owner profile or onboarding record.
- Postgres/Redis procedures are available through a Hermes skill; `SOUL.md`
  remains limited to identity and invariant behavior.
- Existing devices with language, persona, and name are not forced through the
  new interview after an upgrade.