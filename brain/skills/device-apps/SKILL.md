---
name: device-apps
description: >
  Open, show, or print something on the screen, look something up on the
  web, or use a website on the owner's behalf — “put that on the
  screen”, “open the browser”, “show me this file”, “print this”, “find
  me an article about…”, “go to my bank”. Use whenever the owner wants
  an application, a page, or a window brought up or set in front of
  them, and whenever a task means using the web.
version: 1.1.0
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

- **Open** a file, a folder or a web address: `gio open <file-or-url>`.
  This puts it in front of the owner in the application that fits it,
  the same one they would have got by opening it themselves.
- **Browse** — search, read a page, fill something in, click through a
  site — with the browsing tools, directly. Do not open a browser
  first, and do not script one: the browser is already there and
  already under your control (see below).
- **Return the owner to the assistant** when the task is done: bring the
  assistant window back to the front instead of leaving the owner
  somewhere they now have to escape.

So a task is: open or browse what was asked, complete it, then bring the
owner back to the assistant. The owner has one obvious home and it is
the assistant.

## The browser is the owner's browser

There is one browser on this device and it is theirs — their sign-ins,
their bookmarks, their tabs. Your browsing tools act inside that same
browser: what you do happens in the window in front of them, and they
can watch it happen and take over at any point.

Two consequences, and they matter:

- **You are acting in their session, signed in as them.** Everything the
  rule about confirming before harm covers applies at least as strongly
  here. Reading a page needs nothing. Sending, buying, posting,
  deleting, or anything that spends money or speaks in their name is
  confirmed with them first, in plain words, before you click it.
- **Never open a second browser** — not by launching one, not by
  writing a script that drives one. A browser the owner cannot see is a
  browser they cannot trust or interrupt, and a second one would be
  signed in to nothing of theirs.

If the browser will not come up, say that the browser would not open and
leave it there. The reason is never the owner's problem to solve.

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
- Never explain how the browser is driven — no control channels, no
  ports, no profiles, no sessions. To the owner it is simply their
  browser, and you can use it.
- Always leave the owner back at the assistant when the task is done.
