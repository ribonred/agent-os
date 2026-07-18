# Developer entry points for the dev loop. Device-side behavior is
# defined entirely by the flake -- nothing here ships.
#
# The GUI rides the local Hermes Agent gateway (HERMES_URL); its bearer
# token is auto-discovered from ~/.hermes/.env in dev, so no key setup
# is needed beyond a working `hermes` install. `make hermes-env` shows
# what would be used.

REPO       := $(abspath .)
KEYS_FILE  ?= $(REPO)/cloud-keys.toml
HERMES_URL ?= http://127.0.0.1:8642
SOCKET     ?= /tmp/aos-orch.sock

UI_BUNDLE ?= $(REPO)/ui/src-tauri/target/release/ui

.PHONY: help dev gui hermes-env daemon test iso iso-provisioned iso-kiosk iso-full orchestrator host ui-bundle host-kiosk

help: ## List available targets
	@grep -E '^[a-z-]+:.*##' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  %-16s %s\n", $$1, $$2}'

dev: ## Run the GUI against the local Hermes Agent gateway
	@curl -fsS -m 2 $(HERMES_URL)/health >/dev/null 2>&1 \
	  || echo "WARNING: no Hermes gateway answering at $(HERMES_URL) -- chat will fail. Fix with: hermes gateway restart" >&2
	$(MAKE) -s gui

gui: ## Run the Tauri app (expects the Hermes gateway; see `make dev`)
	# env -i for the same reason as ui-bundle: running make from the repo
	# root inherits the direnv/Nix devShell, whose cc/binutils break the
	# ui link against system GTK. Display/session vars pass through so
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

daemon: ## Run the legacy orchestrator daemon (no longer used by the GUI)
	cd agent-core/orchestrator && \
	AGENTIC_OS_SOCKET=$(SOCKET) \
	AGENTIC_OS_CONSTITUTION=$(REPO)/brain/constitution.md \
	AGENTIC_OS_CLOUD_KEYS_FILE=$(KEYS_FILE) \
	cargo run

test: ## All Rust crate tests + svelte-check
	cd agent-core/hw-probe && cargo test
	cd agent-core/cloud-key && cargo test
	cd agent-core/orchestrator && cargo test
	cd ui && bun run check

iso: ## Build the generic (secret-free) self-installing ISO
	nix build .#installer-iso --print-out-paths

iso-provisioned: ## Build the ISO with KEYS_FILE baked in (see hosts/installer/installer.nix for the costs)
	@test -f "$(KEYS_FILE)" || { echo "KEYS_FILE not found: $(KEYS_FILE)" >&2; exit 1; }
	AGENTIC_OS_BAKE_CLOUD_KEYS=$(KEYS_FILE) nix build .#installer-iso --impure --print-out-paths

iso-kiosk: ## Build the ISO with the UI kiosk baked, no secrets (needs UI_BUNDLE; see `make ui-bundle`)
	@test -f "$(UI_BUNDLE)" || { echo "UI_BUNDLE not found: $(UI_BUNDLE) -- run 'make ui-bundle' first" >&2; exit 1; }
	AGENTIC_OS_UI_BUNDLE=$(UI_BUNDLE) nix build .#installer-iso --impure --print-out-paths

iso-full: ## Build the complete device image: kiosk UI + provisioned cloud key
	@test -f "$(UI_BUNDLE)" || { echo "UI_BUNDLE not found: $(UI_BUNDLE) -- run 'make ui-bundle' first" >&2; exit 1; }
	@test -f "$(KEYS_FILE)" || { echo "KEYS_FILE not found: $(KEYS_FILE)" >&2; exit 1; }
	AGENTIC_OS_UI_BUNDLE=$(UI_BUNDLE) AGENTIC_OS_BAKE_CLOUD_KEYS=$(KEYS_FILE) nix build .#installer-iso --impure --print-out-paths

orchestrator: ## Build the orchestrator package via Nix
	nix build .#orchestrator --print-out-paths

host: ## Build the full NixOS host closure (pure = headless, no kiosk)
	nix build .#nixosConfigurations.host.config.system.build.toplevel --print-out-paths

ui-bundle: ## Build the release Tauri binary with the system toolchain
	# Clean caches first: a stale bundler cache once produced a build
	# whose component markup and scoped CSS came from different compiler
	# generations -- styles matched nothing, layout collapsed on-device.
	cd ui && rm -rf .nuxt .output dist node_modules/.vite
	# env -i on purpose: the repo devShell's Nix cc/binutils must not
	# leak into this build -- mixing them with system GTK libs breaks
	# the final link. ui/ is built with the system toolchain, always.
	# The kiosk overlay makes the device window fullscreen/undecorated;
	# dev (`make gui`) keeps the normal window.
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

host-kiosk: ## Build the host closure with the kiosk (needs UI_BUNDLE; see `make ui-bundle`)
	@test -f "$(UI_BUNDLE)" || { echo "UI_BUNDLE not found: $(UI_BUNDLE) -- run 'make ui-bundle' first" >&2; exit 1; }
	AGENTIC_OS_UI_BUNDLE=$(UI_BUNDLE) nix build .#nixosConfigurations.host.config.system.build.toplevel --impure --print-out-paths
