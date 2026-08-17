#!/usr/bin/env bash
#
# Hermes Agent: the device's agent runtime. The UI shell is a frontend to
# it -- it provides the session API the shell drives, persistent memory, a
# skill system, and messaging bridges.
#
# Installed with upstream's own installer, which on a root Linux install
# puts the checkout at /usr/local/lib/hermes-agent and the command at
# /usr/local/bin/hermes. This is a normal FHS install: a git checkout plus
# a Python venv, no build system of ours involved.
#
# `hermes gateway` then runs as a system service on 127.0.0.1:8642,
# bearer-authenticated. Loopback-only matters: the API grants the agent's
# full toolset including terminal access.
#
# Called by build-rootfs.sh inside a prepared chroot; not meant to be run
# on its own.

set -euo pipefail

ROOTFS="$1"
REPO="$2"
DEVICE_USER="$3"

# Pin the agent runtime. An unpinned installer means two builds a week
# apart ship different agents with no record of what changed -- the
# device's whole behaviour lives in this component.
#
# Upstream tags by date, NOT by the version its CLI reports: `hermes
# --version` says 0.19.0 while the tag for that build is v2026.7.20.
# Check `git ls-remote --tags` before bumping this.
HERMES_REF="${HERMES_REF:-v2026.8.13}"
HERMES_REPO="${HERMES_REPO:-https://github.com/NousResearch/hermes-agent.git}"

# The agent runs AS the owner, not as a separate service account.
#
# The device is a single-user appliance: the owner's files are the only
# files, and everything the agent does is on their behalf. A separate
# account bought nothing here and cost a great deal -- every file the
# agent created needed group-writability to stay openable by the owner,
# the agent's own state landed somewhere the owner could not see, and
# desktop-session resources (the GPU, the keyring, the display) were
# granted to the owner's session and invisible to the service.
#
# HERMES_HOME therefore follows the upstream default relative to that
# account: ~/.hermes, exactly as an interactive `hermes` install uses.
HERMES_USER="$DEVICE_USER"
HERMES_HOME="/home/$DEVICE_USER/.hermes"

die() { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

in_chroot() {
    chroot "$ROOTFS" /usr/bin/env -i \
        HOME=/root PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8 \
        HERMES_HOME="$HERMES_HOME" \
        /bin/bash -euo pipefail -c "$1"
}

# ---------------------------------------------------------------------------
# Account
# ---------------------------------------------------------------------------
# Nothing to create: the agent runs as the owner, whose account already
# exists. Its passwordless sudo comes from the same grant the owner has
# (install-desktop.sh) -- the device administers itself on their behalf,
# and a narrow allowlist was rejected because an appliance that hits a
# wall on every unforeseen admin task is not the self-running product.
# The approval gate inside the agent is the safety boundary here, not the
# sudoers file.
in_chroot "id -u '$HERMES_USER' >/dev/null 2>&1" \
    || die "device owner '$HERMES_USER' does not exist -- create it before installing the agent"

install -d -m 700 -o "$HERMES_USER" -g "$HERMES_USER" "$ROOTFS$HERMES_HOME" 2>/dev/null \
    || in_chroot "install -d -m 700 -o '$HERMES_USER' -g '$HERMES_USER' '$HERMES_HOME'"

# ---------------------------------------------------------------------------
# Install
# ---------------------------------------------------------------------------
# Upstream's installer is fetched at build time on the build machine's
# network and run inside the chroot. It brings its own uv and Python, so
# it does not depend on the distro's interpreter version.
echo "  installing hermes ..."

HERMES_CODE_DIR=/usr/local/lib/hermes-agent

# Upstream's installer, run as-is. It clones the source, brings its own
# uv and Python, and sets up the venv.
#
# The clone is ~226MB and is by far the most failure-prone step of the
# whole build. Over HTTPS it crawled and repeatedly died mid-pack
# ("RPC failed ... transfer closed"), killing three image builds. Over
# SSH, on the same machine and connection, the identical clone completes
# in under four minutes -- so the installer's own "try SSH first" path is
# worth enabling rather than letting it fall through to HTTPS.
#
# Set HERMES_SSH_KEY to a GitHub-registered private key to take that
# path. Without it the build still works, just over HTTPS.
#
# The key is copied in for the install and removed immediately after: a
# personal credential must never reach the image, which is copied
# byte-for-byte onto every unit.
HERMES_SSH_KEY="${HERMES_SSH_KEY:-}"

if [ -n "$HERMES_SSH_KEY" ] && [ -f "$HERMES_SSH_KEY" ]; then
    install -D -m 700 -d "$ROOTFS/root/.ssh"
    install -m 600 "$HERMES_SSH_KEY" "$ROOTFS/root/.ssh/id_build"
    cat > "$ROOTFS/root/.ssh/config" <<'EOF'
Host github.com
    HostName github.com
    User git
    IdentityFile /root/.ssh/id_build
    IdentitiesOnly yes
    StrictHostKeyChecking no
    BatchMode yes
EOF
    chmod 600 "$ROOTFS/root/.ssh/config"
    echo "  using SSH for the clone (key: $HERMES_SSH_KEY)"
fi

# Whatever happens below, the credential does not survive this function.
drop_build_key() {
    rm -f "$ROOTFS/root/.ssh/id_build" "$ROOTFS/root/.ssh/config"
    rmdir "$ROOTFS/root/.ssh" 2>/dev/null || true
}
trap drop_build_key EXIT

in_chroot "
    curl -fsSL https://hermes-agent.nousresearch.com/install.sh \
        | bash -s -- --hermes-home $HERMES_HOME
"

# Pin to the tested revision. The installer tracks upstream's default
# branch, so without this two builds a week apart ship different agents
# with no record of what changed -- and the agent's behaviour IS the
# product.
#
# This runs BEFORE the build key is dropped: the installer clones
# shallow, so the tag has to be fetched, and a fetch needs the same
# credentials the clone used. Dropping the key first made this silently
# fall back to the default branch.
in_chroot "
    git -C '$HERMES_CODE_DIR' fetch --depth 1 origin tag '$HERMES_REF'
    git -C '$HERMES_CODE_DIR' checkout --quiet 'refs/tags/$HERMES_REF'
" || die "could not pin the agent runtime to $HERMES_REF.
The image would ship whatever was on upstream's default branch, which is
a silent change to the product's behaviour. Check the tag exists:
  git ls-remote --tags $HERMES_REPO | grep $HERMES_REF"

drop_build_key
trap - EXIT

# Assert rather than trust: report the revision actually checked out, and
# fail if it is not the pinned one. Printing the intended tag while the
# tree sat on another revision is exactly how an unpinned agent would
# reach a customer unnoticed.
actual_ref="$(in_chroot "git -C '$HERMES_CODE_DIR' describe --tags --exact-match 2>/dev/null \
                        || git -C '$HERMES_CODE_DIR' rev-parse --short HEAD")"
[ "$actual_ref" = "$HERMES_REF" ] || die "agent runtime is at '$actual_ref', expected '$HERMES_REF'"
echo "  hermes revision: $actual_ref"

in_chroot "chown -R $HERMES_USER:$HERMES_USER $HERMES_HOME"

# ---------------------------------------------------------------------------
# Identity and skills
# ---------------------------------------------------------------------------
# Identity does NOT go through the workspace: hermes reads it from
# $HERMES_HOME/SOUL.md -- directly in the home, NOT in a .hermes/
# subdirectory. An earlier version wrote to $HERMES_HOME/.hermes/, which
# the agent never reads: the constitution and the device skill were both
# present on disk and silently unused, so the shipped agent ran with its
# upstream identity instead of ours.
#
# Re-pinned on every boot by the first-boot unit -- the constitution IS
# the identity, so the agent's runtime soul-editing must not survive a
# reboot.
install -D -m 660 "$REPO/brain/constitution.md" \
    "$ROOTFS$HERMES_HOME/SOUL.md"
install -D -m 660 "$REPO/brain/skills/device-services/SKILL.md" \
    "$ROOTFS$HERMES_HOME/skills/device-services/SKILL.md"

# The canonical copy the every-boot unit restores from, so a rebuild of
# the identity does not require the repo to be present on the device.
install -D -m 644 "$REPO/brain/constitution.md" \
    "$ROOTFS/usr/local/share/agentic-os/constitution.md"
install -D -m 644 "$REPO/brain/skills/device-services/SKILL.md" \
    "$ROOTFS/usr/local/share/agentic-os/skills/device-services/SKILL.md"

in_chroot "chown -R $HERMES_USER:$HERMES_USER $HERMES_HOME"

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
# Written to $HERMES_HOME/config.yaml, which is the ONLY path the agent
# reads (`get_hermes_home() / "config.yaml"`). An earlier version put this
# at /etc/agentic-os/hermes-config.yaml, where nothing ever loaded it --
# the model, memory and approval settings below all silently did nothing.
#
# Non-secret switches only. Secrets live in /etc/agentic-os/hermes.env,
# written per unit at first boot.
#
# Bare OpenRouter model id, NOT "openrouter/vendor/model" -- OpenRouter
# rejects the prefixed form. This default is duplicated in the UI shell,
# which must name a model when opening a session; keep the two in step.
install -d -m 755 "$ROOTFS/etc/agentic-os"
install -D -m 660 /dev/stdin "$ROOTFS$HERMES_HOME/config.yaml" <<EOF
# The agent's working directory. Set here rather than as TERMINAL_CWD in
# .env: the env var is deprecated and the agent warns about it on every
# start. This is the owner's home, because their files are what it works
# on -- see the environment hint in the service unit.
terminal:
  cwd: /home/$DEVICE_USER

EOF

cat >> "$ROOTFS$HERMES_HOME/config.yaml" <<'EOF'
model:
  provider: openrouter
  default: deepseek/deepseek-v4-flash-0731
  base_url: https://openrouter.ai/api/v1
  api_mode: chat_completions

memory:
  memory_enabled: true
  user_profile_enabled: true
  write_approval: false

# The agent administers this device, so the safety posture is "confirm
# before harm", not "restrict what it can reach": harmless acts run,
# dangerous ones raise an approval the owner taps, and the agent's own
# hard floor blocks unrecoverable ones outright.
#
# `deny` is deliberately tiny. The owner has no separate admin account and
# no settings UI, so relaxing a rule happens THROUGH the agent -- banning
# policy edits would lock them out of their own device. The carve-out is
# only the provisioned credentials: leaking those bills the vendor and is
# never the owner administering anything.
approvals:
  mode: "off"
  deny:
    - "*/etc/agentic-os/hermes.env*"
    - "*cloud-keys.toml*"
EOF

# The agent runs as its own user and must be able to read -- and rewrite,
# when the owner changes something through it -- its own config.
in_chroot "chown $HERMES_USER:$HERMES_USER '$HERMES_HOME/config.yaml'"

# ---------------------------------------------------------------------------
# Service
# ---------------------------------------------------------------------------
# Upstream ships a *user* service for its messaging bridges. This is a
# different thing: the loopback session API the UI shell drives, which has
# to be up before anyone logs in.
#
# No systemd sandboxing, deliberately. The agent administers this device,
# so the sandbox has to be off rather than merely loosened:
# NoNewPrivileges makes the kernel refuse setuid escalation, so sudo
# cannot elevate, and ProtectHome would hide the very files it exists to
# work on.
install -D -m 644 /dev/stdin "$ROOTFS/etc/systemd/system/hermes-gateway.service" <<EOF
[Unit]
Description=Hermes Agent gateway (device session API)
After=network.target postgresql.service redis-server.service
Wants=postgresql.service redis-server.service
StartLimitIntervalSec=600
StartLimitBurst=5

[Service]
Type=simple
User=$HERMES_USER
Group=$HERMES_USER
Environment=HERMES_HOME=$HERMES_HOME
Environment=API_SERVER_ENABLED=true
Environment=API_SERVER_HOST=127.0.0.1
Environment=API_SERVER_PORT=8642
# The appliance boundary, stated at the same layer where host facts are
# injected: the agent uses the OS, the owner uses the device.
Environment=HERMES_ENVIRONMENT_HINT="Treat the host operating system and its package/configuration machinery as internal appliance implementation details. Use them silently when operating the device. Do not volunteer or narrate Linux, Ubuntu, packages, services, or system configuration to the owner. Describe outcomes in terms of the device and the owner's task. If the owner explicitly asks for technical details, answer accurately in plain language.\\n\\nYou administer this device and have full read/write access to it, including root via sudo. If something appears to fail on permissions, it is a real error worth reporting -- not a boundary you should assume and work around.\\n\\n/home/$DEVICE_USER is the owner's home and your working directory. Their files live there, and anything you create for them belongs there -- Documents and Downloads already exist. Keep your own state in $HERMES_HOME and leave the rest of their home to them: it is what the file view in the interface shows, so anything you scatter there is clutter they have to look at."

# Per-unit secrets: API_SERVER_KEY (bearer token) and OPENROUTER_API_KEY
# (empty when no vendor key was provisioned, in which case the owner
# supplies their own through the UI). Written by the first-boot unit.
EnvironmentFile=/etc/agentic-os/hermes.env

ExecStart=/usr/local/bin/hermes gateway run
# The owner's files are what the agent works on, so it starts there.
# Running as that same account, everything it creates is already theirs
# -- no group-writability juggling needed.
WorkingDirectory=/home/$DEVICE_USER
# GPU access for anything the agent runs: the desktop session gets it
# from a logind ACL on the active seat, but a system service has no seat
# and would silently fall back to the CPU.
SupplementaryGroups=render video
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

in_chroot "systemctl enable hermes-gateway.service"

echo "  hermes:  /usr/local/lib/hermes-agent  ($HERMES_REF)"
echo "  gateway: 127.0.0.1:8642 as system service"
