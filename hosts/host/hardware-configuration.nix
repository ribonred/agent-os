{ lib, ... }:

# ==========================================================================
# Device-class hardware profile -- deliberately NOT per-unit.
#
# The product installs onto many identical units from one self-installing
# USB image, so nothing in this file may be unit-specific. The installer
# (hosts/installer/) formats every device with the same GPT layout and
# filesystem labels, and this file mounts by those labels -- there are no
# per-machine UUIDs to regenerate. Do not replace this with raw
# `nixos-generate-config` output from a bench unit: that would pin one
# machine's UUIDs and break the image-installs-anywhere model.
#
# Labels are the contract between installer and system:
#   BOOT  -> FAT32 ESP, mounted at /boot
#   nixos -> ext4 root
# Change them in both places or not at all.
# ==========================================================================

{
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # Broad module set for Intel NUC-class mini-PCs: NVMe and SATA storage,
  # USB boot/input, Thunderbolt. Wider than any single unit needs, so the
  # same image boots across storage variants of the same SKU.
  boot.initrd.availableKernelModules = [ "xhci_pci" "thunderbolt" "nvme" "usb_storage" "usbhid" "sd_mod" "ahci" ];
  boot.kernelModules = [ "kvm-intel" ];

  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-label/BOOT";
    fsType = "vfat";
  };

  swapDevices = [ ];

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.intel.updateMicrocode = lib.mkDefault true;
  # Wifi/Bluetooth on these boards need vendor firmware blobs; without
  # this the installed device has ethernet only.
  hardware.enableRedistributableFirmware = true;
}
