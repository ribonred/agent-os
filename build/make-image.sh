#!/usr/bin/env bash
#
# Stage 2 of the golden image: turn a finished rootfs into a bootable
# disk image.
#
# The output is a single file containing a GPT, an ESP and the root
# filesystem -- a complete installed system. Flashing a unit is a raw
# block copy of this file onto its disk; nothing is installed, resolved
# or downloaded at that point. One image provisions any number of
# identical units, entirely offline.
#
#   sudo ./build/make-image.sh [--rootfs DIR] [--out FILE] [--size N]
#
# Partition labels are the contract with flash.sh and with the device's
# own /etc/fstab: BOOT -> FAT32 ESP at /boot/efi, agentic-root -> ext4
# root. Change them in all three places or not at all.

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$REPO/build"

ROOTFS="${ROOTFS:-$BUILD_DIR/rootfs}"
OUT="${OUT:-$BUILD_DIR/agentic-os.img}"
ESP_MB="${ESP_MB:-512}"
# Headroom over the rootfs's actual size. The image must fit the smallest
# disk the product ships on; the device grows the filesystem to fill
# whatever it actually has on first boot.
SLACK_MB="${SLACK_MB:-1024}"

ROOT_LABEL=agentic-root
ESP_LABEL=BOOT

usage() {
    cat <<EOF
usage: sudo $0 [options]

  --rootfs DIR  finished tree from build-rootfs.sh (default: $ROOTFS)
  --out FILE    image to write                     (default: $OUT)
  --slack MB    free space above rootfs size       (default: $SLACK_MB)
  --esp MB      EFI system partition size          (default: $ESP_MB)
  -h, --help    this message

Must run as root: loop devices, mkfs and chroot all require it.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --rootfs) ROOTFS="$2"; shift 2 ;;
        --out)    OUT="$2"; shift 2 ;;
        --slack)  SLACK_MB="$2"; shift 2 ;;
        --esp)    ESP_MB="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 1 ;;
    esac
done

log() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "must run as root"
[ -d "$ROOTFS" ] || die "rootfs not found: $ROOTFS -- run build-rootfs.sh first"

# Stage 1 leaves this behind when it aborts. Imaging a half-built tree
# would produce something that looks like a device and is not one.
[ -f "$ROOTFS/.INCOMPLETE" ] && die "$ROOTFS is from a FAILED build (marked $(cat "$ROOTFS/.INCOMPLETE")).
Remove it and build again:  sudo rm -rf $ROOTFS"

for tool in sgdisk mkfs.vfat mkfs.ext4 losetup rsync; do
    command -v "$tool" >/dev/null || die "$tool not found"
done

MNT="$(mktemp -d)"
LOOP=""

cleanup() {
    set +e
    for m in dev/pts dev proc sys run boot/efi ""; do
        mountpoint -q "$MNT/$m" && umount -l "$MNT/$m"
    done
    [ -n "$LOOP" ] && losetup -d "$LOOP"
    rmdir "$MNT" 2>/dev/null
    set -e
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
log "Sizing"
# ---------------------------------------------------------------------------
ROOTFS_MB=$(du -sm --apparent-size "$ROOTFS" | cut -f1)
ROOT_MB=$((ROOTFS_MB + SLACK_MB))
TOTAL_MB=$((ESP_MB + ROOT_MB + 2))   # +2MB for GPT structures

printf '    rootfs:  %s MB\n' "$ROOTFS_MB"
printf '    root:    %s MB (with %s MB slack)\n' "$ROOT_MB" "$SLACK_MB"
printf '    image:   %s MB\n' "$TOTAL_MB"

# ---------------------------------------------------------------------------
log "Creating $OUT"
# ---------------------------------------------------------------------------
rm -f "$OUT"
# Sparse: the file only occupies what is written, which matters because
# most of the root partition is empty slack.
truncate -s "${TOTAL_MB}M" "$OUT"

sgdisk --clear \
    --new=1:1M:+${ESP_MB}M --typecode=1:ef00 --change-name=1:"$ESP_LABEL" \
    --new=2:0:0            --typecode=2:8300 --change-name=2:"$ROOT_LABEL" \
    "$OUT" >/dev/null

LOOP="$(losetup --find --show --partscan "$OUT")"
sleep 1
[ -b "${LOOP}p1" ] && [ -b "${LOOP}p2" ] || die "partition nodes never appeared for $LOOP"

# Filesystem labels, not just partition names: the device mounts by them,
# so one image boots regardless of which disk node it lands on.
mkfs.vfat -F 32 -n "$ESP_LABEL" "${LOOP}p1" >/dev/null
mkfs.ext4 -q -F -L "$ROOT_LABEL" "${LOOP}p2"

# ---------------------------------------------------------------------------
log "Copying the system"
# ---------------------------------------------------------------------------
mount "${LOOP}p2" "$MNT"
mkdir -p "$MNT/boot/efi"
mount "${LOOP}p1" "$MNT/boot/efi"

# -x stays on one filesystem; the bind mounts from stage 1 are gone by
# now, but a stray mount inside the tree would otherwise be copied in.
rsync -aHAX --numeric-ids -x "$ROOTFS/" "$MNT/"

# ---------------------------------------------------------------------------
log "Writing fstab"
# ---------------------------------------------------------------------------
# By label, never by UUID: mkfs generates a fresh UUID per build, and one
# image is copied onto many units. A UUID here would be a per-unit fact
# baked into a shared image.
cat > "$MNT/etc/fstab" <<EOF
# Mounted by label -- one image serves every unit, so nothing here may be
# specific to the disk it was built on.
LABEL=$ROOT_LABEL  /          ext4  defaults,errors=remount-ro  0 1
LABEL=$ESP_LABEL   /boot/efi  vfat  umask=0077                  0 2
EOF

# ---------------------------------------------------------------------------
log "Installing the bootloader"
# ---------------------------------------------------------------------------
mount --bind /dev     "$MNT/dev"
mount --bind /dev/pts "$MNT/dev/pts"
mount -t proc  proc   "$MNT/proc"
mount -t sysfs sysfs  "$MNT/sys"
mount -t tmpfs tmpfs  "$MNT/run"

# --removable puts the loader at the firmware's fallback path
# (EFI/BOOT/BOOTX64.EFI). That is what makes one image boot on units
# whose firmware has no NVRAM entry for it -- which is every freshly
# flashed unit.
chroot "$MNT" /usr/bin/env -i \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    DEBIAN_FRONTEND=noninteractive \
    /bin/bash -euo pipefail -c "
        grub-install --target=x86_64-efi --efi-directory=/boot/efi \
            --bootloader-id=agentic-os --removable --no-nvram
        update-grub
    "

# No bootloader menu; holding a key at power-on still summons it.
sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=0/' "$MNT/etc/default/grub"
chroot "$MNT" /usr/bin/env -i \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/bash -c "update-grub" >/dev/null 2>&1

cleanup
trap - EXIT

# ---------------------------------------------------------------------------
log "Compressing"
# ---------------------------------------------------------------------------
# The flash side streams this straight to the disk, so the stick only
# ever holds the compressed form.
command -v zstd >/dev/null && {
    zstd -f -19 -T0 --sparse "$OUT" -o "$OUT.zst"
    printf '    %s\n' "$(du -h "$OUT.zst" | cut -f1) compressed"
}

log "Image complete"
printf '    raw:        %s (%s)\n' "$OUT" "$(du -h --apparent-size "$OUT" | cut -f1)"
[ -f "$OUT.zst" ] && printf '    compressed: %s.zst\n' "$OUT"

cat <<EOF

Next: test it in a VM before flashing anything

    qemu-system-x86_64 -m 4096 -enable-kvm \\
      -bios /usr/share/ovmf/OVMF.fd -drive file=$OUT,format=raw

Then flash a unit

    sudo $BUILD_DIR/flash.sh --image $OUT.zst --disk /dev/nvme0n1

EOF
