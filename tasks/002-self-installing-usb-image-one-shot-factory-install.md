---
id: "002"
title: "Generate and commit hardware-configuration.nix from physical NUC"
status: in-progress
priority: high
effort: small
phase: bootstrap
dependencies: ["001"]
tags: ["nix", "nixos"]
created_at: 2026-07-13
---

# Generate and commit hardware-configuration.nix from physical NUC

## Objective

Replace the placeholder `hosts/host/hardware-configuration.nix` (fabricated
UUIDs, exists only so the flake evaluates) with the real file generated on
the physical SWNUC11PAHi3000. Until this lands, `nixosConfigurations.host`
builds but cannot boot real hardware, which also blocks task 018's final
acceptance (patched Tauri binary launching on the device).

This task requires hands on the physical machine -- it cannot be done from
the dev box. Everything below is the bench runbook.

## Tasks

- [ ] Flash a NixOS 26.05 minimal ISO (x86_64) to USB. From Windows use
      Rufus in **dd mode** (WSL cannot write raw USB block devices);
      from Linux: `sudo dd if=nixos-minimal-*.iso of=/dev/sdX bs=4M
      status=progress conv=fsync`.
- [ ] Boot the NUC from the stick: F10 at power-on for the boot menu,
      UEFI boot enabled in BIOS (F2).
- [ ] Partition the internal disk (likely `/dev/nvme0n1`) -- UEFI layout:

      ```
      sudo parted /dev/nvme0n1 -- mklabel gpt
      sudo parted /dev/nvme0n1 -- mkpart ESP fat32 1MB 512MB
      sudo parted /dev/nvme0n1 -- set 1 esp on
      sudo parted /dev/nvme0n1 -- mkpart root ext4 512MB 100%
      sudo mkfs.fat -F 32 -n BOOT /dev/nvme0n1p1
      sudo mkfs.ext4 -L nixos /dev/nvme0n1p2
      sudo mount /dev/disk/by-label/nixos /mnt
      sudo mkdir -p /mnt/boot
      sudo mount /dev/disk/by-label/BOOT /mnt/boot
      ```

      This wipes the disk -- confirm nothing on the NUC needs saving first.
- [ ] `sudo nixos-generate-config --root /mnt`
- [ ] Get `/mnt/etc/nixos/hardware-configuration.nix` back into this repo
      replacing `hosts/host/hardware-configuration.nix` wholesale (scp it
      to the dev box, or clone the repo on the installer and edit there).
      Do not merge with the placeholder -- replace the entire file; the
      placeholder's header says exactly this.
- [ ] Install from the flake (needs network on the installer):

      ```
      nix-shell -p git
      git clone <repo> && cd agentic-os
      sudo nixos-install --flake .#host
      ```
- [ ] Reboot without the stick; log in as `admin` / `changeme` (and note
      the standing warning in configuration.nix: decide real auth before
      this box leaves the dev bench).
- [ ] Commit the real hardware-configuration.nix and confirm
      `nix build .#nixosConfigurations.host.config.system.build.toplevel`
      still succeeds from the repo with the real file in place.

## Acceptance Criteria

- [ ] `hosts/host/hardware-configuration.nix` contains real
      `nixos-generate-config` output from the physical NUC: real
      filesystem UUIDs/labels, real `boot.initrd.availableKernelModules`
      -- no PLACEHOLDER strings remain anywhere in the file.
- [ ] The NUC boots the installed system from its internal disk with no
      installer USB attached.
- [ ] The flake's toplevel closure still builds with the real file.
