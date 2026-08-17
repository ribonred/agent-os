# The desktop underneath

How the device moved from a single-app kiosk to a real desktop with the
assistant in charge of it, and what that constrains.

## Why

The device booted into a compositor that runs **exactly one** program.
That was the right shape for an appliance: no desktop, no terminal,
nothing launchable but the assistant.

It stopped being right when the product needed a browser. A
single-application compositor cannot host a second window, so a browser
would have had to live *inside* the Tauri shell -- embedding a browser
engine, driving it over a debugging protocol, and hand-managing window
geometry, tab state and popups. That is rebuilding a desktop, badly,
inside an application.

The alternative is to take a desktop that already exists and make the
agent its operator. **Ubuntu's own desktop session** runs underneath: the
assistant autostarts and is what the owner sees, and real applications
open alongside it when the agent or the owner opens them.

This is cheap in a way the embedded-browser path was not, because every
capability it needs is already a shell command, and the agent already has
a shell and root:

```
google-chrome-stable &                 launch
wmctrl -l                              what is open
gio open <file>                        open with the right application
```

No new protocol, no automation framework, no second rendering engine.

## Why Ubuntu's desktop rather than one we assemble

An earlier iteration built the desktop out of parts -- a compositor plus
a separately chosen bar, launcher, notification daemon and wallpaper
tool, each themed by hand. That is defensible on a distribution with no
desktop of its own. On Ubuntu it is redundant: the session, its
authentication agent, portals, audio, network applet and font stack are
already integrated, tested together, and maintained.

The parts approach also carried a cost that only shows up later. Every
component is one more thing to keep working across an upgrade, and none
of it is what makes the product valuable. The assistant is.

So the desktop is Ubuntu's, unmodified. What the product controls is
what the owner *sees by default*: the session autologins, the assistant
starts with it and fills the screen. The desktop is underneath, reachable,
and not the thing being sold.

## What this changes about the product

The assistant is no longer the whole computer. It is the assistant layer
over a real computer, and it is what the owner sees first and returns to.

That is a repositioning, not just a feature, and it was a deliberate
choice rather than a drift. The appliance qualities that matter -- boots
to the assistant, no visible setup, nothing a non-technical owner has to
understand -- are preserved by *default behaviour* rather than by making
anything else impossible.

Nothing built for the previous shape is discarded. The conversation pane,
the file view, the selection-to-context flow and the error translation
all keep their jobs. The file view becomes the agent-aware view of files
rather than the only way to reach them.

## Rules

- **The assistant is what the session starts with.** Whatever else is
  open, there is one obvious place the owner returns to.
- **The desktop is reachable but quiet.** It is not hidden, and nothing
  is disabled to stop the owner reaching it -- but nothing advertises it
  either.
- **The agent returns the owner to the assistant** when a task is done,
  rather than leaving them in an application they now have to escape.

## Who the device runs as

Everything runs as the **device owner's account**. The agent is not a
separate service user.

That was not the original design, and the reason for changing it is worth
recording. A separate account meant every file the agent created needed
group-writability to stay openable by the owner, the agent's own state
lived somewhere the owner's file view could not show, and
session-scoped resources -- the GPU, the keyring, the display -- were
granted to the owner's login session and invisible to a system service.
All of that machinery existed to bridge a division that bought nothing:
this is a single-user appliance, and every file on it is the owner's.

The owner has passwordless `sudo` and a permissive polkit rule. They are
a non-technical daily user of an appliance, not an administrator: a
password prompt they cannot answer is a dead end, not a security
boundary. The device administers itself on their behalf, and the approval
gate inside the agent -- not a `sudo` prompt -- is where "confirm before
harm" lives.

## Installing software

The image ships **complete**. Everything the device needs at the system
level -- services, drivers, the desktop, the agent runtime -- is present
before the unit is powered on, because a golden image is copied
byte-for-byte and nothing is resolved on the customer's machine.

After that, the agent installs applications on request with `apt`. It has
root, so this genuinely works, including things a per-user package
manager could not do.

The tradeoff, stated plainly: **`apt` has no rollback.** A failed or
badly-chosen install cannot be undone by a single command the way an
atomic package manager allows. Two things reduce the risk rather than
eliminate it:

- The base image is complete, so ordinary use requires no installs at
  all. Installing is an occasional, deliberate act.
- A device that breaks badly enough is reflashed from the same image it
  shipped with, which is a supported recovery path rather than a
  disaster.

If rollback becomes a requirement, the answer is an A/B image scheme, not
a different package manager.

## The browser

`google-chrome-stable`, from Google's own apt repository rather than the
archive. Ubuntu's `firefox` and `chromium-browser` packages are
transitional shims that install snaps, and a snap wants a running
`snapd`, updates itself over the network, and is slow on first launch --
none of which suits a device that may never see a network.

Because this is a product that ships to customers, the redistribution
terms are worth confirming with whoever handles that before units go out
-- a business question, not a technical blocker. The build supports
`BROWSER_SKIP=1` so an image can be produced without it while that is
settled.

Chrome is the only application in the base image beyond what the system
itself needs. Everything else is the owner's decision, installed through
the agent on request. Each package baked in costs size in an image copied
onto every unit, so the default is to not bake it in.
