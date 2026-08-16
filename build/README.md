# Golden image build

The device ships as a **golden image**, not an installer. A complete
system is built once here, snapshotted, and copied onto each unit
byte-for-byte. Nothing is installed, resolved or downloaded on the
customer's device — it powers on with every tool already in place.

```
  build machine (network, once)          unit (offline, per device)
  ─────────────────────────────          ──────────────────────────
  build-rootfs.sh   →  rootfs/
  make-image.sh     →  agentic-os.img.zst  →  flash.sh  →  /dev/nvme0n1
```

## Building

```bash
make golden          # UI + rootfs + image, end to end
```

or one stage at a time:

```bash
make ui-bundle       # the Tauri shell binary (system toolchain)
make rootfs          # stage 1: debootstrap + everything installed
make image           # stage 2: bootable disk image
```

Stages 1 and 2 need root — `debootstrap`, `chroot`, loop devices and
`mkfs` all require it. Stage 1 needs a network; nothing afterwards does.

Build-machine prerequisites:

```bash
sudo apt install debootstrap gdisk dosfstools e2fsprogs rsync zstd
```

## Testing before flashing anything

```bash
qemu-system-x86_64 -m 4096 -enable-kvm \
  -bios /usr/share/ovmf/OVMF.fd \
  -drive file=build/agentic-os.img,format=raw
```

## Flashing a unit

```bash
sudo ./build/flash.sh --image build/agentic-os.img.zst
```

Autodetects the internal disk (NVMe first, then non-removable SATA) and
refuses to write to the medium it is running from. **This destroys the
target disk.** A provisioning tool, not a rescue system — never hand it
to a customer.

## What goes where

| File | Role |
|---|---|
| `packages.txt` | Every apt package in the image. The device installs nothing itself, so anything missing here is missing on the shipped unit. |
| `build-rootfs.sh` | Stage 1. Debootstraps Ubuntu, installs packages, calls the scripts below, strips per-unit state. |
| `scripts/install-services.sh` | PostgreSQL (unix socket only) and Redis (loopback). |
| `scripts/install-inference.sh` | llama.cpp (from the archive) and Ollama (upstream tarball). Both installed, neither enabled, no models. |
| `scripts/install-hermes.sh` | The agent runtime, its system user, sudo rule, identity and gateway service. |
| `scripts/install-desktop.sh` | Ubuntu Desktop autologin and the assistant's autostart entry. |
| `rootfs-overlay/` | Files copied into the image verbatim, including the first-boot unit. |
| `make-image.sh` | Stage 2. GPT + ESP + ext4 root, bootloader, compression. |
| `flash.sh` | Stage 3. Raw block copy onto a unit. |

## Per-unit state

One image is copied onto every unit, so anything that must differ
between them cannot be baked in. `build-rootfs.sh` strips it, and
`agentic-firstboot` regenerates it on the device:

- **SSH host keys** — identical keys across a fleet would let any unit
  impersonate any other
- **`machine-id`** — left empty, not absent; systemd regenerates it, but
  some versions refuse to boot without the file
- **`API_SERVER_KEY`** — the agent's bearer token, one per unit
- **Root filesystem size** — the image is sized for the smallest disk
  the product ships on and grown to fit whatever the unit actually has

The same unit re-pins the agent's identity on **every** boot: the
constitution is declarative, so the agent's runtime soul-editing must not
survive a reboot. The owner's name and persona are applied per-turn by
the shell instead, so this does not erase what they set up.

## Provisioning a cloud key

Optional. A unit with no vendor key is a normal unit — the owner supplies
their own through the interface. To bake one in, place a
`cloud-keys.toml` at `/etc/agentic-os/cloud-keys.toml` in the rootfs
before running `make-image.sh`:

```toml
[openrouter]
api_key = "..."
```

Costs: the key lands in the image, and every unit flashed from it shares
that key. Fine for bench and small batches; production should inject
per-unit keys as a factory step.

## Local inference

Two engines ship, and **neither is enabled and neither carries a model**:

- **llama.cpp** (`llama-server`, 127.0.0.1:8080) — from the archive, so
  it links the system `libggml`/`libllama` and gets security updates
  with everything else. Serves one GGUF file over an OpenAI-compatible
  API. Starts only once `/var/lib/agentic-os/models/active.gguf` exists.
- **Ollama** (127.0.0.1:11434) — not in the archive, so it comes from
  upstream's pinned release tarball. Model pulls and hot-swapping.

Model acquisition is a multi-GB transfer and must be a visible
onboarding step with progress and consent, never something that happens
silently at first networked boot. Until the owner picks a model, routing
leans cloud — the honest state of a fresh device.

The Ollama tarball adds roughly **1.4 GB compressed** to the image
because it bundles CUDA and ROCm runtimes the mini-PC tier has no use
for. To build without it:

```bash
OLLAMA_SKIP=1 make rootfs
```

The device still has a local engine in that configuration.

## Known gaps

- **The image is not reproducible from a clean checkout.** The UI binary
  is built outside this pipeline and copied in, and apt resolves against
  a live mirror. Pinning to a snapshot mirror would close the second
  half of that gap.
- **GPU acceleration is Vulkan-only.** `libggml0-backend-vulkan` uses
  the iGPU without dragging in the CUDA stack; the DGX tier will need a
  different backend decision.
