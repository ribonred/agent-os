{ ... }:

# Hermes Agent as the device's agent runtime -- scaffolded, not yet
# live. Everything below is inert until services.hermes-agent.enable is
# flipped to true (the upstream module gates its entire config on it),
# so the shipped closure is unchanged while the integration is built out.
#
# This is the device's agent runtime -- the UI shell is a frontend to
# it. It provides the session-scoped HTTP API the shell drives (create
# sessions, stream turns, long-running runs), persistent cross-session
# memory, a skill system, and first-class bridges to Slack/Telegram/
# WhatsApp for reaching the assistant away from the device. Remaining
# open questions (hardware-tier routing, toolset lockdown) live in the
# internal task tracker, not here.
#
# Interaction model once enabled: `hermes gateway` runs as a hardened
# systemd service, exposing an OpenAI-compatible + sessions API on
# 127.0.0.1:8642, bearer-authenticated. Loopback-only matters: the API
# grants the agent's full toolset, including terminal access, so it must
# never be reachable off the device.

{
  services.hermes-agent = {
    # Identity: the shipped behavior spec installed as the agent's
    # SOUL.md (its primary system-prompt slot). It ships in the closure,
    # so a bad path fails at build time, not on a customer's counter.
    documents."SOUL.md" = ../brain/constitution.md;

    settings = {
      # Hermes family via OpenRouter; the local tier (Ollama on capable
      # hardware) stays in the same model family so behavior is
      # consistent across the local/cloud switch.
      model = {
        provider = "openrouter";
        default = "openrouter/nousresearch/hermes-4-70b";
      };
      memory.memory_enabled = true;
    };

    # Non-secret API-server switches. The secrets live in the
    # provisioned env file below, never here (this attrset is rendered
    # into the world-readable Nix store).
    environment = {
      API_SERVER_ENABLED = "true";
      API_SERVER_HOST = "127.0.0.1";
      API_SERVER_PORT = "8642";
    };

    # Written at factory time by the installer alongside
    # cloud-keys.toml: OPENROUTER_API_KEY (shared vendor key, same
    # trade-offs as the provisioned key file) and API_SERVER_KEY (the
    # local bearer token, generated per unit at install time -- no two
    # devices share it). The upstream module skips this file cleanly if
    # it doesn't exist, so unprovisioned units still boot; the agent
    # then fails loudly at first use instead of silently.
    environmentFiles = [ "/etc/agentic-os/hermes.env" ];
  };

  # The UI shell (running as the device user) must read API_SERVER_KEY
  # from this file to authenticate against the local agent API -- same
  # handoff pattern as cloud-keys.toml in the host config. 'z' only
  # adjusts existing files; unprovisioned devices are untouched.
  systemd.tmpfiles.rules = [
    "z /etc/agentic-os/hermes.env 0600 admin users - -"
  ];
}
