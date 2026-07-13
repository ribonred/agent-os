{
  description = "agentic-os: base OS for MSI AI-assistant devices (NixOS track, mini-PC tier)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
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

      # Dev-time only -- never shipped on the device. `nix develop` gives a
      # Rust toolchain for iterating on agent-core crates (e.g. hw-probe)
      # with plain cargo build/run/test. The shipped host only ever gets
      # the compiled output of those crates as its own Nix package, never
      # a compiler.
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rust-analyzer
        ];
      };
    };
}
