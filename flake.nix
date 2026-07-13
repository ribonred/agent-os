{
  description = "agentic-os: base OS for MSI AI-assistant devices (NixOS track, mini-PC tier)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
    in
    {
      # "host" is deliberately generic, not tied to a device model name --
      # the running agent is what decides local-vs-online LLM behavior at
      # runtime based on detected hardware, not the OS config. Currently
      # deployed to a SWNUC11PAHi3000 dev box for validation; see
      # hosts/host/hardware-configuration.nix before deploying to real
      # hardware -- it is a placeholder, not real disk layout.
      nixosConfigurations.host = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          ./hosts/host/configuration.nix
          ./hosts/host/hardware-configuration.nix
          ./modules/tool-registry/postgres.nix
          ./modules/tool-registry/redis.nix
          ./modules/tool-registry/ollama.nix
        ];
      };
    };
}
