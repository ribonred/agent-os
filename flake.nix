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

      # Patches an already-built ui/ Tauri binary (built with the system
      # rustup/cargo-tauri toolchain, per the deliberate decision not to
      # fight Tauri's GTK/webkit deps into a Nix devShell) so it can
      # actually run on NixOS. A normal-distro binary is linked against
      # /lib/x86_64-linux-gnu/*.so and /lib64/ld-linux-x86-64.so.2, which
      # don't exist on NixOS's non-FHS layout -- this rewrites the ELF
      # interpreter and RPATH to point at matching libraries from our own
      # pinned nixpkgs, so the patched result stays reproducible even
      # though the build itself wasn't done via Nix.
      #
      # Usage: nix run .#patch-ui-for-nixos -- /path/to/ui/binary
      packages.${system}.patch-ui-for-nixos = pkgs.writeShellApplication {
        name = "patch-ui-for-nixos";
        runtimeInputs = [ pkgs.patchelf ];
        text = ''
          if [ "$#" -ne 1 ]; then
            echo "usage: patch-ui-for-nixos <path-to-binary>" >&2
            exit 1
          fi
          BINARY="$1"
          INTERPRETER="${pkgs.stdenv.cc.bintools.dynamicLinker}"
          RPATH="${pkgs.lib.makeLibraryPath [
            pkgs.webkitgtk_4_1
            pkgs.gtk3
            pkgs.cairo
            pkgs.gdk-pixbuf
            pkgs.glib
            pkgs.glib-networking
            pkgs.dbus
            pkgs.openssl
            pkgs.librsvg
            pkgs.at-spi2-atk
            pkgs.atkmm
            pkgs.harfbuzz
            pkgs.libsoup_3
            pkgs.pango
            pkgs.stdenv.cc.cc.lib
          ]}"
          patchelf --set-interpreter "$INTERPRETER" --set-rpath "$RPATH" "$BINARY"
          echo "Patched $BINARY for NixOS."
          echo "Verify with: ldd $BINARY"
        '';
      };
    };
}
