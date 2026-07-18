{
  description = "agentic-os: base OS for MSI AI-assistant devices (NixOS track, mini-PC tier)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    # Hermes Agent: the planned agent runtime (session API the UI will
    # talk to, messaging-platform bridges, skill/memory system). Upstream
    # maintains its Nix flake best-effort ("Tier 2"), so this stays
    # pinned by flake.lock and updates are a deliberate, tested step --
    # never a casual `nix flake update`. It deliberately does NOT follow
    # our nixpkgs: the package set is built against upstream's own pin,
    # and rebasing it onto ours trades a known-good build for a bigger
    # shared closure.
    hermes-agent.url = "github:NousResearch/hermes-agent";
  };

  outputs = { self, nixpkgs, hermes-agent }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      # The Tauri UI shell, packaged from a prebuilt binary. The app is
      # deliberately built OUTSIDE Nix with the system toolchain (the
      # repo's standing decision -- Tauri's GTK/webkit build deps aren't
      # worth fighting into Nix), then autoPatchelfHook rewrites the ELF
      # interpreter/RPATH against our pinned library stack so the result
      # runs on NixOS and stays tied to this nixpkgs revision.
      #
      # Env-pointed + impure, same opt-in pattern as the provisioned ISO:
      #   AGENTIC_OS_UI_BUNDLE=/path/to/ui/src-tauri/target/release/ui \
      #     nix build .#<target> --impure
      # A pure build (no env var) yields uiShell = null and a headless
      # system -- the kiosk is additive, never a hard dependency. A fully
      # in-Nix UI build remains the eventual goal; this formalizes the
      # interim rather than pretending that's solved.
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
      # "host" is deliberately generic, not tied to a device model name --
      # the running agent is what decides local-vs-online LLM behavior at
      # runtime based on detected hardware, not the OS config. Currently
      # deployed to a SWNUC11PAHi3000 dev box for validation.
      nixosConfigurations.host = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          uiShellPackage = uiShell;
        };
        modules = [
          ./hosts/host/configuration.nix
          ./hosts/host/hardware-configuration.nix
          ./modules/tool-registry/postgres.nix
          ./modules/tool-registry/redis.nix
          ./modules/tool-registry/ollama.nix
          ./modules/kiosk.nix
          hermes-agent.nixosModules.default
          ./modules/hermes-agent.nix
        ];
      };

      # Self-installing provisioning image: boots, wipes the target's
      # internal disk, installs the complete host system from a closure
      # baked into the ISO (fully offline), powers off. One image
      # provisions any number of identical units -- there is no per-unit
      # config; hardware-configuration.nix mounts by the filesystem
      # labels the installer creates. Build with: nix build .#installer-iso
      nixosConfigurations.installer = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          hostSystem = self.nixosConfigurations.host.config.system.build.toplevel;
        };
        modules = [ ./hosts/installer/installer.nix ];
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
        # rust-analyzer needs the stdlib sources to resolve std/core
        # symbols; nixpkgs' rustc doesn't bundle them in its sysroot the
        # way rustup does, so point at them explicitly.
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
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
