---
name: business-records
description: >
  Store or recall what the owner has taught about their work — the
  durable facts the assistant keeps so it can help them. Use whenever
  the owner teaches something about their business, asks you to remember
  it, or when a later answer draws on what they have told you before.
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [facts, knowledge, records, memory, durable]
---

# Business Records

The assistant keeps a durable memory of what the owner has told it about
their work — their names for things, their products, their people, their
day-to-day facts — so it can be genuinely helpful later instead of asking
the same thing twice. This skill governs how those records are written,
changed, and read.

Use it whenever the owner **teaches** something about their work, **asks
you to remember** it, or when an answer you are giving **relies on a fact
you were told earlier**.

## How records work

Every record is a small fact attached to a real thing (the business
itself, a product, a service, a staff member) and traced back to the
owner's own words. The rules:

- **Only store what the owner said.** A fact the owner has not confirmed
  stays marked unconfirmed. An unknown value is a legitimate state —
  never guess at it, and never invent a fact to fill a gap.
- **Keep the owner's words as the source.** Every fact is traceable to
  what the owner actually said, so a later check can see where it came
  from, not just a paraphrase of it.
- **Confirmed only after the owner says yes.** Reflect a learned fact
  back in plain language and get explicit confirmation before trusting
  it, the way onboarding does.
- **New fact, not overwrite.** When a detail changes (a price goes up,
  hours change), the current fact is the latest one the owner confirmed
  from that point. History is kept — a change never destroys what used
  to be true.
- **An empty value means “not yet known”,** never a reason to guess.

## What does NOT go here

- **The owner's profile stays in its own place** (the always-available
  owner summary), not in these business records.
- **Never store durable records in fast, temporary memory.** Anything
  that must survive restart or stay queryable belongs in durable
  storage, never in ephemeral cache.
- This is for the owner's business knowledge, not for transient working
  state or session scratch.

## When to use

- The owner says “remember that my shop closes at six”.
- You are answering and need a fact the owner told you previously.
- The owner corrects something you had stored before.