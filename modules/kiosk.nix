{ lib, uiShellPackage, ... }:

# Kiosk session: the device boots straight into the assistant UI --
# power on -> orb, no login prompt, no desktop, no visible OS. cage is a
# single-app Wayland compositor: nothing else is launchable from the
# session, which is exactly the appliance posture.
#
# uiShellPackage is null on a pure build (see flake.nix: the UI bundle
# is env-pointed and impure for now), and the whole kiosk collapses to
# nothing -- a headless system with all services still running. The
# kiosk is additive, never a hard dependency of the closure.

{
  config = lib.mkIf (uiShellPackage != null) {
    services.cage = {
      enable = true;
      # Same user the orchestrator daemon runs as: the daemon's socket
      # is chmod 0600, so the session user and daemon user must match.
      user = "admin";
      program = "${uiShellPackage}/bin/agentic-ui";
    };

    # Unlock the keyring as part of the cage session's PAM login so the
    # UI's keyring writes work in-session. Caveat, deliberately noted:
    # cage auto-logs-in without a password, so the keyring can only
    # auto-unlock if its password is blank -- acceptable for the
    # appliance (disk-level protection is the real boundary; a follow-up
    # once device encryption is decided), but it must stay a conscious
    # decision, not an accident.
    security.pam.services.cage.enableGnomeKeyring = true;
  };
}
