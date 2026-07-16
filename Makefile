# Developer entry points for the dev loop. Device-side behavior is
# defined entirely by the flake -- nothing here ships.
#
# The dev daemon/GUI pair talk over SOCKET; override any of these on the
# command line, e.g.: make daemon KEYS_FILE=/somewhere/else/keys.toml

REPO      := $(abspath .)
SOCKET    ?= /tmp/aos-orch.sock
KEYS_FILE ?= $(REPO)/cloud-keys.toml

.PHONY: help dev daemon gui test iso iso-provisioned orchestrator host

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

orchestrator: ## Build the orchestrator package via Nix
	nix build .#orchestrator --print-out-paths

host: ## Build the full NixOS host closure
	nix build .#nixosConfigurations.host.config.system.build.toplevel --print-out-paths
