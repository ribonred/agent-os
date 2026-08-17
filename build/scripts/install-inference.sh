#!/usr/bin/env bash
#
# Local inference: Ollama and llama.cpp.
#
# Two engines rather than one, because they answer different questions.
# llama.cpp comes from the archive (see packages.txt) and is the floor:
# a single GGUF file and `llama-server` give the device an
# OpenAI-compatible endpoint with no daemon to manage and no model
# registry to reach. Ollama sits above it -- model pulls, hot-swapping,
# and a library the agent can name models from -- and is what the
# hardware probe routes to when the device can actually run a useful
# model locally.
#
# NEITHER downloads a model at build time, and neither is enabled by
# default. Model acquisition is a multi-GB transfer that must be a
# visible onboarding step with progress and consent, not something that
# happens silently at first networked boot. Until the owner chooses a
# model, routing leans cloud -- which is the honest state of a fresh
# device.
#
# Called by build-rootfs.sh inside a prepared chroot; not meant to be run
# on its own.

set -euo pipefail

ROOTFS="$1"

# Pin the runtime. An unpinned tarball means two builds a week apart ship
# different inference engines with no record of what changed.
OLLAMA_VERSION="${OLLAMA_VERSION:-v0.32.13}"

# The base linux-amd64 tarball bundles CUDA and ROCm runtimes and is
# ~1.4GB compressed -- a large share of the image for hardware the
# mini-PC tier does not have. Set OLLAMA_SKIP=1 to build an image with
# llama.cpp only; the agent still has a local engine.
OLLAMA_SKIP="${OLLAMA_SKIP:-0}"

OLLAMA_USER=ollama
OLLAMA_HOME=/var/lib/ollama

in_chroot() {
    chroot "$ROOTFS" /usr/bin/env -i \
        HOME=/root PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8 \
        /bin/bash -euo pipefail -c "$1"
}

# ---------------------------------------------------------------------------
# llama.cpp
# ---------------------------------------------------------------------------
# Installed from the archive by packages.txt; this only asserts the
# binary landed and sets up where models live.
if ! in_chroot "command -v llama-server >/dev/null"; then
    echo "error: llama-server missing -- llama.cpp-tools failed to install" >&2
    exit 1
fi

# Models are owner data, not system state: they are large, they are
# chosen during onboarding, and the owner should be able to see them.
in_chroot "install -d -m 775 -o root -g users /var/lib/agentic-os/models"

# Not enabled. Started by the agent once a model exists -- an
# always-running server with no model to serve is a failed unit in the
# owner's face on every boot.
#
# --host 127.0.0.1 is the boundary that matters: this endpoint is
# unauthenticated, and the agent is the only thing that should reach it.
install -D -m 644 /dev/stdin "$ROOTFS/etc/systemd/system/llama-server.service" <<'EOF'
[Unit]
Description=llama.cpp inference server
After=network.target
# Started on demand once a model is present, never enabled at build
# time: the model path below does not exist on a fresh device.
ConditionPathExists=/var/lib/agentic-os/models/active.gguf

[Service]
Type=simple
User=nobody
Group=users
# Without these the process cannot open /dev/dri/renderD128 and every
# layer silently runs on the CPU -- which looks like a working server
# answering at a few tokens a second, not like an error. The desktop
# session gets GPU access from a logind ACL on the active seat; a system
# service has no seat, so group membership is the only route.
SupplementaryGroups=render video
# The Vulkan driver compiles shaders on first use and caches them. With
# nowhere writable it falls back to recompiling every start, so give it
# a real directory rather than letting it try $HOME.
StateDirectory=llama-server
Environment=XDG_CACHE_HOME=/var/lib/llama-server
ExecStart=/usr/bin/llama-server \
    --model /var/lib/agentic-os/models/active.gguf \
    --host 127.0.0.1 \
    --port 8080 \
    --ctx-size 4096 \
    -ngl 99
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

echo "  llama.cpp: llama-server on 127.0.0.1:8080 (inactive until a model exists)"

# ---------------------------------------------------------------------------
# Ollama
# ---------------------------------------------------------------------------
if [ "$OLLAMA_SKIP" = "1" ]; then
    echo "  ollama:    SKIPPED (OLLAMA_SKIP=1) -- llama.cpp only"
    exit 0
fi

# Not in the Ubuntu archive, so it comes from upstream's release tarball.
# Fetched on the build machine's network and unpacked into the tree --
# upstream's install.sh is not used: it probes the running host's GPU and
# writes a systemd unit for it, neither of which is meaningful when the
# target is an image that has never booted.
echo "  fetching ollama $OLLAMA_VERSION ..."
# .tar.zst, not .tgz: upstream switched compression, and the old name
# 404s rather than redirecting.
TARBALL="$ROOTFS/tmp/ollama.tar.zst"
curl -fsSL --retry 3 \
    "https://github.com/ollama/ollama/releases/download/${OLLAMA_VERSION}/ollama-linux-amd64.tar.zst" \
    -o "$TARBALL"

# The tarball unpacks to bin/ollama plus lib/ollama/, so /usr/local is
# the prefix -- NOT /usr. Upstream's own installer uses the same layout;
# getting this wrong scatters the runtime libraries where the binary
# cannot find them.
#
# tar --zstd shells out to the zstd binary rather than linking a
# library, which is why zstd is in packages.txt.
in_chroot "install -d -m 755 /usr/local/lib/ollama && tar -C /usr/local --zstd -xf /tmp/ollama.tar.zst"
rm -f "$TARBALL"

in_chroot "test -x /usr/local/bin/ollama" || {
    echo "error: /usr/local/bin/ollama not found after unpacking" >&2
    exit 1
}

# Its own system user: the model store is large, shared, and has no
# business being writable by the agent's own account.
in_chroot "
    id -u $OLLAMA_USER >/dev/null 2>&1 || \
        useradd --system --create-home --home-dir $OLLAMA_HOME \
                --shell /usr/sbin/nologin --groups users $OLLAMA_USER
"

# Loopback only. Ollama's default binds 127.0.0.1 already; stated
# explicitly because the endpoint is unauthenticated and an upstream
# default change would silently expose the device on its network.
install -D -m 644 /dev/stdin "$ROOTFS/etc/systemd/system/ollama.service" <<EOF
[Unit]
Description=Ollama model server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$OLLAMA_USER
Group=$OLLAMA_USER
Environment=OLLAMA_HOST=127.0.0.1:11434
Environment=OLLAMA_MODELS=$OLLAMA_HOME/models
ExecStart=/usr/local/bin/ollama serve
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Deliberately NOT enabled, and deliberately shipping no models.
# Preloading a model would mean a silent multi-GB download at first
# networked boot; the agent starts this service after the owner has
# chosen a model during onboarding.
in_chroot "install -d -m 755 -o $OLLAMA_USER -g $OLLAMA_USER $OLLAMA_HOME/models"

echo "  ollama:    127.0.0.1:11434 (installed, not enabled, no models)"
