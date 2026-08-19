# Onboarding protocol (shell-driven)

The shell owns onboarding. This file is the product contract for that
driver — not a free-form system prompt the model improvises from.

## Who owns what

| Concern | Owner |
|---|---|
| Step list, cursor, answers, done flags | Shell (`onboardingState`) |
| Silent Postgres / Redis checks | Shell, once, before turn 1 |
| Final `USER.md` profile write | Shell, after owner accepts summary |
| Phrasing the **current** open step | Hermes |
| Choosing the next step | Shell only |

Hermes never decides that onboarding is complete. Hermes never re-opens a
step the shell has marked done.

## Mandatory steps

1. `owner_name` — what to call the owner (or declined / shared)
2. `role` — who they are and what this device is for
3. `needs` — concrete tasks they want help with
4. `vocabulary` — day-to-day terms and important entities
5. `boundaries` — sensitivities / off-limits
6. `communication` — formality, detail, pace
7. `confirm` — shell-known facts summarized; owner accepts

Each non-confirm step becomes **done** on the first clear answer,
including yes/no, decline, or "not sure." Thin answers are enough.
Unresolved is valid. Guessing is not.

## Name lock

`owner_name` is asked only while it is the current step. After the first
answer it is locked in shell state and listed as a known fact. Hermes is
told not to ask it again; the shell will not make it current again.

## Per-turn Hermes brief

Each turn the shell injects only:

- identity / language overlay from setup
- locked facts already answered
- the single current step and how to ask it
- the chat answer-offering convention

Hermes must:

- ask exactly one question, then stop at the question mark
- not run tools, load skills, check services, or search sessions
- not write memory during setup
- not invent a next topic

## Device checks

Before the greeting, the shell probes Postgres and Redis live and writes
successful facts to Hermes `MEMORY.md`. Failures stay silent. Hermes is
not asked to discover the device during onboarding.

## Completion

When the owner accepts the confirm summary, the shell:

1. writes a compact `USER.md` from structured step answers
2. sets `onboardingComplete`
3. streams a **fixed** closing line (owner name + assistant name,
   language-aware) so finish is never blank or improvised
4. drops the onboarding session so normal chat starts fresh

Setup is complete when the write succeeds and `onboardingComplete` is
set — not when the model decides to say goodbye.

