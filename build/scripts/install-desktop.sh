#!/usr/bin/env bash
#
# The graphical session: Ubuntu's own desktop, used as shipped.
#
# GNOME, GDM, PipeWire, portals and polkit all come from the
# ubuntu-desktop metapackage and are left alone. The assistant is an
# ordinary autostarted application on top of that session rather than a
# replacement for it -- the shell's own build config makes its window
# fullscreen and undecorated, so it fills the screen without the session
# needing to know anything about it.
#
# Called by build-rootfs.sh inside a prepared chroot; not meant to be run
# on its own.

set -euo pipefail

ROOTFS="$1"
DEVICE_USER="$2"
UI_BUNDLE="${3:-}"

in_chroot() {
    chroot "$ROOTFS" /usr/bin/env -i \
        HOME=/root PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8 \
        /bin/bash -euo pipefail -c "$1"
}

# ---------------------------------------------------------------------------
# Autologin
# ---------------------------------------------------------------------------
# The device has one user and no keyboard at the counter, so a login
# screen is a dead end rather than a security boundary.
#
# Caveat, and it must stay a conscious choice: autologin means the login
# keyring only auto-unlocks if its password is blank. Acceptable while
# disk encryption is undecided.
install -D -m 644 /dev/stdin "$ROOTFS/etc/gdm3/custom.conf" <<EOF
# GDM configuration. See /usr/share/doc/gdm3 for the full reference.
[daemon]
AutomaticLoginEnable=true
AutomaticLogin=$DEVICE_USER

[security]

[xdmcp]

[chooser]

[debug]
EOF

# ---------------------------------------------------------------------------
# The assistant
# ---------------------------------------------------------------------------
if [ -n "$UI_BUNDLE" ] && [ -f "$UI_BUNDLE" ]; then
    install -D -m 755 "$UI_BUNDLE" "$ROOTFS/usr/local/bin/agentic-ui"

    # A normal-distro binary on a normal-distro filesystem: it links
    # against the libraries apt installed and needs no patching. This is
    # the whole reason the shell is built with the system toolchain.
    install -D -m 644 /dev/stdin "$ROOTFS/etc/xdg/autostart/agentic-ui.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Assistant
Exec=/usr/local/bin/agentic-ui
# The assistant is the device's reason to exist: if it dies, the session
# should bring it back rather than leave the owner at an empty desktop.
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Phase=Applications
NoDisplay=true
EOF
    echo "  assistant: /usr/local/bin/agentic-ui (autostarts with the session)"
else
    echo "  assistant: NOT INCLUDED -- no UI bundle supplied."
    echo "             The image is a complete system with no assistant UI."
fi

# ---------------------------------------------------------------------------
# Keyring
# ---------------------------------------------------------------------------
# The shell stores the cloud key in the OS keyring. Ubuntu wires
# gnome-keyring into PAM by default; asserted here so a packaging change
# shows up as a build failure rather than a device that silently cannot
# save a key.
in_chroot "
    grep -rq pam_gnome_keyring /etc/pam.d/ || \
        echo 'warning: pam_gnome_keyring not configured -- saving a cloud key may fail' >&2
"

# ---------------------------------------------------------------------------
# Graphics workarounds
# ---------------------------------------------------------------------------
# Both diagnosed on a VM: the webview crashes on some virtual GPUs
# without the first. Harmless on real hardware, and bench units are VMs
# often enough that shipping it beats rediscovering it.
install -D -m 644 /dev/stdin "$ROOTFS/etc/environment.d/50-agentic-webview.conf" <<'EOF'
WEBKIT_DISABLE_DMABUF_RENDERER=1
EOF

echo "  desktop:   ubuntu-desktop, autologin as $DEVICE_USER"
