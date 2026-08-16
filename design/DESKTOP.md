# The desktop underneath

How the device moved from a single-app kiosk to a real desktop with the
assistant in charge of it, and what that constrains.

## Why

The device booted into `services.cage`, a Wayland compositor that runs
**exactly one** program. That was the right shape for an appliance: no
desktop, no terminal, nothing launchable but the assistant.

It stopped being right when the product needed a browser. cage cannot
host a second window, so a browser would have had to live *inside* the
Tauri shell -- embedding a browser engine, driving it over a debugging
protocol, and hand-managing window geometry, tab state and popups. That
is rebuilding a desktop, badly, inside an application.

The alternative is to take a desktop that already exists and make the
agent its operator. **Hyprland** runs underneath. The assistant shell
autostarts fullscreen on workspace 1 and stays running. Real applications
open fullscreen on their own workspaces when the agent or the owner opens
them.

This is cheap in a way the embedded-browser path was not, because every
capability it needs is already a shell command, and the agent already has
a shell and root:

```
hyprctl dispatch exec google-chrome-stable    launch
hyprctl clients                               what is open
hyprctl dispatch workspace 2                  switch
hyprctl dispatch focuswindow address:0x...    bring forward
```

No new protocol, no automation framework, no second rendering engine.

## What this changes about the product

The assistant is no longer the whole computer. It is the assistant layer
over a real computer, and it is what the owner sees first and returns to.

That is a repositioning, not just a feature, and it was a deliberate
choice rather than a drift. The appliance qualities that matter -- boots
to the orb, no visible OS, nothing a non-technical owner has to
understand -- are preserved by *default behaviour* rather than by making
anything else impossible.

Nothing built for the previous shape is discarded. The conversation pane,
the file view, the selection-to-context flow and the error translation
all keep their jobs. The file view becomes the agent-aware view of files
rather than the only way to reach them.

## Why the desktop is ours and not an imported one

Hyprland on its own is a compositor, not a desktop: no bar, no launcher,
no notifications, no wallpaper. A device with only the assistant running
on it has no way to open anything, which is not "an appliance" -- it is
being stuck.

The obvious shortcut is to import one of the well-made personal Hyprland
configurations and ride it. That was tried and rejected on inspection,
for reasons worth recording so it is not re-proposed:

- They export no reusable modules. Their flake outputs are their own
  `nixosConfigurations`, so there is nothing to import; taking their
  desktop means reaching into the input's store path and matching
  whatever `specialArgs` they happen to take.
- They track `nixos-unstable` and Hyprland's git master. This device is
  pinned to a release, so importing one puts a second nixpkgs in a
  closure that already carries the entire system for offline install.
- Their theme would become this product's look everywhere except the
  assistant's own workspace. `design/DESIGN.md` governs what the owner
  sees; a device that switches from a designed surface to somebody
  else's defaults reads as two products stitched together.

The parts themselves -- waybar, fuzzel, dunst, hyprpaper -- are ordinary
packages. `modules/desktop.nix` takes those and dresses them in the
product's own tokens, which costs about as much work as adapting an
import would have and leaves nothing external in a shipped image.

## Rules

- **The assistant is always workspace 1.** Whatever else is open, there
  is one obvious place the owner returns to.
- **Applications open fullscreen on their own workspace**, not as
  floating windows over the assistant. Hyprland's native model, so it is
  a dispatch rather than geometry management, and it keeps one thing on
  screen at a time -- which is what a counter device wants.
- **The desktop is reachable but quiet.** Workspace switching and closing
  a window work. Bindings that are destructive, or that would strand
  someone who pressed them by accident, are not bound at all.
- **The agent returns the owner to workspace 1** when a task is done,
  rather than leaving them in an application they now have to escape.

## Installing software

**Shipped devices use `nix profile` only. Never `nixos-rebuild`.**

`nix profile` is per-user and atomic, and `nix profile rollback` undoes
any change. A failed install leaves the running system untouched. A
rebuild, by contrast, can break the graphical session -- and an owner
looking at a black screen has no way to ask the assistant for help. That
failure mode is unacceptable on a device someone bought, so the path that
can cause it is closed.

Rebuilds remain the developer path, on developer machines.

The cost of that decision is a hard boundary, and it is the single most
important consequence in this document:

> `nix profile` installs **user applications**. It cannot enable system
> services, install drivers, or change anything in `/etc`.

So the shipped image must be **complete up front**. The agent can install
a photo editor later; it cannot make a scanner work, bring up a network
share, or enable a daemon that was not built in. Anything the device may
need at the system level ships at build time.

For work the agent needs running in the background afterwards, **user
systemd units** (`~/.config/systemd/user/`) are the answer -- no root, no
rebuild, and removable the same way they were added.

## The browser

`google-chrome`, which is unfree and therefore requires `allowUnfree`.
Chromium is the same engine and carries no licensing question; Chrome was
chosen deliberately anyway. Because this is a product that ships to
customers, the redistribution terms are worth confirming with whoever
handles that before units go out -- a business question, not a technical
blocker.

Chrome is the only application in the base image beyond what the system
itself needs. Everything else is the owner's decision, installed through
the agent on request. Each package baked into the image costs size in an
ISO that carries the entire system closure for offline installation, so
the default is to not bake it in.
