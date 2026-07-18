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
    # NOTE: identity does NOT go through `documents` -- that option
    # installs workspace context files, while hermes reads its identity
    # (system-prompt slot #1) from $HERMES_HOME/SOUL.md, which upstream
    # treats as runtime-owned and auto-seeds with the Hermes default.
    # The first device chat proved it: the agent introduced itself as
    # "built by Nous Research" with the constitution sitting unused in
    # the workspace. The tmpfiles rule below owns the identity file
    # instead.

    settings = {
      # Bare OpenRouter model id, NOT "openrouter/vendor/model": hermes
      # passes this string to the provider verbatim, and OpenRouter 400s
      # on the prefixed form ("not a valid model ID" -- hit live on the
      # first device chat). Shape mirrors a known-good hermes install.
      model = {
        provider = "openrouter";
        default = "x-ai/grok-4.5";
        base_url = "https://openrouter.ai/api/v1";
        api_mode = "chat_completions";
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
  #
  # 'C+' (unconditional copy) pins the constitution as the agent's
  # identity file on every boot: on this appliance the constitution IS
  # the identity and stays declarative -- hermes' runtime soul-editing
  # never survives a reboot. The owner's name/persona still apply per
  # turn via the UI's system_message overlay, not by editing this file.
  systemd.tmpfiles.rules = [
    "z /etc/agentic-os/hermes.env 0600 admin users - -"
    "C+ /var/lib/hermes/.hermes/SOUL.md 0660 hermes hermes - ${../brain/constitution.md}"
  ];
}
