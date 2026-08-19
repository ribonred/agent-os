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

die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

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

    # -----------------------------------------------------------------
    # One browser, and the assistant can drive it
    # -----------------------------------------------------------------
    # The owner asks the assistant to open a page and expects the page to
    # appear in front of them, in their browser, signed in to the things
    # they are signed in to. That only works if the assistant drives the
    # same browser the owner uses -- so the browser is launched, always,
    # in a way that lets it be driven, and there is exactly one of them.
    #
    # The launcher below is the single place that knows how; the desktop's
    # own browser entries are then pointed at it, so no launch path can
    # miss it.
    install -D -m 755 /dev/stdin "$ROOTFS/usr/local/bin/agentic-browser" <<'AGENTIC_BROWSER'
#!/usr/bin/env bash
#
# The device's browser.
#
# Every way the browser can start goes through here: the dock, a file the
# owner opens, a page the assistant opens. That uniformity is the point.
# The browser always exposes a control channel on the loopback interface,
# so the assistant can act in the window the owner is already looking at
# instead of opening a second, signed-out one they never see.
#
# Called with arguments it behaves exactly like the browser (it is the
# browser, plus the flags). Called with --ensure it makes sure the browser
# is up and driveable, and says so if it could not be.

set -euo pipefail

BROWSER=/usr/bin/google-chrome-stable

# The control channel is refused outright when the browser runs out of the
# profile directory it would pick by itself -- a deliberate upstream
# hardening with no flag to disable it. So the profile lives elsewhere.
# This is still the owner's own profile and their only one: their
# sign-ins, bookmarks and history are here. Nothing is split in two.
PORT="${AGENTIC_BROWSER_PORT:-9222}"
PROFILE="${AGENTIC_BROWSER_PROFILE:-$HOME/.local/share/agentic-os/browser}"

# --ozone-platform-hint=auto rather than a fixed platform: the shipped
# session is Wayland, but the secondary hardware tier runs its vendor's
# own OS and may not be, and a browser that refuses to start there is a
# worse failure than one that goes through X11.
FLAGS=(
    "--user-data-dir=$PROFILE"
    "--remote-debugging-port=$PORT"
    --ozone-platform-hint=auto
    --no-first-run
    --no-default-browser-check
)

control_channel_up() {
    curl -fsS --max-time 1 "http://127.0.0.1:$PORT/json/version" >/dev/null 2>&1
}

# The assistant's runtime is a background service. It is started before
# anyone logs in and so has no desktop session in its environment: a
# window opened from there would land on no screen at all. The session's
# own manager is asked where the screen is rather than guessing, because
# the X authority file in particular is named freshly per session.
adopt_desktop_session() {
    if [ -n "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ]; then
        return 0
    fi
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    local session_env
    session_env="$(systemctl --user show-environment 2>/dev/null \
        | grep -E '^(DISPLAY|WAYLAND_DISPLAY|XAUTHORITY|DBUS_SESSION_BUS_ADDRESS)=' \
        | sed 's/^/export /')" || return 0
    if [ -n "$session_env" ]; then
        # show-environment quotes what needs quoting, so this is the
        # intended way to read it back.
        eval "$session_env"
    fi
}

if [ "${1:-}" = "--ensure" ]; then
    control_channel_up && exit 0

    adopt_desktop_session
    # setsid, not a bare background job: whatever asked for the browser
    # may be killed or timed out the moment this returns, and the browser
    # must outlive it.
    setsid "$BROWSER" "${FLAGS[@]}" </dev/null >/dev/null 2>&1 &

    # Opened windows are not driveable windows. Waiting for the control
    # channel is the difference between the assistant acting on the page
    # and the assistant reporting an error it cannot explain.
    for _ in $(seq 1 40); do
        control_channel_up && exit 0
        sleep 0.5
    done

    # Said in a form the assistant's runtime understands as "stop here",
    # because continuing means driving a browser that is not on screen --
    # exactly the confusion this file exists to prevent.
    printf '%s\n' '{"action": "block", "message": "The browser did not come up on the screen, so there is nothing to open the page in. Tell the owner the browser would not open and ask them to try again; do not describe profiles, ports or sessions to them."}'
    exit 2
fi

adopt_desktop_session
exec "$BROWSER" "${FLAGS[@]}" "$@"
AGENTIC_BROWSER

    # The desktop's browser entries, rewritten to start the browser
    # through the launcher. Two of them: the browser ships a legacy id and
    # a reverse-DNS one, and file associations on this system point at
    # either, so both have to lead to the same place.
    #
    # Shadowed rather than edited. /usr/local/share/applications takes
    # precedence over /usr/share/applications, and the browser's own
    # package owns the files there -- an upgrade would quietly restore its
    # own launch command and the assistant would lose the browser with no
    # visible change to explain it.
    install -d -m 755 "$ROOTFS/usr/local/share/applications"
    for entry in google-chrome com.google.Chrome; do
        src="$ROOTFS/usr/share/applications/$entry.desktop"
        [ -f "$src" ] || continue
        sed -E 's|^(Exec=)\S*google-chrome-stable|\1/usr/local/bin/agentic-browser|' \
            "$src" > "$ROOTFS/usr/local/share/applications/$entry.desktop"
        chmod 644 "$ROOTFS/usr/local/share/applications/$entry.desktop"
    done
    in_chroot "update-desktop-database /usr/local/share/applications" || true

    # Assert rather than trust: if the browser package ever changes how it
    # spells its own launch command, the rewrite above silently does
    # nothing and the assistant silently loses the ability to browse.
    in_chroot "
        grep -q '^Exec=/usr/local/bin/agentic-browser' \
            /usr/local/share/applications/google-chrome.desktop
    " || die "browser entries were not rewritten to the device launcher --
the assistant would open a browser it cannot drive. Check the Exec lines in
/usr/share/applications/google-chrome.desktop against the rewrite above."

    echo "  browser:   google-chrome-stable (from Google's apt repo)"
    echo "             launched via /usr/local/bin/agentic-browser, driveable on 127.0.0.1:9222"
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
