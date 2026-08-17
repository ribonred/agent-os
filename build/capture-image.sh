#!/usr/bin/env bash
#
# Capture a provisioned machine's disk as a golden image.
#
# Run this from a LIVE USB, with the provisioned machine's disk present
# but not mounted. A filesystem cannot be imaged consistently while it is
# mounted and being written to -- the copy would contain a half-updated
# journal, open database files and a session's worth of churn.
#
#   sudo ./build/capture-image.sh --disk /dev/nvme0n1 --out agentic-os.img
#
# Two things happen here, and the second matters more than the first:
#
#   1. the disk is copied into a file
#   2. everything that must differ between units is stripped first
#
# One image is written byte-for-byte onto every unit, so anything unique
# left behind becomes a fleet-wide duplicate: identical SSH host keys let
# any unit impersonate any other, a shared machine-id breaks anything
# keyed on it, and a leftover bearer token is the same credential on
# every device sold.

set -euo pipefail

DISK="${DISK:-}"
OUT="${OUT:-agentic-os.img}"
COMPRESS="${COMPRESS:-true}"
SKIP_SANITIZE="${SKIP_SANITIZE:-false}"

usage() {
    cat <<EOF
usage: sudo $0 --disk DEV [options]

  --disk DEV     the provisioned machine's disk   (required)
  --out FILE     image to write                   (default: $OUT)
  --no-compress  keep the raw .img only
  --no-sanitize  DANGEROUS: skip per-unit cleanup
  -h, --help     this message

Run from a live USB. The target disk must not be mounted.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --disk) DISK="$2"; shift 2 ;;
        --out)  OUT="$2"; shift 2 ;;
        --no-compress) COMPRESS=false; shift ;;
        --no-sanitize) SKIP_SANITIZE=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 1 ;;
    esac
done

log() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "must run as root"
[ -n "$DISK" ] || { usage >&2; die "--disk is required"; }
[ -b "$DISK" ] || die "not a block device: $DISK"

# Where the image lands is the easiest thing to get wrong here. A live
# USB session's filesystem is RAM, so a default relative path writes into
# memory and is lost at reboot -- and on a machine with less RAM than
# image, fills memory first. Refuse anything that is not on real storage.
OUT_DIR="$(dirname "$OUT")"
[ -d "$OUT_DIR" ] || die "output directory does not exist: $OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
OUT="$OUT_DIR/$(basename "$OUT")"

out_fstype="$(findmnt -no FSTYPE --target "$OUT_DIR")"
case "$out_fstype" in
    tmpfs|ramfs|overlay|squashfs)
        die "$OUT_DIR is in memory ($out_fstype), not on a disk.
A live USB's filesystem does not survive a reboot, and the image is
several GB. Write to external storage instead:

  lsblk                                    # find your USB drive
  sudo mkdir -p /mnt/usb
  sudo mount /dev/sdX1 /mnt/usb            # NOT the disk being imaged
  sudo $0 --disk $DISK --out /mnt/usb/agentic-os.img" ;;
    iso9660|udf)
        die "$OUT_DIR is a read-only disc filesystem ($out_fstype).
This is what a USB written in DD/raw mode looks like -- the whole medium
is the ISO image and nothing can be written to it. Use a second USB
drive for the capture, or rewrite the boot stick in ISO mode (Rufus:
\"Write in ISO Image mode\"), which leaves a writable partition." ;;
esac

# Actually try to write, rather than trusting the mount flags: a stick
# can be mounted rw and still fail on a hardware write-protect switch or
# a filesystem the kernel silently downgraded. Two seconds here saves
# discovering it at the end of a long capture.
if ! touch "$OUT_DIR/.capture-write-test" 2>/dev/null; then
    die "$OUT_DIR is not writable.
Common causes: the medium was written in DD/raw mode (nothing can be
written to it), it was unplugged from Windows without ejecting and is
mounted read-only, or the drive has a physical write-protect switch.

  mount | grep \"\$(findmnt -no TARGET --target '$OUT_DIR')\"

Use a different USB drive, or remount it read-write."
fi
rm -f "$OUT_DIR/.capture-write-test"

# FAT32 cannot hold a file over 4GB, and a compressed capture lands
# uncomfortably close to that. Fail now with the fix rather than partway
# through the write.
if [ "$out_fstype" = "vfat" ]; then
    printf '\033[1;33m  warning: %s is FAT32, which cannot hold a file over 4GB.\n' "$OUT_DIR"
    printf '  A compressed capture is usually 2-4GB, so this may fail near the end.\n'
    printf '  Safer: split the output as it is written --\n\n'
    printf '    dd if=%s bs=4M status=progress \\\n' "$DISK"
    printf '      | zstd -T0 -19 | split -b 3G - %s.zst.part-\n\n' "$OUT"
    printf '  and reassemble when flashing with: cat %s.zst.part-* | zstdcat | dd of=<disk>\033[0m\n\n' "$OUT"
fi

# The destination must also not live on the disk being captured: writing
# into the source while reading it is both a consistency problem and a
# guaranteed out-of-space.
out_src="$(findmnt -no SOURCE --target "$OUT_DIR")"
out_disk="$(lsblk -no PKNAME "$out_src" 2>/dev/null | head -n1 || true)"
[ -n "$out_disk" ] && [ "/dev/$out_disk" = "$DISK" ] && \
    die "$OUT_DIR is on $DISK, the disk being captured -- pick another destination"

out_free="$(df -B1 --output=avail "$OUT_DIR" | tail -1)"

# Imaging the disk this system booted from would capture a live,
# inconsistent filesystem -- and is the most likely mistake here.
root_src="$(findmnt -no SOURCE / 2>/dev/null || true)"
root_disk="$(lsblk -no PKNAME "$root_src" 2>/dev/null | head -n1 || true)"
[ -n "$root_disk" ] && [ "/dev/$root_disk" = "$DISK" ] && \
    die "$DISK is the disk this live system is running from"

mounted="$(lsblk -no MOUNTPOINT "$DISK" | grep -c . || true)"
[ "$mounted" != "0" ] && die "$DISK has mounted partitions -- unmount them first:
  for p in ${DISK}*; do umount \$p 2>/dev/null; done"

MNT="$(mktemp -d)"
cleanup() { set +e; mountpoint -q "$MNT" && umount -R "$MNT"; rmdir "$MNT" 2>/dev/null; set -e; }
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Sanitize
# ---------------------------------------------------------------------------
# Done by mounting the root partition read-write and removing per-unit
# state in place. This modifies the source machine -- which is correct:
# it is a reference unit, not a customer's device, and it regenerates
# everything removed here on its own next boot.
if [ "$SKIP_SANITIZE" != true ]; then
    log "Stripping per-unit state"

    # The root partition is the largest ext4 on the disk. Asking blkid
    # rather than assuming p2 keeps this working across layouts.
    ROOT_PART="$(lsblk -rno NAME,FSTYPE,SIZE "$DISK" \
        | awk '$2=="ext4" {print $1, $3}' \
        | sort -k2 -h | tail -n1 | cut -d' ' -f1)"
    [ -n "$ROOT_PART" ] || die "no ext4 partition found on $DISK"
    ROOT_PART="/dev/$ROOT_PART"
    echo "  root filesystem: $ROOT_PART"

    mount "$ROOT_PART" "$MNT"

    # Identical host keys across a fleet would let any unit impersonate
    # any other. Regenerated by the first-boot unit.
    rm -f "$MNT"/etc/ssh/ssh_host_*

    # Empty, not absent: systemd regenerates it from an empty file, but
    # some versions refuse to boot when the file is missing entirely.
    : > "$MNT/etc/machine-id"
    rm -f "$MNT/var/lib/dbus/machine-id"
    ln -sf /etc/machine-id "$MNT/var/lib/dbus/machine-id"

    # The per-unit bearer token. Regenerated on first boot; leaving it
    # would ship one credential across every device.
    rm -f "$MNT/etc/agentic-os/hermes.env"

    # So the first-boot unit actually runs on each flashed unit rather
    # than seeing this machine's completed stamp.
    rm -f "$MNT/var/lib/agentic-os/.firstboot-done"

    # The reference machine's own history, not the product's.
    rm -rf "$MNT"/var/log/* "$MNT"/tmp/* "$MNT"/var/tmp/*
    rm -f  "$MNT"/root/.bash_history
    rm -f  "$MNT"/home/*/.bash_history
    find "$MNT/home" -maxdepth 2 -name '.ssh' -type d -exec rm -rf {} + 2>/dev/null || true

    # WiFi credentials from the bench network. These are the bench's
    # secrets and have no business on a customer's device.
    rm -f "$MNT"/etc/NetworkManager/system-connections/*

    # Package lists rebuild on demand and are pure image weight.
    rm -rf "$MNT"/var/lib/apt/lists/*
    rm -rf "$MNT"/var/cache/apt/archives/*.deb

    # Zeroing free space makes the sparse copy and the compression far
    # more effective -- deleted files otherwise persist as random bytes
    # that compress to nothing useful.
    echo "  zeroing free space (this takes a while) ..."
    dd if=/dev/zero of="$MNT/ZEROFILL" bs=4M status=none 2>/dev/null || true
    sync
    rm -f "$MNT/ZEROFILL"
    sync

    umount "$MNT"
else
    printf '\033[1;33m  WARNING: --no-sanitize. This image will carry this machine\n'
    printf '  identity onto every unit flashed from it.\033[0m\n'
fi

# ---------------------------------------------------------------------------
# Capture
# ---------------------------------------------------------------------------
log "Capturing $DISK"

disk_size="$(lsblk -bdno SIZE "$DISK")"
printf '    disk:  %s\n' "$(numfmt --to=iec "$disk_size")"
printf '    into:  %s\n' "$OUT"
printf '    free:  %s\n\n' "$(numfmt --to=iec "$out_free")"

# Compressed output is unpredictable but lands far under the raw size
# once free space is zeroed; uncompressed needs the whole disk. Warn
# rather than refuse on the compressed path -- the estimate is a rule of
# thumb, not a measurement.
if [ "$COMPRESS" = true ]; then
    if [ "$out_free" -lt 8000000000 ]; then
        printf '\033[1;33m  warning: under 8GB free. A compressed capture is usually\n'
        printf '  2-4GB, but that depends on how full the device is.\033[0m\n\n'
    fi
elif [ "$out_free" -lt "$disk_size" ]; then
    die "an uncompressed capture needs the full $(numfmt --to=iec "$disk_size"), and $OUT_DIR has $(numfmt --to=iec "$out_free").
Drop --no-compress, or write somewhere larger."
fi

if [ "$COMPRESS" = true ] && command -v zstd >/dev/null; then
    # Straight to compressed: the raw image is the size of the whole
    # disk, and on a 1TB NVMe that is rarely something you want on the
    # capture medium even briefly.
    dd if="$DISK" bs=4M status=progress | zstd -T0 -19 -o "$OUT.zst"
    printf '\n    %s\n' "$(du -h "$OUT.zst" | cut -f1) compressed"
    RESULT="$OUT.zst"
else
    dd if="$DISK" of="$OUT" bs=4M status=progress conv=fsync
    RESULT="$OUT"
fi

sync

log "Captured"
cat <<EOF

    image: $RESULT

Flash another unit with it:

    sudo $(dirname "${BASH_SOURCE[0]}")/flash.sh --image $RESULT

The reference machine you captured from has had its identity stripped.
That is intended -- it regenerates host keys, machine-id and its bearer
token on its next boot, exactly as a flashed unit does.

EOF
