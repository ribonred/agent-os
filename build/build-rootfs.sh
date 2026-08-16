#!/usr/bin/env bash
#
# Stage 1 of the golden image: build a complete, ready-to-run rootfs.
#
# Runs ONCE on a build machine with a network connection. The result is a
# directory tree containing the entire shipped system -- every package,
# service, config and binary already in place. Nothing is installed on
# the customer's device; stage 2 (make-image.sh) turns this tree into a
# bootable disk image that is copied onto units byte-for-byte.
#
#   sudo ./build/build-rootfs.sh [--rootfs DIR] [--suite SUITE]
#
# Re-running against an existing rootfs refuses rather than half-updating
# it: a partially rebuilt tree is the one thing that would ship silently
# broken. Delete it and build again.

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$REPO/build"

ROOTFS="${ROOTFS:-$BUILD_DIR/rootfs}"
SUITE="${SUITE:-resolute}"   # Ubuntu 26.04 LTS
MIRROR="${MIRROR:-http://archive.ubuntu.com/ubuntu}"
ARCH="${ARCH:-amd64}"

# The owner's account on the device. Generic on purpose -- this ships to
# customers, so it must never carry a developer's name.
DEVICE_USER="${DEVICE_USER:-admin}"
DEVICE_HOSTNAME="${DEVICE_HOSTNAME:-agentic-os}"

# The Tauri shell binary. Optional: without it the image is a complete
# system with no assistant UI, which is a legitimate thing to build and
# boot while the GUI packaging story is still being settled.
UI_BUNDLE="${UI_BUNDLE:-$REPO/ui/src-tauri/target/release/ui}"

usage() {
    cat <<EOF
usage: sudo $0 [options]

  --rootfs DIR   where to build the tree      (default: $ROOTFS)
  --suite NAME   Ubuntu suite to debootstrap  (default: $SUITE)
  --mirror URL   apt mirror                   (default: $MIRROR)
  --ui PATH      Tauri shell binary to bake in (default: $UI_BUNDLE)
  --no-ui        build a system with no assistant UI
  -h, --help     this message

Must run as root: debootstrap and chroot both require it.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --rootfs) ROOTFS="$2"; shift 2 ;;
        --suite)  SUITE="$2"; shift 2 ;;
        --mirror) MIRROR="$2"; shift 2 ;;
        --ui)     UI_BUNDLE="$2"; shift 2 ;;
        --no-ui)  UI_BUNDLE=""; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 1 ;;
    esac
done

log() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "must run as root (debootstrap + chroot)"

for tool in debootstrap chroot; do
    command -v "$tool" >/dev/null || die "$tool not found -- apt install debootstrap"
done

[ -e "$ROOTFS" ] && die "$ROOTFS already exists.
A half-rebuilt tree is the one thing that ships silently broken, so this
refuses to touch it. Remove it and run again:  sudo rm -rf $ROOTFS"

# ---------------------------------------------------------------------------
# Teardown. Bind mounts inside a rootfs that outlive the script are how a
# stray `rm -rf` reaches the host's /dev or /proc. Unmount on every exit
# path, deepest first.
# ---------------------------------------------------------------------------
cleanup() {
    set +e
    for m in dev/pts dev proc sys run; do
        mountpoint -q "$ROOTFS/$m" && umount -l "$ROOTFS/$m"
    done
    set -e
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
log "Bootstrapping $SUITE ($ARCH) into $ROOTFS"
# ---------------------------------------------------------------------------
mkdir -p "$ROOTFS"
debootstrap --arch="$ARCH" --variant=minbase "$SUITE" "$ROOTFS" "$MIRROR"

# ---------------------------------------------------------------------------
log "Configuring apt sources"
# ---------------------------------------------------------------------------
# main+restricted alone omits most of the desktop; universe and multiverse
# are where the rest lives.
cat > "$ROOTFS/etc/apt/sources.list.d/ubuntu.sources" <<EOF
Types: deb
URIs: $MIRROR
Suites: $SUITE $SUITE-updates $SUITE-backports
Components: main restricted universe multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg

Types: deb
URIs: http://security.ubuntu.com/ubuntu
Suites: $SUITE-security
Components: main restricted universe multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF
rm -f "$ROOTFS/etc/apt/sources.list"

mount --bind /dev     "$ROOTFS/dev"
mount --bind /dev/pts "$ROOTFS/dev/pts"
mount -t proc  proc   "$ROOTFS/proc"
mount -t sysfs sysfs  "$ROOTFS/sys"
mount -t tmpfs tmpfs  "$ROOTFS/run"

# Services must not start while we are installing into a chroot: the
# chroot's systemd is not running, and postgres in particular will try.
cat > "$ROOTFS/usr/sbin/policy-rc.d" <<'EOF'
#!/bin/sh
exit 101
EOF
chmod +x "$ROOTFS/usr/sbin/policy-rc.d"

in_chroot() {
    chroot "$ROOTFS" /usr/bin/env -i \
        HOME=/root PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8 \
        /bin/bash -euo pipefail -c "$1"
}

# ---------------------------------------------------------------------------
log "Installing packages from build/packages.txt"
# ---------------------------------------------------------------------------
PACKAGES="$(grep -vE '^\s*(#|$)' "$BUILD_DIR/packages.txt" | tr '\n' ' ')"
echo "$PACKAGES" | tr ' ' '\n' | grep -c . | xargs printf '  %s packages\n'

in_chroot "apt-get update -qq"
in_chroot "apt-get install -y --no-install-recommends $PACKAGES"

# ---------------------------------------------------------------------------
log "System identity and locale"
# ---------------------------------------------------------------------------
echo "$DEVICE_HOSTNAME" > "$ROOTFS/etc/hostname"
cat > "$ROOTFS/etc/hosts" <<EOF
127.0.0.1	localhost
127.0.1.1	$DEVICE_HOSTNAME
::1		localhost ip6-localhost ip6-loopback
EOF

# Placeholder, same as the previous system config carried. Set the real
# timezone before units ship, or have first boot derive it.
in_chroot "ln -sf /usr/share/zoneinfo/UTC /etc/localtime"
in_chroot "locale-gen en_US.UTF-8 && update-locale LANG=en_US.UTF-8"

# ---------------------------------------------------------------------------
log "Creating the device owner account: $DEVICE_USER"
# ---------------------------------------------------------------------------
# Decide the auth strategy before this leaves the bench. Do not ship a
# device with an unauthenticated user -- this password is a placeholder
# that exists so the account is usable on the bench, nothing more.
in_chroot "
    useradd --create-home --shell /bin/bash --groups sudo,audio,video,plugdev '$DEVICE_USER'
    echo '$DEVICE_USER:changeme' | chpasswd
    mkdir -p /home/$DEVICE_USER/Documents /home/$DEVICE_USER/Downloads
    chown -R $DEVICE_USER:$DEVICE_USER /home/$DEVICE_USER
"

# ---------------------------------------------------------------------------
log "Device services: PostgreSQL and Redis"
# ---------------------------------------------------------------------------
"$BUILD_DIR/scripts/install-services.sh" "$ROOTFS" "$DEVICE_USER"

# ---------------------------------------------------------------------------
log "Local inference: llama.cpp and Ollama"
# ---------------------------------------------------------------------------
"$BUILD_DIR/scripts/install-inference.sh" "$ROOTFS"

# ---------------------------------------------------------------------------
log "Agent runtime: Hermes"
# ---------------------------------------------------------------------------
"$BUILD_DIR/scripts/install-hermes.sh" "$ROOTFS" "$REPO" "$DEVICE_USER"

# ---------------------------------------------------------------------------
log "Desktop session"
# ---------------------------------------------------------------------------
"$BUILD_DIR/scripts/install-desktop.sh" "$ROOTFS" "$DEVICE_USER" "$UI_BUNDLE"

# ---------------------------------------------------------------------------
log "First-boot unit"
# ---------------------------------------------------------------------------
install -D -m 644 "$BUILD_DIR/rootfs-overlay/etc/systemd/system/agentic-firstboot.service" \
    "$ROOTFS/etc/systemd/system/agentic-firstboot.service"
install -D -m 755 "$BUILD_DIR/rootfs-overlay/usr/local/sbin/agentic-firstboot" \
    "$ROOTFS/usr/local/sbin/agentic-firstboot"
in_chroot "systemctl enable agentic-firstboot.service"

# ---------------------------------------------------------------------------
log "Cleaning image-specific state"
# ---------------------------------------------------------------------------
# Everything below exists because one image is copied onto every unit.
# Anything unique left in the tree becomes a fleet-wide duplicate.
in_chroot "apt-get clean"
rm -rf "$ROOTFS/var/lib/apt/lists/"*
rm -f  "$ROOTFS/usr/sbin/policy-rc.d"

# Identical SSH host keys across a fleet would let any unit impersonate
# any other. Regenerated on first boot.
rm -f "$ROOTFS/etc/ssh/ssh_host_"*

# systemd regenerates machine-id when the file exists and is empty. An
# absent file is NOT the same: some systemd versions refuse to boot.
: > "$ROOTFS/etc/machine-id"
rm -f "$ROOTFS/var/lib/dbus/machine-id"
in_chroot "ln -sf /etc/machine-id /var/lib/dbus/machine-id"

rm -rf "$ROOTFS/var/log/"*
rm -rf "$ROOTFS/tmp/"* "$ROOTFS/var/tmp/"*
rm -f  "$ROOTFS/root/.bash_history" "$ROOTFS/home/$DEVICE_USER/.bash_history"

cleanup
trap - EXIT

log "Rootfs complete: $ROOTFS"
du -sh "$ROOTFS" | awk '{print "    size: " $1}'
cat <<EOF

Next: turn this tree into a bootable image

    sudo $BUILD_DIR/make-image.sh --rootfs $ROOTFS

EOF
