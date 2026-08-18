---
name: taskmd
description: >
  Track and update this repo's work with taskmd before assuming what is
  done. Use whenever starting a session, picking the next change, editing
  files under tasks/, creating or closing work, or answering "what's next"
  / "is this implemented" — even if the user did not say taskmd.
---

# Taskmd

This repo's source of truth for unfinished work is `taskmd`, not git
history and not a reconstructed reading of the tree.

## When to Use

- Session start, before changing product code
- "What's left", "is X done", "pick up the next task"
- Adding, editing, completing, or blocking work
- Anything that would otherwise invent project state

Don't use for: shipped product comments. Task IDs stay inside `tasks/`
and this skill. They must not appear in `agent-core/`, `ui/`, `build/`,
`brain/`, or `registry/`.

## Commands

Run from the repo root. If `taskmd` is missing, say so and read
`tasks/*.md` plus `.taskmd.yaml` instead of guessing.

```
taskmd list
taskmd next
taskmd graph --format ascii
taskmd validate
taskmd set <id> --status in-progress
taskmd set <id> --status completed
taskmd add "Task title"
```

Full field list: `tasks/TASKMD_SPEC.md`. Workflow: `tasks/CLAUDE.md`.
Phases must already exist in `.taskmd.yaml` before you assign one.

## Procedure

1. Run `taskmd list` and `taskmd next`.
2. If you take a task, `taskmd set <id> --status in-progress`.
3. Check off `- [ ]` lines in the task file as you go.
4. Before calling it done: acceptance criteria, then
   `taskmd verify <id>` if `verify:` exists, then
   `taskmd set <id> --status completed`, then `taskmd validate`.

## Gotchas

- `.gitignore` lists `tasks/`. The files can exist locally and still be
  absent from git. Local `tasks/` still wins over "I don't see it on
  GitHub."
- `.taskmd.yaml` phases still mention an older NixOS track. `CLAUDE.md`
  and `build/` are the current Ubuntu golden-image track. Do not revive
  NixOS because a phase name says so.
- A pending task with unchecked boxes is not implemented, even if nearby
  files look related.
- Do not close a task because the code "seems" to match. Close it when
  the acceptance criteria are true.
