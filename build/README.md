# Golden image build

The device ships as a **golden image**, not an installer. A complete
system is built once here, snapshotted, and copied onto each unit
byte-for-byte. Nothing is installed, resolved or downloaded on the
customer's device — it powers on with every tool already in place.

There are two ways to produce that image. They share the same install
scripts, so both put the same software on the device.

**A — capture from a reference machine.** Install Ubuntu on a real unit,
provision it, confirm it works, then snapshot its disk.

```
  the reference NUC                      other units
  ─────────────────                      ───────────
  install Ubuntu 26.04 Desktop
  provision.sh        (installs everything)
  reboot, test it properly
  capture-image.sh    →  agentic-os.img.zst  →  flash.sh  →  /dev/nvme0n1
```

**B — synthesize in a container.** No reference machine; the tree is
assembled from scratch.

```
  build machine (network, once)          unit (offline, per device)
  ─────────────────────────────          ──────────────────────────
  build-rootfs.sh   →  rootfs/
  make-image.sh     →  agentic-os.img.zst  →  flash.sh  →  /dev/nvme0n1
```

A is the faster route to a first working device: it runs on the hardware
the product ships on, so drivers, boot and the GPU are proven rather than
deferred. B is reproducible without hardware and is the better factory
path once the design has settled. Start with A, move to B.

## A — building from a reference machine

**1. Install Ubuntu 26.04 Desktop on the NUC**, normally, from a USB
stick. Nothing special: create the owner's account, connect to a
network, let it finish and reboot.

**2. Get this repo onto it** and provision:

```bash
git clone <this repo> && cd agentic-os
sudo ./build/provision.sh --user <the account you created>
```

It installs every package, both inference engines, the agent runtime,
the browser, and enables autologin. Options match the container build —
`OLLAMA_SKIP=1`, `BROWSER_SKIP=1`, and `--key ~/path/to/key.pem` if the
agent-runtime clone is slow over HTTPS.

To include the assistant UI, build it first (`make ui-bundle` on a
machine with the toolchain) and pass `--ui /path/to/ui`.

**3. Reboot and actually use it.** This is the step the container path
cannot do for you: confirm it boots, autologin works, the desktop comes
up, the assistant runs, and the agent answers. Fix anything wrong and
re-run `provision.sh` — it is safe to repeat.

**4. Capture the disk.**

The NUC's own filesystem cannot be imaged while it is running, so this
step boots a **live USB** instead — a temporary system that leaves the
internal disk untouched. That has two consequences worth stating plainly,
because neither is obvious:

- **The repo is not available.** It lives on the internal disk, which is
  the thing being imaged and must stay unmounted. Copy `capture-image.sh`
  onto a USB stick beforehand, or `git clone` it again inside the live
  session — it is one small script with no dependencies.
- **The image cannot be written "here".** A live session's filesystem is
  RAM: it is gone at reboot, and it is smaller than the image. The output
  must go to external storage. The script refuses to write anywhere else
  rather than let you discover this after a long capture.

So, before rebooting, put `capture-image.sh` on a USB stick with room for
the image — one stick does both jobs. Then boot the NUC from the Ubuntu
live USB, choose **Try Ubuntu**, and:

```bash
lsblk                                    # identify the stick and the NUC's disk
sudo mkdir -p /mnt/usb
sudo mount /dev/sdX1 /mnt/usb            # the stick, NOT /dev/nvme0n1

sudo /mnt/usb/capture-image.sh \
     --disk /dev/nvme0n1 \
     --out  /mnt/usb/agentic-os.img
```

`--disk` is the machine being captured; `--out` is where the image is
written. They must be different devices, and the script checks that.

Result: `/mnt/usb/agentic-os.img.zst`, typically 2–4 GB.

**5. Flash the second machine.** Same live USB, same stick:

```bash
sudo /mnt/usb/flash.sh --image /mnt/usb/agentic-os.img.zst
```

## B — building in a container

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
| `scripts/install-desktop.sh` | Ubuntu Desktop autologin, browser, and the assistant's autostart entry. |
| `rootfs-overlay/` | Files copied into the image verbatim, including the first-boot unit. |
| `make-image.sh` | Path B stage 2. GPT + ESP + ext4 root, bootloader, compression. |
| `provision.sh` | Path A. Provisions a live, freshly-installed Ubuntu machine. |
| `capture-image.sh` | Path A. Strips per-unit identity, then images the disk. |
| `flash.sh` | Both paths. Raw block copy onto a unit, from a file or a URL. |

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

[elevenlabs]
api_key = "..."
```

One section per service, and they are independent: a unit can be
provisioned for one and not the other. `openrouter` is what the device
thinks with when it thinks off-device; `elevenlabs` is what it listens
and speaks with. A missing section is simply a device where the owner
supplies that key themselves, or does without.

Costs: the key lands in the image, and every unit flashed from it shares
that key. Fine for bench and small batches; production should inject
per-unit keys as a factory step.

## Voice

Optional, and off on a unit with no `elevenlabs` key — the owner types
instead, and nothing else about the device changes.

Two things it needs that the image cannot supply:

- **A microphone.** The mini-PC tier has none of its own, so a headset
  or USB microphone is part of what ships in the box, or voice is dark on
  that unit. The shell says so in words when it cannot find one.
- **Egress to `api.elevenlabs.io`.** Speech is recognised and generated
  off the device. A site that firewalls outbound HTTPS has a device that
  listens and never answers.

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
