---
name: commercial-hygiene
description: >
  Keep shipped files free of personal names, internal process leaks, and
  Ubuntu lectures. Use before committing, when writing comments, UI copy,
  logs the owner might see, system account names, or any text that could
  land on a customer's device.
---

# Commercial hygiene

This is product code that ships to customers. Every file is something a
device could expose or a future teammate could read with no shared
context.

## When to Use

- Any edit under `agent-core/`, `ui/`, `build/`, `brain/`, `registry/`,
  `design/`
- Comments, commit messages that might be copied into comments, UI
  strings, systemd units, Hermes environment hints
- Naming users, hosts, paths, or services

Don't use this skill as a place to store task status. Use `taskmd`.

## Rules

1. **No personal names, usernames, emails, or "my machine".** System
   accounts are generic (`admin`, or the build-time device owner), never
   a developer's handle.
2. **No internal process by name in shipped comments.** Do not cite
   taskmd IDs, skill names, or private doc names as the reason for a
   rule. If the rule matters, restate the reason in plain terms.
3. **Explain why.** "unix socket only -- nothing on this box needs
   network pg yet" is correct. "see task 027" is not.
4. **Task IDs are fine only inside `tasks/` and `.claude/`.**
5. **The owner never has to understand Ubuntu.** Owner-facing strings
   (UI, agent replies, `agentErrors.ts`, constitution overlay) describe
   the device and the task. Do not volunteer Linux, apt, systemd, file
   paths, model names, or error codes. If the owner asks, answer plainly.
6. **No vertical knowledge in the image.** Clinic / POS / skincare /
   finance behavior is learned after purchase. Do not bake it into
   prompts, schemas, or packages.
7. **Do not present the assistant as a licensed professional.**

## Check before you finish an edit

- [ ] No person or handle in the diff
- [ ] No `task 0xx`, ticket id, or private ritual name in shipped
      comments
- [ ] New owner-visible text would make sense to someone who has only
      used a phone
- [ ] New comments still make sense after `tasks/` is gone
