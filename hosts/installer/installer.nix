{ pkgs, lib, modulesPath, hostSystem, ... }:

let
  # Optional: bake a vendor cloud key so an imaged unit has cloud access
  # immediately.
  #
  #   AGENTIC_OS_BAKE_CLOUD_KEYS=/abs/path/cloud-keys.toml \
  #     nix build .#installer-iso --impure
  #
  # Costs: the key lands in the ISO and in the build machine's
  # world-readable store, and every unit from that stick shares it. Fine
  # for bench and small batches; production should inject per-unit keys
  # as a factory step. A pure build produces a secret-free image and
  # stays the default.
  bakedKeyFile = builtins.getEnv "AGENTIC_OS_BAKE_CLOUD_KEYS";
  bakeKeys = bakedKeyFile != "";
in

# Self-installing USB image: boots, wipes the internal disk, installs
# agentic-os, powers off. No keyboard, no network, no per-unit steps.
# The whole host closure is baked in (isoImage.storeContents), so the
# install is fully offline.
#
# Build: nix build .#installer-iso
# Flash: dd (or Rufus in dd mode) to USB, boot the target with UEFI on.
#
# THIS DESTROYS THE TARGET'S INTERNAL DISK ON BOOT after a 15s console
# countdown. A provisioning tool, not a rescue system -- never hand it
# to a customer.

{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-minimal.nix" ];

  isoImage.storeContents = [ hostSystem ];

  # Rides at the ISO root (mounted at /iso), outside the installed
  # system's store; the script below places it with the right mode.
  isoImage.contents = lib.optional bakeKeys {
    source = /. + bakedKeyFile;
    target = "/cloud-keys.toml";
  };
  # baseName, not fileName: the iso builder never reads fileName --
  # the override eval'd fine and the artifact ignored it.
  image.baseName = lib.mkIf bakeKeys (lib.mkForce "agentic-os-provisioned-installer");

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
      # Visible on the machine's own screen -- no journal reader at the
      # factory.
      StandardOutput = "journal+console";
      StandardError = "journal+console";
    };
    script = ''
      set -euo pipefail

      # The disk is already wiped by the time anything fails, so a
      # silent exit leaves a machine that looks installed until its
      # first boot finds no bootloader.
      trap 'echo ""; echo "=============================================================="; echo "  INSTALL FAILED (line $LINENO). This machine has NO bootloader."; echo "  Check: journalctl -u agentic-install"; echo "  Fix the cause and boot this installer again."; echo "=============================================================="' ERR

      # NVMe expected, SATA fallback. The boot USB is /dev/sd* too, so
      # the removable check matters if a SATA SKU ever exists.
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

      # settle alone isn't enough on slow NVMe emulation (VMware): nodes
      # lag parted, and mkfs on a missing node aborts a wiped disk.
      for _ in $(seq 30); do
        [ -b "$p1" ] && [ -b "$p2" ] && break
        sleep 1
        partprobe "$disk" 2>/dev/null || true
      done
      [ -b "$p1" ] && [ -b "$p2" ] || { echo "partition nodes $p1/$p2 never appeared" >&2; exit 1; }

      # Labels here are the contract with
      # hosts/host/hardware-configuration.nix, which mounts by them.
      mkfs.fat -F 32 -n BOOT "$p1"
      mkfs.ext4 -F -L nixos "$p2"
      udevadm settle

      mount "$p2" /mnt
      mkdir -p /mnt/boot
      mount "$p1" /mnt/boot

      # Copies from the stick, no network. --no-root-passwd because user
      # auth is declared in the system config.
      echo ""
      echo "Copying the system to disk. This takes several minutes and"
      echo "prints little progress -- it is NOT stuck. Do not power off."
      echo ""
      nixos-install --system ${hostSystem} --no-root-passwd
      echo "System copy + bootloader done."

      # The cloud key is optional; the bearer token is not. Written
      # together once, which meant non-provisioned images shipped no
      # hermes.env and the agent could not authenticate against its own
      # gateway. An image with no cloud key is a normal image.
      or_key=""
      if [ -f /iso/cloud-keys.toml ]; then
        echo "Installing vendor-provisioned cloud keys."
        install -D -m 600 -o root -g root /iso/cloud-keys.toml /mnt/etc/agentic-os/cloud-keys.toml

        or_key="$(sed -n 's/^api_key *= *"\(.*\)"/\1/p' /iso/cloud-keys.toml | head -n1)"
        if [ -z "$or_key" ]; then
          echo "WARNING: could not parse api_key from cloud-keys.toml; hermes.env gets no cloud key" >&2
        fi
      else
        echo "No vendor cloud key baked in; the owner supplies one through the UI."
      fi

      # Per-unit token, always. OPENROUTER_API_KEY stays empty when
      # nothing was baked -- the owner's keyring key takes precedence.
      install -D -m 600 -o root -g root /dev/null /mnt/etc/agentic-os/hermes.env
      {
        echo "OPENROUTER_API_KEY=$or_key"
        echo "API_SERVER_KEY=$(head -c 32 /dev/urandom | sha256sum | cut -c1-48)"
      } > /mnt/etc/agentic-os/hermes.env

      echo ""
      echo "=============================================================="
      echo "  Install complete. Powering off -- remove the USB stick."
      echo "=============================================================="
      sleep 5
      poweroff
    '';
  };
}
