{ orchestratorPackage, ... }:

# The agent orchestrator daemon: routes chat between the local model
# (Ollama) and the cloud provider, serving the UI shell over a Unix
# socket. See agent-core/orchestrator for the API and design rationale.

{
  systemd.services.agentic-orchestrator = {
    description = "Agent orchestrator (LLM routing + chat)";
    wantedBy = [ "multi-user.target" ];
    # Ollama is the local backend; ordering after it avoids a burst of
    # connection-refused errors at boot. wants (not requires): the
    # orchestrator must still run when Ollama is down -- cloud routing
    # and /status stay available, and local failures surface as loud
    # per-request errors rather than a dead daemon.
    wants = [ "ollama.service" ];
    after = [ "network.target" "ollama.service" ];

    environment = {
      AGENTIC_OS_SOCKET = "/run/agentic-os/orchestrator.sock";
      # The constitution ships in the system closure -- the daemon
      # refuses to start without its behavior spec, so a bad path here
      # fails at boot, loudly, not at first chat.
      AGENTIC_OS_CONSTITUTION = "${../brain/constitution.md}";
    };

    serviceConfig = {
      ExecStart = "${orchestratorPackage}/bin/orchestrator";
      # Same user as the UI session: the socket is chmod 0600, so only
      # this user's processes (the Tauri shell) can talk to the agent.
      #
      # Known limitation, deliberate for now: as a system service this
      # daemon has no session D-Bus, so it cannot read keys the user
      # saved to the OS keyring -- it falls back to the provisioned key
      # file (loudly logged; see agent-core/cloud-key's read posture).
      # Wiring keyring access needs the device's session/login design
      # (PAM unlock, user service vs system service), which is still an
      # open decision -- revisit alongside that, not before.
      User = "admin";
      Group = "users";
      RuntimeDirectory = "agentic-os";
      Restart = "on-failure";
      RestartSec = 2;
    };
  };

  # The vendor-provisioned key file is written at factory time (as root,
  # before the admin user exists on the installed system); this hands it
  # to the service user at boot. 'z' only adjusts existing files -- a
  # device with nothing provisioned is untouched.
  systemd.tmpfiles.rules = [
    "z /etc/agentic-os/cloud-keys.toml 0600 admin users - -"
  ];
}
