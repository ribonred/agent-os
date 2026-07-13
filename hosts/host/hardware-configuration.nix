{ lib, ... }:

# ==========================================================================
# PLACEHOLDER -- NOT REAL HARDWARE DATA.
#
# This file exists only so `nix flake check` / `nix build` can evaluate
# without a physical disk attached. Every value below is fabricated.
#
#   1. Boot the HOST from a NixOS installer USB.
#   2. Run `nixos-generate-config --root /mnt` (after partitioning/mounting).
#   3. Replace this entire file with the generated one (real UUIDs, real
#      fileSystems, real boot.loader settings).
#
# Do not hand-edit fake UUIDs to "look real" -- that hides the fact that
# this hasn't been validated against the hardware.
# ==========================================================================

{
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  boot.initrd.availableKernelModules = [ "xhci_pci" "nvme" "usb_storage" "sd_mod" ];
  boot.kernelModules = [ ];

  fileSystems."/" = {
    device = "/dev/disk/by-label/PLACEHOLDER-ROOT";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-label/PLACEHOLDER-BOOT";
    fsType = "vfat";
  };

  swapDevices = [ ];

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.intel.updateMicrocode = lib.mkDefault true;
}
