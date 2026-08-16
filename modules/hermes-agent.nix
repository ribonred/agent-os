{ lib, ... }:

# Hermes Agent: the device's agent runtime. The UI shell is a frontend to
# it -- it provides the session API the shell drives, persistent memory, a
# skill system, and messaging bridges.
#
# `hermes gateway` runs as a systemd service on 127.0.0.1:8642,
# bearer-authenticated. Loopback-only matters: the API grants the agent's
# full toolset including terminal access.

{
  services.hermes-agent = {
    # Identity does NOT go through `documents` -- that installs workspace
    # context files, while hermes reads its identity from
    # $HERMES_HOME/SOUL.md. The tmpfiles rule below owns that file. (The
    # first device chat had the agent introduce itself as "built by Nous
    # Research" with the constitution sitting unused in the workspace.)

    settings = {
      # Bare OpenRouter model id, NOT "openrouter/vendor/model" --
      # OpenRouter 400s on the prefixed form. `default` is duplicated in
      # the UI shell, which must name a model when opening a session;
      # keep the two in step.
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

      # The agent administers this device, so the safety posture is
      # "confirm before harm", not "restrict what it can reach":
      # harmless acts run, dangerous ones raise an approval the owner
      # taps, and hermes' HARDLINE floor blocks unrecoverable ones
      # outright.
      #
      # `deny` is deliberately tiny. The owner has no separate admin and
      # no settings UI, so relaxing a rule happens THROUGH the agent --
      # banning policy edits would lock them out of their own device.
      # The carve-out is only the provisioned credentials: leaking those
      # bills the vendor and is never the owner administering anything.
      approvals = {
        mode = "off";
        deny = [
          "*/etc/agentic-os/hermes.env*"
          "*cloud-keys.toml*"
        ];
      };
    };

    # Non-secret switches only -- this attrset renders into the
    # world-readable Nix store.
    environment = {
      API_SERVER_ENABLED = "true";
      API_SERVER_HOST = "127.0.0.1";
      API_SERVER_PORT = "8642";
      # The appliance boundary, stated at the same prompt layer where
      # hermes injects host facts: the agent uses the OS, the owner uses
      # the device.
      HERMES_ENVIRONMENT_HINT = ''
        Treat the host operating system and its package/configuration machinery
        as internal appliance implementation details. Use them silently when
        operating the device. Do not volunteer or narrate Linux, NixOS, Nix,
        packages, services, derivations, generations, or system configuration
        to the owner. Describe outcomes in terms of the device and the owner's
        task. If the owner explicitly asks for technical details, answer
        accurately in plain language.

        You administer this device and have full read/write access to it,
        including root via sudo. If something appears to fail on permissions,
        it is a real error worth reporting -- not a boundary you should assume
        and work around.

        /home/admin is the owner's home and your working directory. Their
        files live there, and anything you create for them belongs there --
        Documents and Downloads already exist. Use /var/lib/hermes only for
        your own state, never for the owner's work: they cannot see it, and
        the file view in the interface shows their home.
      '';
    };

    # Written by the installer, always: API_SERVER_KEY (per-unit bearer
    # token) and OPENROUTER_API_KEY (empty when no vendor key was baked
    # in, in which case the owner supplies their own through the UI).
    #
    # Unconditional on purpose -- it was once written only alongside a
    # baked cloud key, so every non-provisioned image shipped with no env
    # file and an agent that could not authenticate against its own
    # gateway.
    environmentFiles = [ "/etc/agentic-os/hermes.env" ];
  };

  # 'z' adjusts existing files only, so unprovisioned devices are
  # untouched. 'C+' re-pins on every boot: the constitution IS the
  # identity and stays declarative, so hermes' runtime soul-editing never
  # survives a reboot. Name and persona apply per-turn via the shell's
  # system_message overlay instead.
  systemd.tmpfiles.rules = [
    "z /etc/agentic-os/hermes.env 0600 admin users - -"
    # Group write so the agent works in the owner's home without taking
    # ownership of it.
    "d /home/admin 0775 admin users - -"
    "C+ /var/lib/hermes/.hermes/SOUL.md 0660 hermes hermes - ${../brain/constitution.md}"
    "d /var/lib/hermes/.hermes/skills 0770 hermes hermes - -"
    "d /var/lib/hermes/.hermes/skills/device-services 0770 hermes hermes - -"
    "C+ /var/lib/hermes/.hermes/skills/device-services/SKILL.md 0660 hermes hermes - ${../brain/skills/device-services/SKILL.md}"
  ];

  # The device runs itself on behalf of a non-technical owner, so the
  # agent gets passwordless sudo. A narrow allowlist was rejected: an
  # appliance that hits a wall on every unforeseen admin task is not the
  # self-running product.
  security.sudo.extraRules = [{
    users = [ "hermes" ];
    commands = [{
      command = "ALL";
      options = [ "NOPASSWD" "SETENV" ];
    }];
  }];

  # The agent administers this device, so the systemd sandbox has to be
  # off, not merely loosened. NoNewPrivileges makes the kernel refuse any
  # setuid escalation, so sudo cannot elevate; ProtectSystem and
  # ReadWritePaths keep the filesystem read-only outside /var/lib/hermes,
  # which left it unable to write the owner's own files even as root.
  #
  # The approval gate is the safety boundary here, not the sandbox.
  systemd.services.hermes-agent.serviceConfig = {
    NoNewPrivileges = lib.mkForce false;
    ProtectSystem = lib.mkForce false;
    ProtectHome = lib.mkForce false;
    PrivateTmp = lib.mkForce false;
    ReadWritePaths = lib.mkForce [ "/" ];
    # The owner's files are what the agent works on, so it starts there
    # rather than in a private workspace they cannot see.
    WorkingDirectory = lib.mkForce "/home/admin";
    # Files the agent creates must stay openable by the owner. 0007
    # closes them to everyone outside the group, including admin.
    UMask = lib.mkForce "0002";
  };

  # In `users` so it can write the owner's files, and in `wheel` to match
  # the sudo rule above.
  users.users.hermes.extraGroups = [ "users" "wheel" ];

}
