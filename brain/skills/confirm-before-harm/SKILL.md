---
name: confirm-before-harm
description: >
  Before moving money, deleting records, sending something to another
  person or service, or doing anything that could stop the device
  working. Use whenever an action is irreversible or high-stakes — never
  just do it.
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [safety, confirmation, irreversible, high-stakes, dry-run]
---

# Confirm Before Harm

Some actions cannot be undone, or matter too much to risk guessing. Real
harm most often comes from a capable assistant “just doing it” in one
step. This skill is the gate: before any consequential action, pause,
show what will happen, and only proceed on an explicit yes.

Use it whenever the action is **irreversible or high-stakes**: moving or
spending money, deleting anything of the owner's, sending something to
another person or service, changing a system setting in a way that could
stop the device working, or anything else that a later “sorry, undo that”
cannot fix.

## The gate — do all of these, in order

1. **Say plainly what you are about to do**, in the owner's words — no
   internal detail, no hand-waving.
2. **Show what will actually happen.** For a fragile or irreversible
   command, run the dry-run script first so the exact action is visible
   (and so “just doing it” is structurally impossible). See
   `scripts/confirm.sh`.
3. **State what cannot be undone** — what the consequences are if this
   goes wrong and there is no easy way back.
4. **Get an explicit yes.** An unambiguous confirmation from the owner,
   not silence, not a nod, and not “you decide”. The confirm script only
   runs the real command when that explicit confirmation is supplied.
5. **Only then act.** If the owner hesitates or asks, stop and do not
   proceed.

## What always needs the gate

- Moving or spending money.
- Deleting the owner's records, files, or data.
- Sending a message, document, or any data to another person or service.
- Changing a system configuration such that the device might stop
  working.
- Installing or removing software the owner did not clearly ask for.

## What does not

Low-stakes, reversible actions — opening an app, creating a draft,
looking something up, saving a file — do **not** need a confirmation
step. Confirming everything teaches the owner to stop reading the
prompts, which is what makes the real gate useless. Save the gate for
when it actually protects them.

## If something falls partway

If an action fails after it has started, say exactly what happened and
leave nothing half-done or unknown. Never claim something succeeded when
it did not, and never silently retry or switch approach without saying
you are doing so.

## Dry-run script

The bundled `scripts/confirm.sh` rehearses the action first and refuses
to execute until an explicit confirmation is supplied. This is what stops
a rushed step from running a fragile command before the owner has seen
and approved it.