#!/usr/bin/env bash
#
# Turn a freshly-installed Ubuntu machine into an agentic-os device.
#
# This is the other way to build a golden image. Instead of assembling a
# filesystem tree in a container, you install Ubuntu on a real machine
# the ordinary way, run this against it, confirm the result actually
# works, and then capture that disk with capture-image.sh. The captured
# image is what gets flashed onto every other unit.
#
#   sudo ./build/provision.sh
#
# Its advantage over the container path is that it runs on the hardware
# the product ships on, so what you test is exactly what you capture --
# no boot, driver or GPU question is deferred to a later stage. Its cost
# is that a machine's state is harder to reproduce than a script's, which
# is why the same install scripts are shared between both paths.
#
# Requires: Ubuntu 26.04 Desktop, already installed and booted, with a
# network connection. Safe to re-run.

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$REPO/build"

# Provisioning the running system, so the target is the root filesystem
# itself rather than a tree somewhere under it.
ROOTFS=/

# Defaults to whoever is running this, since on a hand-installed
# reference machine that is the account created during the Ubuntu
# install. Falls back to the same name the from-scratch image build uses,
# so both paths produce a device with the same owner account.
DEVICE_USER="${DEVICE_USER:-$(logname 2>/dev/null || echo "${SUDO_USER:-admin-agent}")}"
UI_BUNDLE="${UI_BUNDLE:-$REPO/ui/src-tauri/target/release/ui}"

OLLAMA_SKIP="${OLLAMA_SKIP:-0}"
BROWSER_SKIP="${BROWSER_SKIP:-0}"
HERMES_SSH_KEY="${HERMES_SSH_KEY:-}"

usage() {
    cat <<EOF
usage: sudo $0 [options]

  --user NAME   the device owner's account  (default: $DEVICE_USER)
  --ui PATH     Tauri shell binary to install (default: $UI_BUNDLE)
  --no-ui       provision without the assistant UI
  --key PATH    GitHub-registered SSH key for the agent-runtime clone
  -h, --help    this message

Environment: OLLAMA_SKIP=1 and BROWSER_SKIP=1 behave as in the container
build.

Run this on a freshly installed Ubuntu 26.04 Desktop machine. It
installs and configures everything the device needs; it does not
partition, format or image anything.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --user) DEVICE_USER="$2"; shift 2 ;;
        --ui)   UI_BUNDLE="$2"; shift 2 ;;
        --no-ui) UI_BUNDLE=""; shift ;;
        --key)  HERMES_SSH_KEY="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 1 ;;
    esac
done

log() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || die "must run as root"

# Guard against running this on the build machine by accident. It
# installs services, creates users and rewrites system config -- fine on
# a device being provisioned, wrong anywhere else.
if [ ! -f /etc/os-release ]; then
    die "cannot identify this system"
fi
. /etc/os-release
if [ "${ID:-}" != "ubuntu" ]; then
    die "this expects Ubuntu; found ${PRETTY_NAME:-unknown}"
fi
case "${VERSION_ID:-}" in
    26.04) ;;
    *) printf '\033[1;33mwarning: expected Ubuntu 26.04, found %s\033[0m\n' \
            "${VERSION_ID:-unknown}" >&2 ;;
esac

id -u "$DEVICE_USER" >/dev/null 2>&1 || \
    die "user '$DEVICE_USER' does not exist -- pass --user with the account created during the Ubuntu install"

cat <<EOF

==============================================================
  Provisioning this machine as an agentic-os device

  system:  ${PRETTY_NAME:-unknown}
  owner:   $DEVICE_USER
  UI:      ${UI_BUNDLE:-<none>}

  Installs packages, services and the agent runtime, and
  enables autologin. It does NOT touch partitions.
==============================================================

EOF
printf '  Starting in 10s. Ctrl-C to abort.\n\n'
sleep 10

# ---------------------------------------------------------------------------
log "Installing packages from build/packages.txt"
# ---------------------------------------------------------------------------
# The same list the container build uses, so both paths produce the same
# set of software. Recommends are left on here, unlike the container
# build: this machine already has a full desktop, and suppressing them on
# an existing system removes optional pieces the user can see.
PACKAGES="$(grep -vE '^\s*(#|$)' "$BUILD_DIR/packages.txt" | tr '\n' ' ')"
export DEBIAN_FRONTEND=noninteractive
apt-get update
# shellcheck disable=SC2086
apt-get install -y $PACKAGES

# ---------------------------------------------------------------------------
log "Device services: PostgreSQL and Redis"
# ---------------------------------------------------------------------------
"$BUILD_DIR/scripts/install-services.sh" "$ROOTFS" "$DEVICE_USER"

# ---------------------------------------------------------------------------
log "Local inference: llama.cpp and Ollama"
# ---------------------------------------------------------------------------
OLLAMA_SKIP="$OLLAMA_SKIP" "$BUILD_DIR/scripts/install-inference.sh" "$ROOTFS"

# ---------------------------------------------------------------------------
log "Agent runtime: Hermes"
# ---------------------------------------------------------------------------
HERMES_SSH_KEY="$HERMES_SSH_KEY" \
    "$BUILD_DIR/scripts/install-hermes.sh" "$ROOTFS" "$REPO" "$DEVICE_USER"

# ---------------------------------------------------------------------------
log "Owner account groups"
# ---------------------------------------------------------------------------
# The account already exists here -- it was created by the Ubuntu
# installer -- so only the group membership the device needs is added.
# render and video are what let anything outside the desktop session open
# the GPU; without them a local inference server silently runs on the CPU.
usermod -aG sudo,audio,video,render,plugdev,users "$DEVICE_USER"
id -nG "$DEVICE_USER" | tr ' ' '\n' | grep -E '^(render|video|sudo)$' | sed 's/^/  in group: /'

# ---------------------------------------------------------------------------
log "Desktop session"
# ---------------------------------------------------------------------------
BROWSER_SKIP="$BROWSER_SKIP" \
    "$BUILD_DIR/scripts/install-desktop.sh" "$ROOTFS" "$DEVICE_USER" "$UI_BUNDLE"

# ---------------------------------------------------------------------------
log "First-boot unit"
# ---------------------------------------------------------------------------
install -D -m 644 "$BUILD_DIR/rootfs-overlay/etc/systemd/system/agentic-firstboot.service" \
    /etc/systemd/system/agentic-firstboot.service
install -D -m 755 "$BUILD_DIR/rootfs-overlay/usr/local/sbin/agentic-firstboot" \
    /usr/local/sbin/agentic-firstboot
systemctl daemon-reload
systemctl enable agentic-firstboot.service

# On this machine the unit has already "run" in the sense that the
# machine has an identity. Stamping it now would leave the captured image
# claiming its per-unit setup was done -- so the stamp is deliberately
# NOT written here. capture-image.sh removes it along with the rest of
# the per-unit state.

# ---------------------------------------------------------------------------
log "Starting services"
# ---------------------------------------------------------------------------
# Unlike the container build, this system is running -- so the services
# can actually be started and checked rather than only enabled.
systemctl restart postgresql redis-server || true
systemctl start agentic-pg-init || true

if [ -f /etc/agentic-os/hermes.env ]; then
    systemctl restart hermes-gateway || true
else
    echo "  hermes.env not present yet -- the gateway starts after first boot"
fi

# ---------------------------------------------------------------------------
log "Provisioned"
# ---------------------------------------------------------------------------
cat <<EOF

Check it before capturing an image from this machine:

    systemctl status postgresql redis-server hermes-gateway
    psql -XAtqc "SELECT version()"       # as the hermes user
    curl -sf http://127.0.0.1:8642/health

Then reboot and confirm the whole device works -- autologin, the
desktop, and the assistant. Only once this machine behaves the way a
customer's should:

    sudo $BUILD_DIR/capture-image.sh --disk /dev/nvme0n1 --out agentic-os.img

Run that from a live USB, not from this installed system: a filesystem
cannot be imaged consistently while it is mounted and being written to.

EOF
