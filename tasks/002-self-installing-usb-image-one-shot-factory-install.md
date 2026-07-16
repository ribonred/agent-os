---
id: "002"
title: "Self-installing USB image: one-shot factory install, no per-unit config"
status: in-progress
priority: high
effort: small
phase: bootstrap
dependencies: ["001"]
tags: ["nix", "nixos", "installer"]
created_at: 2026-07-13
---

# Self-installing USB image: one-shot factory install, no per-unit config

## Objective

Originally "generate hardware-configuration.nix on the physical NUC" --
rescoped on a product decision: per-unit `nixos-generate-config` is a dev
workflow that cannot scale to shipping many identical units. Instead the
install must be one-shot and unit-agnostic: boot a USB stick, walk away,
the device is fully provisioned.

Design that makes this possible: the installer creates the same GPT
layout with fixed filesystem labels (`BOOT`, `nixos`) on every unit, and
`hosts/host/hardware-configuration.nix` mounts by those labels -- a
device-class profile with zero per-machine data (no UUIDs). The full host
system closure is baked into the ISO, so the install is offline and
byte-identical across units.

## Tasks

- [x] Rewrite `hosts/host/hardware-configuration.nix` as a device-class
      profile: label-based mounts, broad NUC-class initrd module set
      (NVMe + SATA + USB + Thunderbolt), redistributable firmware for
      wifi/bluetooth. Header comment forbids replacing it with raw
      per-unit `nixos-generate-config` output.
- [x] `hosts/installer/installer.nix`: auto-install systemd service on
      the minimal installer ISO -- picks the internal disk (NVMe first,
      non-removable SATA fallback, loud failure if neither), 15-second
      abort countdown on the console, wipe + partition + label, install
      from the baked-in closure with `nixos-install --system`, power off.
- [x] Flake: `nixosConfigurations.installer` (host toplevel passed in via
      specialArgs) and `packages.x86_64-linux.installer-iso`.
      Build: `nix build .#installer-iso`.
- [x] End-to-end VM validation, twice on independent hypervisors:
      QEMU/OVMF (blank NVMe -> unattended install -> self-poweroff ->
      booted from disk; SSH up, hostname agentic-os, postgresql/redis/
      ollama all active) and VMware (live run: installed, powered off,
      booted to login). Both proved the image is UEFI-only by
      construction -- VMware's default BIOS firmware + LSI SCSI disk
      reproduces "Operating System Not Found"; the fix is
      firmware = "efi" and an NVMe virtual disk. Real NUCs boot UEFI by
      default, but any bench VM must match.
- [ ] Flash the ISO (Rufus dd mode from Windows / `dd` from Linux) and
      boot the physical NUC with it. Verify: installs unattended, powers
      off, then boots from internal disk with no USB attached.

## Acceptance Criteria

- [x] No per-unit configuration anywhere: the same ISO provisions any
      unit of the SKU; hardware-configuration.nix contains no UUIDs or
      other machine-specific values.
- [x] `nix build .#installer-iso` succeeds; install path is fully offline
      (closure in `isoImage.storeContents`).
- [ ] Physical NUC provisioned by the stick boots to the installed system
      -- hands-on validation at the bench.

## Notes

- The image is a provisioning tool that destroys the target disk on boot;
  it must never be handed to a customer.
- Two build flavors. The pure build is generic and secret-free (safe
  default). An opt-in provisioned flavor bakes a vendor cloud-key file
  into the image (`AGENTIC_OS_BAKE_CLOUD_KEYS=/path nix build
  .#installer-iso --impure`); the installer places it at
  `/etc/agentic-os/cloud-keys.toml` (root, 0600) where the UI shell's
  fallback lookup expects it. The provisioned ISO gets a distinct file
  name so the flavors can't be confused. Costs are documented in
  installer.nix: the key lives in the ISO and the build machine's Nix
  store, and all units imaged from one stick share it -- a bench/small-
  batch tool, not the customer-scale answer (that's per-unit keys as a
  separate factory step after imaging).
- The standing auth warning in configuration.nix still applies: admin /
  changeme is a bench credential, not shippable.
