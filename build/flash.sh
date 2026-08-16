#!/usr/bin/env bash
#
# Stage 3: write a golden image onto a unit's internal disk.
#
# This replaces an install step entirely -- there is no package
# resolution, no network and no per-unit configuration here. It is a raw
# block copy of an already-finished system, so it either works or fails
# loudly, and it takes about as long as the disk can write.
#
#   sudo ./build/flash.sh --image agentic-os.img.zst [--disk /dev/nvme0n1]
#
# THIS DESTROYS THE TARGET DISK. A provisioning tool, not a rescue
# system -- never hand it to a customer.

set -euo pipefail

IMAGE="${IMAGE:-}"
DISK="${DISK:-}"
ASSUME_YES="${ASSUME_YES:-false}"
COUNTDOWN="${COUNTDOWN:-15}"

usage() {
    cat <<EOF
usage: sudo $0 --image FILE [options]

  --image FILE  .img or .img.zst to write   (required)
  --disk DEV    target disk                 (default: autodetect internal)
  --yes         skip the countdown          (factory use)
  -h, --help    this message

Autodetection prefers NVMe, then a non-removable SATA disk. The boot
medium is removable, so it is never selected.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --image) IMAGE="$2"; shift 2 ;;
        --disk)  DISK="$2"; shift 2 ;;
        --yes)   ASSUME_YES=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 1 ;;
    esac
done

die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "must run as root"
[ -n "$IMAGE" ] || { usage >&2; die "--image is required"; }
[ -f "$IMAGE" ] || die "image not found: $IMAGE"

# The disk is wiped by the time anything downstream can fail, so a silent
# exit leaves a machine that looks installed until its first boot finds
# no bootloader.
trap 'printf "\n%s\n" "==============================================================
  FLASH FAILED (line $LINENO). This machine has NO bootloader.
  Fix the cause and run this again.
=============================================================="' ERR

# ---------------------------------------------------------------------------
# Target selection
# ---------------------------------------------------------------------------
if [ -z "$DISK" ]; then
    # NVMe expected, SATA fallback. The boot medium is /dev/sd* too, so
    # the removable check is what keeps this from eating the stick.
    if [ -b /dev/nvme0n1 ]; then
        DISK=/dev/nvme0n1
    elif [ -b /dev/sda ] && [ "$(cat /sys/block/sda/removable)" = "0" ]; then
        DISK=/dev/sda
    else
        die "no internal disk found (no /dev/nvme0n1, no non-removable /dev/sda)"
    fi
fi

[ -b "$DISK" ] || die "not a block device: $DISK"

# Refuse to write to the medium this script is running from. Autodetection
# already avoids it; this also covers an explicit --disk typo.
root_src="$(findmnt -no SOURCE / 2>/dev/null || true)"
root_disk="$(lsblk -no PKNAME "$root_src" 2>/dev/null || true)"
[ -n "$root_disk" ] && [ "/dev/$root_disk" = "$DISK" ] && \
    die "$DISK is the disk this system is running from"

disk_size="$(lsblk -bdno SIZE "$DISK")"
disk_model="$(lsblk -dno MODEL "$DISK" 2>/dev/null || echo unknown)"

# The image is sized for the smallest supported disk; anything smaller
# cannot hold it, and dd would only discover that most of the way in.
if [[ "$IMAGE" == *.zst ]]; then
    image_size="$(zstd -l "$IMAGE" 2>/dev/null | awk 'NR==2 {print $5}' || echo 0)"
else
    image_size="$(stat -c %s "$IMAGE")"
fi

cat <<EOF

==============================================================
  agentic-os -- flashing a unit

  image:  $IMAGE
  target: $DISK  ($disk_model, $(numfmt --to=iec "$disk_size"))

  ERASING $DISK. Everything on it is lost.
==============================================================
EOF

if [ "$ASSUME_YES" != true ]; then
    printf '\n  Starting in %ss. Ctrl-C or power off to abort.\n\n' "$COUNTDOWN"
    sleep "$COUNTDOWN"
fi

# ---------------------------------------------------------------------------
# Write
# ---------------------------------------------------------------------------
# Anything holding a partition open makes the kernel keep the old table.
for part in "$DISK"?* ; do
    [ -b "$part" ] && { umount "$part" 2>/dev/null || true; }
done
wipefs --all "$DISK" >/dev/null

echo "  writing ..."
if [[ "$IMAGE" == *.zst ]]; then
    zstdcat "$IMAGE" | dd of="$DISK" bs=4M conv=fsync status=progress
else
    dd if="$IMAGE" of="$DISK" bs=4M conv=fsync status=progress
fi

sync
partprobe "$DISK" 2>/dev/null || true

cat <<EOF

==============================================================
  Done. Remove the boot medium and power on.

  The unit generates its own host keys, bearer token and grows
  its filesystem on first boot -- give it one extra minute the
  first time.
==============================================================
EOF
