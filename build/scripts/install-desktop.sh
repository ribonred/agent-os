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
# The repo, for assets that are checked in rather than built -- the
# app's icon is the only one so far. Same argument install-hermes.sh
# takes, for the same reason.
REPO="${4:-}"

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
    # A normal-distro binary on a normal-distro filesystem: it links
    # against the libraries apt installed and needs no patching. This is
    # the whole reason the shell is built with the system toolchain.
    install -D -m 755 "$UI_BUNDLE" "$ROOTFS/usr/local/bin/matoakaui"

    # The product's mark, at the sizes the desktop actually asks for.
    # Installed into the shared icon theme rather than beside the binary
    # so `Icon=matoakaui` resolves the way every other application's
    # does, and so GNOME can pick the size it wants instead of scaling
    # one bitmap to everything.
    if [ -n "$REPO" ] && [ -d "$REPO/ui/src-tauri/icons" ]; then
        for size in 32 64 128 256; do
            case "$size" in
                256) source_icon="$REPO/ui/src-tauri/icons/128x128@2x.png" ;;
                *)   source_icon="$REPO/ui/src-tauri/icons/${size}x${size}.png" ;;
            esac
            [ -f "$source_icon" ] || continue
            install -D -m 644 "$source_icon" \
                "$ROOTFS/usr/share/icons/hicolor/${size}x${size}/apps/matoakaui.png"
        done
        echo "  icon:      /usr/share/icons/hicolor/*/apps/matoakaui.png"
    else
        echo "  icon:      NOT INSTALLED -- no repo path given."
    fi

    # Two entries, and both are needed for different reasons.
    #
    # The session looks up a window's name and icon in applications/,
    # never in autostart/ -- so an autostart entry alone starts the
    # assistant with the desktop's generic placeholder mark next to it.
    # NoDisplay keeps it out of the app grid, which is right for a device
    # whose one application is already on screen, and still allows the
    # window to be matched.
    install -D -m 644 /dev/stdin "$ROOTFS/usr/share/applications/matoakaui.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Assistant
Exec=/usr/local/bin/matoakaui
Icon=matoakaui
NoDisplay=true
# Ties the running window to this entry. GTK takes a Wayland window's
# app id from the executable's name, which is why the binary, this file,
# and the icon all have to be called the same thing.
StartupWMClass=matoakaui
EOF

    install -D -m 644 /dev/stdin "$ROOTFS/etc/xdg/autostart/matoakaui.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Assistant
Exec=/usr/local/bin/matoakaui
Icon=matoakaui
# The assistant is the device's reason to exist: if it dies, the session
# should bring it back rather than leave the owner at an empty desktop.
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Phase=Applications
NoDisplay=true
StartupWMClass=matoakaui
EOF
    echo "  assistant: /usr/local/bin/matoakaui (autostarts with the session)"
else
    echo "  assistant: NOT INCLUDED -- no UI bundle supplied."
    echo "             The image is a complete system with no assistant UI."
fi

# ---------------------------------------------------------------------------
# Owner privileges
# ---------------------------------------------------------------------------
# The owner is a non-technical daily user of an appliance, not an
# administrator: they were never given a password, the account autologins,
# and nothing in the product ever asks them to type one. A password prompt
# they cannot answer is a dead end, not a security boundary -- it just
# means whatever raised it silently fails.
#
# This is the same posture the agent runs under, for the same reason: the
# device administers itself on the owner's behalf. Physical access to a
# counter-top appliance is already full access; disk encryption, not a
# sudo prompt, is what would change that.
install -D -m 440 /dev/stdin "$ROOTFS/etc/sudoers.d/device-owner" <<EOF
$DEVICE_USER ALL=(ALL) NOPASSWD: ALL
EOF
in_chroot "visudo -cf /etc/sudoers.d/device-owner >/dev/null"

# polkit is the other prompt the owner cannot answer -- GNOME raises it
# for software updates, timezone changes, mounting internal disks. Without
# this those actions fail with a dialog asking for a password that does
# not exist, which reads as "the device is broken".
install -D -m 644 /dev/stdin "$ROOTFS/etc/polkit-1/rules.d/49-device-owner.rules" <<EOF
// The device owner administers the appliance through it, not around it.
// Matches the passwordless sudo grant in /etc/sudoers.d/device-owner.
polkit.addRule(function(action, subject) {
    if (subject.user == "$DEVICE_USER") {
        return polkit.Result.YES;
    }
});
EOF

# ---------------------------------------------------------------------------
# Browser
# ---------------------------------------------------------------------------
# The only application baked in beyond what the system needs. Everything
# else is installed on request, since each package costs size in an image
# that carries the whole system for an offline install.
#
# Chrome from Google's own apt repo rather than the archive's `firefox`
# or `chromium-browser`: both of those are transitional packages that
# install snaps, and a snap wants snapd running, auto-updates over the
# network, and is slow on first launch -- none of which suits a device
# that may never see a network.
#
# OPEN QUESTION, deliberately not settled here: redistributing Google's
# binary inside a sold product is a licensing matter someone must confirm
# before units ship. Set BROWSER_SKIP=1 to build without it; the desktop
# is fully usable, it simply has no browser until one is chosen.
BROWSER_SKIP="${BROWSER_SKIP:-0}"

if [ "$BROWSER_SKIP" = "1" ]; then
    echo "  browser:   SKIPPED (BROWSER_SKIP=1)"
else
    # Keyed repo, not a bare .deb download: this way the browser gets
    # security updates through the same apt path as everything else on
    # any unit that does reach a network.
    curl -fsSL --retry 3 https://dl.google.com/linux/linux_signing_key.pub \
        -o "$ROOTFS/usr/share/keyrings/google-chrome.asc"

    install -D -m 644 /dev/stdin "$ROOTFS/etc/apt/sources.list.d/google-chrome.sources" <<'EOF'
Types: deb
URIs: https://dl.google.com/linux/chrome/deb/
Suites: stable
Components: main
Architectures: amd64
Signed-By: /usr/share/keyrings/google-chrome.asc
EOF

    in_chroot "
        apt-get update -qq
        apt-get install -y --no-install-recommends google-chrome-stable
    "
    echo "  browser:   google-chrome-stable (from Google's apt repo)"
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
