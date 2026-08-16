{ pkgs, ... }:

# Install side of the "postgres" tool-registry entry. Agent-facing
# metadata lives in registry/postgres.yaml, out of Nix so the running
# agent can read it without evaluating the flake.
{
  services.postgresql = {
    enable = true;
    package = pkgs.postgresql_18;
    enableTCPIP = false; # unix socket only -- nothing here needs network pg

    # Peer auth means the role name must match the system user, and the
    # agent runs as `hermes`. Without this role the server is up and
    # unreachable by the only thing that uses it.
    #
    # Superuser grants nothing it could not already take -- it has
    # passwordless root (modules/hermes-agent.nix) -- and avoids walling
    # it off behind a permission error it cannot explain to the owner.
    ensureUsers = [
      {
        name = "hermes";
        ensureClauses.superuser = true;
      }
    ];
  };
}
