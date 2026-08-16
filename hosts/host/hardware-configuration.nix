{ lib, ... }:

# Device-class profile, NOT per-unit. One image installs onto many
# identical units, so nothing here may be unit-specific -- mounts go by
# the labels the installer writes, not per-machine UUIDs. Never replace
# this with `nixos-generate-config` output from a bench unit.
#
# Label contract with hosts/installer/: BOOT -> FAT32 ESP at /boot,
# nixos -> ext4 root. Change them in both places or not at all.

{
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # Wider than any single unit needs, so one image boots across storage
  # variants of the same SKU.
  boot.initrd.availableKernelModules = [ "xhci_pci" "thunderbolt" "nvme" "usb_storage" "usbhid" "sd_mod" "ahci" ];
  # Early KMS: the splash can only paint once a DRM device exists.
  # Without these the brand mark flashes at the end of boot instead of
  # covering it. i915 is the NUC tier, the rest cover bench VMs; loading
  # a driver whose hardware is absent is a no-op.
  boot.initrd.kernelModules = [ "i915" "vmwgfx" "virtio_gpu" "bochs" ];
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
  # Without this the device has ethernet only.
  hardware.enableRedistributableFirmware = true;
}
