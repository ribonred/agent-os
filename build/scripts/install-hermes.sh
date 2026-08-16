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
HERMES_REF="${HERMES_REF:-v0.19.0}"
HERMES_REPO="${HERMES_REPO:-https://github.com/NousResearch/hermes-agent.git}"

# The agent runs as its own system user, not as the owner. It needs to
# write the owner's files, which is what the shared 'users' group is for.
HERMES_USER=hermes
HERMES_HOME=/var/lib/hermes

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
# In 'users' so it can write the owner's files, and in 'sudo' to match the
# sudoers rule below.
in_chroot "
    id -u $HERMES_USER >/dev/null 2>&1 || \
        useradd --system --create-home --home-dir $HERMES_HOME \
                --shell /bin/bash --groups users,sudo $HERMES_USER
"

# The device runs itself on behalf of a non-technical owner, so the agent
# gets passwordless sudo. A narrow allowlist was rejected: an appliance
# that hits a wall on every unforeseen admin task is not the self-running
# product. The approval gate inside the agent is the safety boundary
# here, not the sudoers file.
install -D -m 440 /dev/stdin "$ROOTFS/etc/sudoers.d/hermes" <<EOF
$HERMES_USER ALL=(ALL) NOPASSWD:SETENV: ALL
EOF
in_chroot "visudo -cf /etc/sudoers.d/$HERMES_USER >/dev/null"

# The owner's home must stay group-writable so the agent can work in it
# without taking ownership of it.
in_chroot "
    chgrp users /home/$DEVICE_USER
    chmod 775 /home/$DEVICE_USER
"

# ---------------------------------------------------------------------------
# Install
# ---------------------------------------------------------------------------
# Upstream's installer is fetched at build time on the build machine's
# network and run inside the chroot. It brings its own uv and Python, so
# it does not depend on the distro's interpreter version.
echo "  installing hermes $HERMES_REF ..."
curl -fsSL https://hermes-agent.nousresearch.com/install.sh \
    > "$ROOTFS/tmp/hermes-install.sh"
chmod +x "$ROOTFS/tmp/hermes-install.sh"

# Root install => FHS layout: code at /usr/local/lib/hermes-agent, command
# at /usr/local/bin/hermes, world-readable uv Python under /usr/local.
in_chroot "/tmp/hermes-install.sh --hermes-home $HERMES_HOME"
rm -f "$ROOTFS/tmp/hermes-install.sh"

# Pin to the tested revision. The installer tracks its default branch, so
# without this the image's agent is whatever upstream shipped that day.
HERMES_CODE="$(in_chroot "readlink -f /usr/local/lib/hermes-agent 2>/dev/null || echo $HERMES_HOME/hermes-agent")"
in_chroot "
    if [ -d '$HERMES_CODE/.git' ]; then
        git -C '$HERMES_CODE' fetch --tags --quiet origin || true
        git -C '$HERMES_CODE' checkout --quiet '$HERMES_REF' || \
            echo 'warning: could not check out $HERMES_REF -- image carries the installer default' >&2
    fi
"

in_chroot "chown -R $HERMES_USER:$HERMES_USER $HERMES_HOME"

# ---------------------------------------------------------------------------
# Identity and skills
# ---------------------------------------------------------------------------
# Identity does NOT go through the workspace: hermes reads it from
# $HERMES_HOME/.hermes/SOUL.md. Installed as a plain file here, and
# re-pinned on every boot by the first-boot/every-boot unit -- the
# constitution IS the identity, so the agent's runtime soul-editing must
# not survive a reboot.
install -D -m 660 "$REPO/brain/constitution.md" \
    "$ROOTFS$HERMES_HOME/.hermes/SOUL.md"
install -D -m 660 "$REPO/brain/skills/device-services/SKILL.md" \
    "$ROOTFS$HERMES_HOME/.hermes/skills/device-services/SKILL.md"

# The canonical copy the every-boot unit restores from, so a rebuild of
# the identity does not require the repo to be present on the device.
install -D -m 644 "$REPO/brain/constitution.md" \
    "$ROOTFS/usr/local/share/agentic-os/constitution.md"
install -D -m 644 "$REPO/brain/skills/device-services/SKILL.md" \
    "$ROOTFS/usr/local/share/agentic-os/skills/device-services/SKILL.md"

in_chroot "chown -R $HERMES_USER:$HERMES_USER $HERMES_HOME/.hermes"

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
# Non-secret switches only -- this file is world-readable. Secrets live in
# /etc/agentic-os/hermes.env, written per unit at first boot.
#
# Bare OpenRouter model id, NOT "openrouter/vendor/model" -- OpenRouter
# rejects the prefixed form. This default is duplicated in the UI shell,
# which must name a model when opening a session; keep the two in step.
install -d -m 755 "$ROOTFS/etc/agentic-os"
install -D -m 644 /dev/stdin "$ROOTFS/etc/agentic-os/hermes-config.yaml" <<'EOF'
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
# cannot elevate; ProtectSystem and ReadWritePaths would keep the
# filesystem read-only outside its own state directory, leaving it unable
# to write the owner's files even as root.
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
Environment=HERMES_ENVIRONMENT_HINT="Treat the host operating system and its package/configuration machinery as internal appliance implementation details. Use them silently when operating the device. Do not volunteer or narrate Linux, Ubuntu, packages, services, or system configuration to the owner. Describe outcomes in terms of the device and the owner's task. If the owner explicitly asks for technical details, answer accurately in plain language.\\n\\nYou administer this device and have full read/write access to it, including root via sudo. If something appears to fail on permissions, it is a real error worth reporting -- not a boundary you should assume and work around.\\n\\n/home/$DEVICE_USER is the owner's home and your working directory. Their files live there, and anything you create for them belongs there -- Documents and Downloads already exist. Use $HERMES_HOME only for your own state, never for the owner's work: they cannot see it, and the file view in the interface shows their home."

# Per-unit secrets: API_SERVER_KEY (bearer token) and OPENROUTER_API_KEY
# (empty when no vendor key was provisioned, in which case the owner
# supplies their own through the UI). Written by the first-boot unit.
EnvironmentFile=/etc/agentic-os/hermes.env

ExecStart=/usr/local/bin/hermes gateway run
# The owner's files are what the agent works on, so it starts there
# rather than in a private workspace they cannot see.
WorkingDirectory=/home/$DEVICE_USER
# Files the agent creates must stay openable by the owner. 0007 would
# close them to everyone outside the group, including the owner.
UMask=0002
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
