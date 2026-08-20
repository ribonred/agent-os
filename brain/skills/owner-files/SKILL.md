---
name: owner-files
description: >
  Create, move, find, or attach the owner's files — anything in their
  Documents, Downloads, or the in-app file shelf. Use when the owner
  asks to save, locate, rename, move, or share a file, or to hand a file
  to a conversation.
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [files, documents, shelf, attach]
---

# Owner Files

The owner's files live where the owner can see them. This skill is how
the assistant creates, moves, finds, and attaches files without letting
them pile up where they become invisible or confusing.

Use it whenever the owner asks to **save, find, rename, move, open, or
share a file**, or to hand a document into the current conversation.

## Where files belong

- **Documents** — the owner's documents and anything they created.
- **Downloads** — files pulled in from elsewhere.
- **The file shelf** — files the owner wants close at hand for a
  conversation.

When you create a file for the owner, put it in one of these places the
owner can actually see, not somewhere hidden.

## Rules

- **Do not scatter files across the home folder.** The file view shows
  that clutter — a file dropped in a random location becomes noise the
  owner has to wade through. Put files where they belong.
- **Do not save assistant state here.** The assistant's own working
  notes, scratch files, and internal state do not belong among the
  owner's documents. Keep that separate so the owner's file view only
  ever shows the owner's things.

- **Scripts and tooling go in `~/.agentic-os/tools/`, always.** Anything
  written to be run rather than read -- a script to pull figures out of a
  spreadsheet, a one-off helper, anything that ends in `.py` or `.sh` --
  belongs there and nowhere else. Create the folder if it is not there.

  Two reasons, and both matter. The owner's file view never shows it, so
  a folder of machinery cannot pile up in front of someone who would not
  know what any of it was for or whether it was safe to delete. And
  keeping every script in one known place means a script can be found
  and reused next month instead of being written again.

  This is not a suggestion about tidiness. A `.py` file sitting in
  Documents is a file the owner will eventually open, fail to
  understand, and worry about.
- **Attach, don't bury.** When the owner wants a file in the
  conversation, attach it cleanly; when they want it saved, save it
  somewhere sensible and tell them where in plain words.

## When to use

- The owner wants a file created (a letter, a list, a document).
- The owner asks “where is my file?” or “find the receipt”.
- The owner wants something renamed or moved into a sensible place.
- The owner wants a file handed into the chat so the assistant can read it.