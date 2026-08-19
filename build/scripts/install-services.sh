#!/usr/bin/env bash
#
# Device services: PostgreSQL and Redis.
#
# Agent-facing metadata for both lives in registry/*.yaml, kept out of the
# build system so the running agent can read it without any build tooling
# present on the device.
#
# Called by build-rootfs.sh inside a prepared chroot; not meant to be run
# on its own.

set -euo pipefail

ROOTFS="$1"
DEVICE_USER="$2"

in_chroot() {
    chroot "$ROOTFS" /usr/bin/env -i \
        HOME=/root PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8 \
        /bin/bash -euo pipefail -c "$1"
}

# ---------------------------------------------------------------------------
# PostgreSQL
# ---------------------------------------------------------------------------
# Unix socket only -- nothing on this box needs network pg. Ubuntu's
# package listens on localhost:5432 by default, so this is an explicit
# narrowing, not the default.
PGCONF_DIR="$(in_chroot "ls -d /etc/postgresql/*/main 2>/dev/null | head -n1" || true)"

if [ -n "$PGCONF_DIR" ]; then
    in_chroot "
        sed -i \"s/^#\\?listen_addresses.*/listen_addresses = ''/\" '$PGCONF_DIR/postgresql.conf'
    "
else
    echo "warning: no postgresql config dir found -- skipping listen_addresses" >&2
fi

# No per-agent role is created, and none is wanted.
#
# An earlier version made a superuser role named after the account the
# agent was expected to run under, so that a bare `psql` would work by
# peer authentication. That premise stopped being true when the agent
# moved to the device owner's own account, and the role it created was
# named after an account that no longer exists -- so `psql` failed with
# "role does not exist" and the database looked broken to the one thing
# that uses it.
#
# The agent now connects as `postgres` (see brain/skills/device-services).
# That is the role that can govern the others -- create and drop roles,
# grant and revoke, reach every database -- which is what administering
# the device's database actually requires. It grants nothing that could
# not already be taken, since the agent has passwordless root, and it
# removes a boot-time unit that existed only to prop up the bare-psql
# assumption.

# ---------------------------------------------------------------------------
# Redis
# ---------------------------------------------------------------------------
# Loopback only, default port. Ubuntu's default is already 127.0.0.1;
# stated explicitly so a packaging change upstream cannot quietly widen it.
if [ -f "$ROOTFS/etc/redis/redis.conf" ]; then
    in_chroot "
        sed -i 's/^bind .*/bind 127.0.0.1 -::1/' /etc/redis/redis.conf
        sed -i 's/^# *protected-mode .*/protected-mode yes/' /etc/redis/redis.conf
    "
else
    echo "warning: /etc/redis/redis.conf not found -- skipping bind narrowing" >&2
fi

# ---------------------------------------------------------------------------
in_chroot "
    systemctl enable postgresql.service
    systemctl enable redis-server.service
"

echo "  postgresql: unix socket only, reached as the postgres role"
echo "  redis:      127.0.0.1:6379"
