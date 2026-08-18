---
name: golden-image
description: >
  Change the Ubuntu golden-image pipeline, packages, first-boot, Hermes
  install, or inference/services on the device. Use for build/,
  packages.txt, provision.sh, rootfs, flash, registry/*.yaml, or any
  "what ships on the unit" question — even if the user said OS, image,
  apt, or install.
---

# Golden image

This product is **not** a custom OS. It is a complete Ubuntu 26.04
Desktop image, copied byte-for-byte onto each unit. Nothing is resolved
on the customer's machine.

## When to Use

- Editing `build/`, `registry/`, or `Makefile` image targets
- Adding a system package, service, font, or browser
- Hermes / Ollama / llama.cpp / Postgres / Redis install
- First-boot identity, host keys, bearer token, disk grow
- "Does the device have X?" / "install Y on the unit"

## Two build paths

They share the same install scripts.

A — reference machine (faster to a first working unit):

```
sudo ./build/provision.sh --user <device-account>
# reboot, actually use it, then from a live USB:
./build/capture-image.sh
./build/flash.sh
```

B — container (reproducible factory path):

```
make ui-bundle
make rootfs-docker    # or: make rootfs
make image-docker     # or: make image
```

`make golden` / `make golden-docker` run UI + rootfs + image.

## Package entry

`build/packages.txt` is the only apt list. If it is not in that file, it
is not on the unit. Add a short comment justifying the size.

Not in the archive (installed by scripts, still part of the image):

- Hermes — `build/scripts/install-hermes.sh` (`HERMES_REF` is pinned)
- Ollama — `build/scripts/install-inference.sh`
- Chrome — `build/scripts/install-desktop.sh` (Google's apt repo)

Flags: `OLLAMA_SKIP=1`, `BROWSER_SKIP=1`, `HERMES_SSH_KEY=...`.

## First boot vs every boot

`build/rootfs-overlay/usr/local/sbin/agentic-firstboot`:

- Once: SSH host keys, bearer token, grow rootfs, write
  `/etc/agentic-os/hermes.env`
- Every boot: re-pin `SOUL.md` and shipped skills from
  `/usr/local/share/agentic-os/`. Runtime soul edits do not survive
  reboot. Name and persona live in the UI overlay, not in `SOUL.md`.

## Procedure

1. Decide whether the change belongs in the image at all. Extra packages
   cost every unit. Chrome is the only extra app; everything else is
   installed later by the agent on request.
2. Put apt packages in `packages.txt`. Put install logic in the matching
   `build/scripts/install-*.sh`.
3. Put agent-facing service facts in `registry/*.yaml`, not in the build
   scripts. The running agent has no build tooling.
4. Do not download models at first networked boot. `registry/ollama.yaml`
   and `registry/llama-cpp.yaml` are `installed_not_enabled` until a
   model exists and the owner consents.
5. After a Hermes identity or skill change, confirm both copies:
   `$HERMES_HOME/` (runtime) and `/usr/local/share/agentic-os/` (re-pin
   source). An earlier bug wrote identity under a path Hermes never
   reads.

## Gotchas

- The agent runs **as the device owner**, not a `hermes` service user.
  Passwordless sudo is intentional. The approval gate is inside the
  agent, not a sudo prompt.
- Postgres is unix-socket only (`listen_addresses = ''`). Peer auth:
  the role name must match the owner account. Role creation happens on
  the device (`agentic-pg-init`), not in the chroot.
- `apt` has no rollback. A ruined unit is reflashed. Do not invent an
  atomic package manager in comments or code.
- `ui-bundle` is built with `env -i` and the system rustup/bun, then
  copied in. Do not build the UI inside Nix or rewrite RPATH.
- A container build cannot prove the image boots. Only hardware can.
