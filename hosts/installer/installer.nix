{ pkgs, lib, modulesPath, hostSystem, ... }:

let
  # Optional: bake a vendor cloud-key file into the image so a freshly
  # imaged unit comes up with cloud access already working:
  #
  #   AGENTIC_OS_BAKE_CLOUD_KEYS=/abs/path/cloud-keys.toml \
  #     nix build .#installer-iso --impure
  #
  # Understand the costs before using it: the key becomes part of the ISO
  # file AND of the build machine's world-readable Nix store, and every
  # unit imaged from that stick shares the same key. Acceptable for bench
  # provisioning and small batches; customer-scale production should
  # instead inject per-unit keys as a separate factory step after imaging.
  # A pure build (no env var, no --impure) produces a generic, secret-free
  # image -- that stays the default. The provisioned flavor gets a
  # distinct ISO name so the two can never be confused on a shelf.
  bakedKeyFile = builtins.getEnv "AGENTIC_OS_BAKE_CLOUD_KEYS";
  bakeKeys = bakedKeyFile != "";
in

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

  # Provisioned flavor: the key file rides at the ISO root (the live
  # system mounts the boot medium at /iso), outside the Nix store of the
  # installed system -- the install script places it with proper
  # ownership/mode where the UI shell's fallback lookup expects it.
  isoImage.contents = lib.optional bakeKeys {
    source = /. + bakedKeyFile;
    target = "/cloud-keys.toml";
  };
  image.fileName = lib.mkIf bakeKeys (lib.mkForce "agentic-os-provisioned-installer.iso");

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

      # Provisioned flavor only: place the vendor cloud-key file where the
      # UI shell's fallback lookup expects it, with the ownership/mode the
      # shell's permission check wants (root-owned, 0600).
      if [ -f /iso/cloud-keys.toml ]; then
        echo "Installing vendor-provisioned cloud keys."
        install -D -m 600 -o root -g root /iso/cloud-keys.toml /mnt/etc/agentic-os/cloud-keys.toml
      fi

      echo ""
      echo "=============================================================="
      echo "  Install complete. Powering off -- remove the USB stick."
      echo "=============================================================="
      sleep 5
      poweroff
    '';
  };
}
