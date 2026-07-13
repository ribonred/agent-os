{ pkgs, ... }:

# Install/activation side of the "postgres" tool-registry entry.
# Agent-facing metadata (what it's for, when to use it) lives in
# registry/postgres.yaml, deliberately kept out of Nix so the running
# agent can read it without evaluating the flake.
{
  services.postgresql = {
    enable = true;
    package = pkgs.postgresql_18;
    enableTCPIP = false; # unix socket only -- nothing on this box needs network pg yet
  };
}
