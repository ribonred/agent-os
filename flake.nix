{
  description = "agentic-os: base OS for MSI AI-assistant devices (NixOS track, mini-PC tier)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    # Upstream maintains its flake best-effort, so updates are a
    # deliberate, tested step -- never a casual `nix flake update`.
    # Deliberately does NOT follow our nixpkgs: rebasing it trades a
    # known-good build for a smaller closure.
    hermes-agent.url = "github:NousResearch/hermes-agent";
  };

  outputs = { self, nixpkgs, hermes-agent }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      # The Tauri UI shell, built OUTSIDE Nix with the system toolchain
      # (standing decision -- Tauri's GTK/webkit deps aren't worth
      # fighting into Nix), then autoPatchelfHook rewrites its ELF
      # interpreter/RPATH against our pinned stack.
      #
      #   AGENTIC_OS_UI_BUNDLE=/path/to/ui nix build .#<target> --impure
      #
      # A pure build yields null and a headless system -- the session is
      # additive, never a hard dependency. A fully in-Nix UI build
      # remains the goal; this formalizes the interim.
      uiBundlePath = builtins.getEnv "AGENTIC_OS_UI_BUNDLE";
      uiShell =
        if uiBundlePath == "" then
          null
        else
          pkgs.stdenv.mkDerivation {
            pname = "agentic-ui-shell";
            version = "0.1.0";
            src = /. + uiBundlePath;
            dontUnpack = true;
            nativeBuildInputs = [ pkgs.autoPatchelfHook ];
            # The same runtime stack patch-ui-for-nixos targets -- these
            # become the RPATH autoPatchelfHook resolves against.
            buildInputs = with pkgs; [
              webkitgtk_4_1
              gtk3
              cairo
              gdk-pixbuf
              glib
              glib-networking
              dbus
              openssl
              librsvg
              at-spi2-atk
              atkmm
              harfbuzz
              libsoup_3
              pango
              stdenv.cc.cc.lib
            ];
            installPhase = ''
              install -D -m 755 $src $out/bin/agentic-ui
            '';
          };
    in
    {
      # "host" is generic on purpose -- the running agent decides
      # local-vs-cloud routing from detected hardware, not the OS config.
      nixosConfigurations.host = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          uiShellPackage = uiShell;
          # Pinned into the device's flake registry -- see
          # hosts/host/configuration.nix.
          inherit nixpkgs;
        };
        modules = [
          ./hosts/host/configuration.nix
          ./hosts/host/hardware-configuration.nix
          ./modules/tool-registry/postgres.nix
          ./modules/tool-registry/redis.nix
          ./modules/tool-registry/ollama.nix
          ./modules/session.nix
          ./modules/desktop.nix
          hermes-agent.nixosModules.default
          ./modules/hermes-agent.nix
        ];
      };

      # Self-installing image: boots, wipes the target disk, installs
      # from a closure baked into the ISO (fully offline), powers off.
      # One image provisions any number of identical units.
      nixosConfigurations.installer = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          hostSystem = self.nixosConfigurations.host.config.system.build.toplevel;
        };
        modules = [ ./hosts/installer/installer.nix ];
      };

      # Dev only -- never shipped. The host gets compiled output of
      # agent-core crates, never a compiler.
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rust-analyzer
        ];
        # nixpkgs' rustc doesn't bundle stdlib sources in its sysroot
        # the way rustup does; rust-analyzer needs them explicitly.
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      };

      # Makes a system-toolchain-built ui/ binary runnable on NixOS: a
      # normal-distro binary links against paths that don't exist on a
      # non-FHS layout, so rewrite the interpreter and RPATH against our
      # pinned nixpkgs.
      packages.${system} = (if uiShell == null then { } else { ui-shell = uiShell; }) // {
        installer-iso =
          self.nixosConfigurations.installer.config.system.build.isoImage;

      # Usage: nix run .#patch-ui-for-nixos -- /path/to/ui/binary
      patch-ui-for-nixos = pkgs.writeShellApplication {
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
    };
}
