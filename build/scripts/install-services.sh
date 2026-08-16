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

# Peer auth means the role name must match the system user, and the agent
# runs as 'hermes'. Without this role the server is up and unreachable by
# the only thing that uses it.
#
# Superuser grants nothing it could not already take -- the agent has
# passwordless root -- and avoids walling it off behind a permission
# error it cannot explain to the owner.
#
# The chroot has no running postgres, so this cannot be done with psql at
# build time. It runs once on the device, before the agent starts.
install -D -m 755 /dev/stdin "$ROOTFS/usr/local/sbin/agentic-pg-init" <<'EOF'
#!/usr/bin/env bash
# Create the agent's PostgreSQL role. Idempotent: safe to re-run, and it
# must be, because it runs on every boot until it succeeds once.
set -euo pipefail

for _ in $(seq 30); do
    su - postgres -c "pg_isready -q" && break
    sleep 1
done

su - postgres -c "psql -XAtqc \"SELECT 1 FROM pg_roles WHERE rolname='hermes'\"" \
    | grep -q 1 && exit 0

su - postgres -c "psql -XAtqc \"CREATE ROLE hermes LOGIN SUPERUSER\""
echo "created postgresql role: hermes"
EOF

install -D -m 644 /dev/stdin "$ROOTFS/etc/systemd/system/agentic-pg-init.service" <<'EOF'
[Unit]
Description=Create the agent's PostgreSQL role
After=postgresql.service
Requires=postgresql.service
# Ordered before the agent so its first query cannot lose the race with
# role creation.
Before=hermes-gateway.service

[Service]
Type=oneshot
ExecStart=/usr/local/sbin/agentic-pg-init
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

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
    systemctl enable agentic-pg-init.service
"

echo "  postgresql: unix socket only, role 'hermes' created on first boot"
echo "  redis:      127.0.0.1:6379"
