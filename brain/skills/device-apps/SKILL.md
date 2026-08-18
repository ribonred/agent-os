---
name: device-apps
description: >
  Open, show, or print something on the screen, or when the owner says
  “put that on the screen”, “open the browser”, “show me this file”, or
  “print this”. Use whenever the owner wants an application or a window
  brought up or set in front of them.
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [desktop, apps, browser, print, window]
---

# Device Apps

The assistant is what the owner sees first and returns to, but it is not
the whole computer. Real applications open alongside it when the owner
asks. This skill is how the assistant drives that desktop the way the
owner intends — without ever making them manage the machine underneath.

Use it when the owner asks for something to be **opened, shown, printed,
or put on the screen**, even if they put it in their own words (“get me
to my bank”, “pull up that file”, “show the receipt”).

## When to use

- The owner wants an application opened (a browser, an editor, anything).
- The owner wants a specific file or document shown.
- The owner wants something printed or viewed on screen.
- The owner asks “what’s open?” or wants a window brought to the front.

## How to do it

Speak to the owner in what they asked for, never in internal names. Do
not say the names of the desktop, the window tool, or the package
manager — the owner asked “open the browser”, not for a lesson in how
the desktop works. Internal tool names below are for the assistant's own
use; they never appear in what the assistant tells the owner.

The working desktop keeps these exact moves:

- **Launch** an application: `google-chrome-stable &` (the browser),
  `gio open <file>` (open any file with the application that fits it).
- **See what is open**: `wmctrl -l` — use this whenever you need to know
  what is already on screen or where to return focus.
- **Return the owner to the assistant** when the task is done: bring the
  assistant window back to the front instead of leaving the owner
  somewhere they now have to escape.

So a task is: launch or open what was asked, check what is on screen,
complete the task, then bring the assistant back to the front. The owner
has one obvious home and it is the assistant.

## Installing software

The image ships complete; ordinary use needs no installs. When the owner
*asks* for a new application, install it on request — but:

- **Confirm before installing.** Installing is a deliberate act, not a
  background chore. State what you are about to install and get a yes.
- **Do not bake extra packages into the image.** Each package baked in
  costs size on every shipped unit. The default is to install on the
  owner's request, not to make the image bigger.

There is no rollback after install. A device that breaks badly enough is
rebuilt from the image it shipped with — that is the supported recovery
path. So a deliberate, confirmed install is how mistakes are avoided in
the first place.

## Boundaries

- Never expose the underlying desktop or package tools by name.
- Never install anything silently or without confirmation.
- Always leave the owner back at the assistant when the task is done.
