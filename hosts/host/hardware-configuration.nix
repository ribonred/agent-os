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
  # Display drivers in the initrd, deliberately: the boot splash can only
  # paint once a DRM device exists, and without early KMS the brand mark
  # appears for a barely-visible flash at the end of boot instead of
  # covering it (seen on a bench install: long black screen with a
  # blinking cursor, then the logo for a blink). i915 is the NUC tier;
  # vmwgfx/virtio_gpu/bochs cover the VMware/QEMU bench VMs. Loading a
  # driver whose hardware is absent is a no-op, so the union is safe on
  # every target.
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
  # Wifi/Bluetooth on these boards need vendor firmware blobs; without
  # this the installed device has ethernet only.
  hardware.enableRedistributableFirmware = true;
}
