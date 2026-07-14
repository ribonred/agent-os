{ pkgs, lib, modulesPath, hostSystem, ... }:

# Self-installing USB image: boot it on a factory-fresh unit and it wipes
# the internal disk, installs the full agentic-os system, and powers off.
# No keyboard, no network, no per-unit steps -- one image provisions any
# number of identical units.
#
# The complete host system closure is baked into the ISO
# (isoImage.storeContents), so the install is fully offline: it copies
# from the stick's own store instead of downloading anything.
#
# Build: nix build .#installer-iso
# Flash: dd (or Rufus in dd mode) to USB, boot the target with UEFI on.
#
# THIS IMAGE DESTROYS THE TARGET'S INTERNAL DISK ON BOOT (after a 15s
# countdown on the console). It is a provisioning tool, not a live/rescue
# system -- never hand it to a customer.

{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-minimal.nix" ];

  isoImage.storeContents = [ hostSystem ];

  systemd.services.agentic-install = {
    description = "One-shot agentic-os installer";
    wantedBy = [ "multi-user.target" ];
    after = [ "multi-user.target" ];
    path = with pkgs; [
      util-linux
      parted
      dosfstools
      e2fsprogs
      nixos-install-tools
      nix
      systemd
    ];
    serviceConfig = {
      Type = "oneshot";
      # Progress and the abort countdown must be visible on the machine's
      # own screen -- at the factory there is no journal reader attached.
      StandardOutput = "journal+console";
      StandardError = "journal+console";
    };
    script = ''
      set -euo pipefail

      # Target = the internal disk. NVMe is the expected case for the
      # mini-PC tier; SATA is the fallback. The boot USB itself shows up
      # as /dev/sd* too, but only after the NVMe check fails -- if a SATA
      # variant of the SKU ever exists, this needs to exclude removable
      # devices before shipping to a factory that images over USB.
      if [ -b /dev/nvme0n1 ]; then
        disk=/dev/nvme0n1 p1=/dev/nvme0n1p1 p2=/dev/nvme0n1p2
      elif [ -b /dev/sda ] && [ "$(cat /sys/block/sda/removable)" = "0" ]; then
        disk=/dev/sda p1=/dev/sda1 p2=/dev/sda2
      else
        echo "agentic-install: no internal disk found (no /dev/nvme0n1, no non-removable /dev/sda) -- nothing installed" >&2
        exit 1
      fi

      echo ""
      echo "=============================================================="
      echo "  agentic-os installer"
      echo "  ERASING $disk and installing in 15 seconds."
      echo "  POWER OFF NOW to abort."
      echo "=============================================================="
      echo ""
      sleep 15

      wipefs --all "$disk"
      parted --script "$disk" -- \
        mklabel gpt \
        mkpart ESP fat32 1MB 512MB \
        set 1 esp on \
        mkpart root ext4 512MB 100%
      udevadm settle

      # Labels here are the contract with
      # hosts/host/hardware-configuration.nix, which mounts by them.
      mkfs.fat -F 32 -n BOOT "$p1"
      mkfs.ext4 -F -L nixos "$p2"
      udevadm settle

      mount "$p2" /mnt
      mkdir -p /mnt/boot
      mount "$p1" /mnt/boot

      # The closure is already on the stick -- this copies it to the disk
      # and installs the bootloader, no network involved. --no-root-passwd
      # because user auth is declared in the system config itself.
      nixos-install --system ${hostSystem} --no-root-passwd

      echo ""
      echo "=============================================================="
      echo "  Install complete. Powering off -- remove the USB stick."
      echo "=============================================================="
      sleep 5
      poweroff
    '';
  };
}
