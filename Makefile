# Developer entry points for the dev loop. Device-side behavior is
# defined entirely by the flake -- nothing here ships.
#
# The dev daemon/GUI pair talk over SOCKET; override any of these on the
# command line, e.g.: make daemon KEYS_FILE=/somewhere/else/keys.toml

REPO      := $(abspath .)
SOCKET    ?= /tmp/aos-orch.sock
KEYS_FILE ?= $(REPO)/cloud-keys.toml

UI_BUNDLE ?= $(REPO)/ui/src-tauri/target/release/ui

.PHONY: help dev daemon gui test iso iso-provisioned iso-kiosk iso-full orchestrator host ui-bundle host-kiosk

help: ## List available targets
	@grep -E '^[a-z-]+:.*##' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  %-16s %s\n", $$1, $$2}'

dev: ## Run daemon + GUI together (daemon dies with the GUI)
	@trap 'kill 0' EXIT; \
	$(MAKE) -s daemon & \
	$(MAKE) -s gui

daemon: ## Run the orchestrator daemon in the foreground
	cd agent-core/orchestrator && \
	AGENTIC_OS_SOCKET=$(SOCKET) \
	AGENTIC_OS_CONSTITUTION=$(REPO)/brain/constitution.md \
	AGENTIC_OS_CLOUD_KEYS_FILE=$(KEYS_FILE) \
	cargo run

gui: ## Run the Tauri app (expects a running daemon; see `make dev`)
	cd ui && AGENTIC_OS_SOCKET=$(SOCKET) bun run tauri dev

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
