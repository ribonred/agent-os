# Developer entry points for the dev loop, plus the golden-image build.
# Nothing in this file ships: the device's behavior is defined by
# build/packages.txt and the scripts under build/.
#
# The GUI rides the local Hermes Agent gateway (HERMES_URL); its bearer
# token is auto-discovered from ~/.hermes/.env in dev, so no key setup
# is needed beyond a working `hermes` install. `make hermes-env` shows
# what would be used.

REPO       := $(abspath .)
KEYS_FILE  ?= $(REPO)/cloud-keys.toml
HERMES_URL ?= http://127.0.0.1:8642
# Must match devUrl in ui/src-tauri/tauri.conf.json and devServer in
# ui/nuxt.config.ts -- the shell polls this exact address at startup.
UI_DEV_URL ?= http://localhost:3000

UI_BUNDLE ?= $(REPO)/ui/src-tauri/target/release/ui

ROOTFS ?= $(REPO)/build/rootfs
IMAGE  ?= $(REPO)/build/agentic-os.img

.PHONY: help dev gui hermes-env test ui-bundle rootfs image golden clean-image

help: ## List available targets
	@grep -E '^[a-z-]+:.*##' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  %-16s %s\n", $$1, $$2}'

dev: ## Run the GUI against the local Hermes Agent gateway
	@curl -fsS -m 2 $(HERMES_URL)/health >/dev/null 2>&1 \
	  || echo "WARNING: no Hermes gateway answering at $(HERMES_URL) -- chat will fail. Fix with: hermes gateway restart" >&2
	@cd ui && env -i HOME="$$HOME" USER="$$USER" TERM="$$TERM" \
	  PATH="$$HOME/.bun/bin:$$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
	  NUXT_TELEMETRY_DISABLED=1 bun run build >/dev/null 2>&1 \
	  || echo "WARNING: prebuild failed -- the shell may time out waiting for the dev server" >&2
	@if curl -fsS -m 2 $(UI_DEV_URL) >/dev/null 2>&1; then \
	  echo "WARNING: something already serves $(UI_DEV_URL) -- the shell will attach to it, not to a fresh dev server" >&2; \
	fi
	$(MAKE) -s gui

gui: ## Run the Tauri app (expects the Hermes gateway; see `make dev`)
	# env -i for the same reason as ui-bundle: a clean, predictable
	# environment for the toolchain. Display/session vars pass through so
	# the window, webview, and keyring still work.
	cd ui && env -i HOME="$$HOME" USER="$$USER" TERM="$$TERM" \
	  PATH="$$HOME/.bun/bin:$$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
	  DISPLAY="$$DISPLAY" WAYLAND_DISPLAY="$$WAYLAND_DISPLAY" \
	  XDG_RUNTIME_DIR="$$XDG_RUNTIME_DIR" \
	  DBUS_SESSION_BUS_ADDRESS="$$DBUS_SESSION_BUS_ADDRESS" \
	  NUXT_TELEMETRY_DISABLED=1 \
	  AGENTIC_OS_HERMES_URL=$(HERMES_URL) \
	  bun run tauri dev

hermes-env: ## Show which gateway URL/key the GUI would resolve (key masked)
	@echo "url: $(HERMES_URL)"
	@if [ -n "$$AGENTIC_OS_HERMES_KEY" ]; then src="AGENTIC_OS_HERMES_KEY"; key="$$AGENTIC_OS_HERMES_KEY"; \
	elif [ -f /etc/agentic-os/hermes.env ] && grep -q '^API_SERVER_KEY=' /etc/agentic-os/hermes.env 2>/dev/null; then \
	  src="/etc/agentic-os/hermes.env"; key=$$(sed -n 's/^API_SERVER_KEY=//p' /etc/agentic-os/hermes.env | head -n1); \
	elif [ -f "$$HOME/.hermes/.env" ] && grep -q '^API_SERVER_KEY=' "$$HOME/.hermes/.env"; then \
	  src="$$HOME/.hermes/.env"; key=$$(sed -n 's/^API_SERVER_KEY=//p' "$$HOME/.hermes/.env" | head -n1); \
	else src="(none)"; key=""; fi; \
	if [ -n "$$key" ]; then echo "key: $$(echo "$$key" | cut -c1-4)**** (from $$src)"; \
	else echo "key: NOT FOUND -- set AGENTIC_OS_HERMES_KEY or enable the API server in ~/.hermes/.env" >&2; exit 1; fi
	@curl -fsS -m 2 $(HERMES_URL)/health || { echo "gateway not answering at $(HERMES_URL)" >&2; exit 1; }

test: ## All Rust crate tests + UI typecheck
	cd agent-core/hw-probe && cargo test
	cd agent-core/cloud-key && cargo test
	cd ui && bun run check

rootfs: ## Build the golden rootfs (stage 1 -- needs root and a network)
	sudo $(REPO)/build/build-rootfs.sh --rootfs $(ROOTFS) --ui $(UI_BUNDLE)

image: ## Turn the rootfs into a bootable disk image (stage 2 -- needs root)
	@test -d "$(ROOTFS)" || { echo "no rootfs at $(ROOTFS) -- run 'make rootfs' first" >&2; exit 1; }
	sudo $(REPO)/build/make-image.sh --rootfs $(ROOTFS) --out $(IMAGE)

golden: ui-bundle rootfs image ## Build everything: UI, rootfs, and the flashable image

clean-image: ## Remove the built rootfs and image
	sudo rm -rf $(ROOTFS) $(IMAGE) $(IMAGE).zst

ui-bundle: ## Build the release Tauri binary with the system toolchain
	# Clean caches first: a stale bundler cache once produced a build
	# whose component markup and scoped CSS came from different compiler
	# generations -- styles matched nothing, layout collapsed on-device.
	cd ui && rm -rf .nuxt .output dist node_modules/.vite
	# env -i on purpose: an inherited toolchain must not leak into this
	# build -- mixing compilers with the system GTK libs breaks the final
	# link. ui/ is built with the system toolchain, always, and the
	# result is copied straight into the image; on a normal FHS distro it
	# needs no interpreter or RPATH rewriting.
	# The device overlay makes the window fullscreen/undecorated so the
	# assistant fills the screen; dev (`make gui`) keeps a normal window.
	cd ui && env -i HOME="$$HOME" USER="$$USER" TERM="$$TERM" \
	  PATH="$$HOME/.bun/bin:$$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
	  NUXT_TELEMETRY_DISABLED=1 \
	  bun run tauri build --no-bundle --config src-tauri/tauri.kiosk.conf.json
	# Canary for the stale-cache failure: the orb's scoped-style
	# attribute must pair up between built CSS and built JS markup.
	@hash=$$(grep -rho '\.orb\[data-v-[a-f0-9]*\]' ui/dist/_nuxt/*.css | sort -u | sed 's/.*\[\(data-v-[a-f0-9]*\)\]/\1/'); \
	if [ -z "$$hash" ]; then echo "CANARY: no scoped .orb rule in built CSS -- broken build, do not ship" >&2; exit 1; fi; \
	if ! grep -rqF "$$hash" ui/dist/_nuxt/*.js; then \
	  echo "SCOPE HASH MISMATCH: css has $$hash but no JS applies it -- stale-cache build, do not ship" >&2; exit 1; \
	fi; echo "scope-hash check ok: $$hash"
