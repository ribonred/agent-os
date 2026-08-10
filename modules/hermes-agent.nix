{ lib, ... }:

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
        default = "deepseek/deepseek-v4-flash-0731";
        base_url = "https://openrouter.ai/api/v1";
        api_mode = "chat_completions";
      };
      memory = {
        memory_enabled = true;
        user_profile_enabled = true;
        write_approval = false;
      };

      # Root-capable agent, gated. The agent administers its own
      # device (packages, services, config) via sudo (granted below),
      # so it is a real admin -- but every action hermes' detector
      # flags as dangerous stops and asks the owner first. That is the
      # product's whole safety posture in one line:
      #   - harmless          -> just do it
      #   - break-the-system / -> "manual" halts the run and surfaces an
      #     data-loss / money      approval.request the owner taps to
      #                            approve or deny (approval_events over
      #                            the sessions API -> a confirm prompt
      #                            in chat)
      #   - unrecoverable     -> hermes' HARDLINE floor blocks it
      #                          outright (disk wipe at /, block-device
      #                          overwrite, power-off) -- not even an
      #                          approval can pass it, by upstream design
      # `deny` is our own never-list on top of that floor: fnmatch
      # globs matched before any approval/bypass. Deliberately tiny --
      # this is the owner's own device with no separate admin and no
      # settings UI, so the agent IS the administrator: it may change
      # its own approval policy (config.yaml), install/remove software,
      # edit system config, all of it. The owner relaxing "stop asking
      # before X" happens THROUGH the agent, so banning policy edits
      # would lock the owner out of their own device. HARDLINE already
      # blocks the unrecoverable cases.
      #
      # The only carve-out is the provisioned credentials: the vendor
      # OpenRouter key and the device auth token. Destroying or
      # exfiltrating those is never "the owner administering their
      # device" -- it has no upside and a leaked key bills the vendor.
      # This is about the secrets, not about restricting the agent.
      approvals = {
        mode = "off";
        deny = [
          "*/etc/agentic-os/hermes.env*"
          "*cloud-keys.toml*"
        ];
      };
    };

    # Non-secret API-server switches. The secrets live in the
    # provisioned env file below, never here (this attrset is rendered
    # into the world-readable Nix store).
    environment = {
      API_SERVER_ENABLED = "true";
      API_SERVER_HOST = "127.0.0.1";
      API_SERVER_PORT = "8642";
      # Hermes also injects factual host details after SOUL.md. Keep that
      # operational awareness while making the appliance boundary explicit
      # at the same prompt layer: the agent uses the OS, the owner uses the
      # device. Direct questions still get an honest answer per the soul.
      HERMES_ENVIRONMENT_HINT = ''
        Treat the host operating system and its package/configuration machinery
        as internal appliance implementation details. Use them silently when
        operating the device. Do not volunteer or narrate Linux, NixOS, Nix,
        packages, services, derivations, generations, or system configuration
        to the owner. Describe outcomes in terms of the device and the owner's
        task. If the owner explicitly asks for technical details, answer
        accurately in plain language.
      '';
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
    "d /var/lib/hermes/.hermes/skills 0770 hermes hermes - -"
    "d /var/lib/hermes/.hermes/skills/device-services 0770 hermes hermes - -"
    "C+ /var/lib/hermes/.hermes/skills/device-services/SKILL.md 0660 hermes hermes - ${../brain/skills/device-services/SKILL.md}"
  ];

  # Root capability for the agent. This device is meant to run itself --
  # install packages, edit config, manage services -- on behalf of a
  # non-technical owner, so the agent user gets passwordless sudo. The
  # safety boundary is NOT "restrict what it can reach" but "confirm
  # before harm": the approvals gate above stops dangerous actions for
  # the owner, and hermes' HARDLINE floor blocks the unrecoverable ones
  # regardless. Granting a narrow allowlist instead was considered and
  # rejected -- an appliance that hits a wall on every unforeseen admin
  # task is not the self-running product.
  security.sudo.extraRules = [{
    users = [ "hermes" ];
    commands = [{
      command = "ALL";
      options = [ "NOPASSWD" "SETENV" ];
    }];
  }];

  # The upstream service hardening makes sudo physically impossible and
  # the filesystem read-only -- both must be lifted for a root-capable
  # agent, or the sudoers rule above is dead letter:
  #   - NoNewPrivileges=true tells the kernel to refuse ANY setuid
  #     escalation, so sudo cannot elevate no matter the sudoers config.
  #   - ProtectSystem=strict mounts the whole fs read-only for the
  #     service, so even as root it could not change the system.
  # Lifting these widens the blast radius by design; the approval gate
  # is what keeps that safe, not the sandbox. Revisit if a future
  # capability-scoped approach can grant admin without full unconfinement
  # (an auto-recovery layer -- snapshot the NixOS generation before
  # harmless changes, roll back on damage -- is the planned complement).
  systemd.services.hermes-agent.serviceConfig = {
    NoNewPrivileges = lib.mkForce false;
    ProtectSystem = lib.mkForce false;
  };
}
