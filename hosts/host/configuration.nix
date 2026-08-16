{ pkgs, lib, nixpkgs, ... }:

{
  networking.hostName = "agentic-os";
  time.timeZone = "UTC"; # placeholder -- set the real timezone before deploying

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # `nix profile` resolves nixpkgs#foo through the registry. Unpinned it
  # follows the default registry, so anything the agent installs is built
  # against a different nixpkgs and drags a second glibc onto a small
  # disk. Pinned, it reuses what is already there.
  nix.registry.nixpkgs.flake = nixpkgs;

  users.users.admin = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    # Decide auth strategy before this leaves the bench. Do not ship a
    # device with an unauthenticated user.
    initialPassword = "changeme";
  };

  services.openssh.enable = true;

  # Without this the device has no agent at all -- a bench install proved
  # it: everything booted and chat had nothing to reach.
  services.hermes-agent.enable = true;

  # Boot identity (design/DESIGN.md): brand mark on black from power-on,
  # no kernel chatter. A custom theme rather than `boot.plymouth.logo`
  # for two reasons found on a VM: two-step composites a no-alpha PNG as
  # fully transparent, and the stock spinner theme pins the watermark to
  # the bottom edge with no way to center it.
  boot.plymouth = {
    enable = true;
    theme = "agentic";
    themePackages = [
      (pkgs.runCommand "plymouth-agentic-theme"
        { nativeBuildInputs = [ pkgs.imagemagick ]; }
        ''
          theme=$out/share/plymouth/themes/agentic
          mkdir -p $theme

          # Dialog/throbber assets from the stock spinner theme --
          # disk-password prompts still need them.
          cp ${pkgs.plymouth}/share/plymouth/themes/spinner/throbber-*.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/entry.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/bullet.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/lock.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/capslock.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/keyboard.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/keymap-render.png \
             $theme/

          # design/logo.jpg is the single source: alpha derived from
          # luminance so the mark sits on pure black seamlessly.
          magick ${../../design/logo.jpg} -resize 256x256 -strip \
            \( +clone -colorspace gray \) -alpha off \
            -compose CopyOpacity -composite PNG32:$theme/watermark.png

          # Mark dead-center, spinner below. Alignments screenshot-verified.
          cat > $theme/agentic.plymouth <<EOF
          [Plymouth Theme]
          Name=agentic-os
          Description=Brand mark on black
          ModuleName=two-step

          [two-step]
          ImageDir=$theme
          DialogHorizontalAlignment=.5
          DialogVerticalAlignment=.382
          TitleHorizontalAlignment=.5
          TitleVerticalAlignment=.382
          HorizontalAlignment=.5
          VerticalAlignment=.68
          WatermarkHorizontalAlignment=.5
          WatermarkVerticalAlignment=.5
          Transition=none
          TransitionDuration=0.0
          BackgroundStartColor=0x000000
          BackgroundEndColor=0x000000
          ProgressBarBackgroundColor=0x606060
          ProgressBarForegroundColor=0xffffff
          MessageBelowAnimation=true

          [boot-up]
          UseEndAnimation=false

          [shutdown]
          UseEndAnimation=false
          EOF
        '')
    ];
  };
  # vt.global_cursor_default=0 kills the blinking console cursor before
  # the splash paints.
  boot.kernelParams = [ "quiet" "splash" "udev.log_level=3" "vt.global_cursor_default=0" ];
  boot.consoleLogLevel = 3;
  boot.initrd.verbose = false;
  # No bootloader menu; holding a key at power-on still summons it.
  boot.loader.timeout = 0;

  # Scoped rather than a blanket allowUnfree: anything else unfree that
  # creeps in should be a build failure someone looks at, not something
  # that silently ships. Redistributing Google's binary in a sold product
  # is a licensing question worth confirming before units ship.
  nixpkgs.config.allowUnfreePredicate =
    pkg: builtins.elem (lib.getName pkg) [ "google-chrome" ];

  environment.systemPackages = with pkgs; [
    # The only application baked in beyond what the system needs --
    # everything else is installed on request, since each package here
    # costs size in an ISO carrying the whole closure for offline install.
    google-chrome

    git
    vim
    uv # every Python *project* invocation goes through uv -- no bare pip
    # ...but a real interpreter must exist: uv's downloaded CPython
    # assumes FHS and fails on NixOS. This is what uv should target.
    python3
    fnm # nvm isn't packaged; its rc-file-mutating model doesn't fit Nix
    bun
  ];

  services.ollama = {
    package = pkgs.ollama-cpu;
    # No loadModels: that downloads ~2GB silently at first networked
    # boot. Model acquisition is a GUI onboarding step with visible
    # progress and consent. Until then routing leans cloud, which is the
    # honest state of a fresh device.
  };

  # 5 of the 10 supported languages need non-Latin scripts, and
  # design/DESIGN.md relies on the system font stack rather than a
  # bundled webfont -- which only works if these are installed. CJK is a
  # separate, much larger package upstream splits out on purpose.
  fonts.packages = with pkgs; [
    noto-fonts
    noto-fonts-cjk-sans
  ];

  # The shell stores the cloud key in the OS keyring, which needs a
  # daemon running -- without this, saving a key fails loudly rather than
  # degrading to insecure storage.
  services.gnome.gnome-keyring.enable = true;

  # Optional vendor-provisioned cloud key, so the buyer needs no API
  # account. Written at factory time as root, before the device user
  # exists; 'z' hands it over at boot and leaves unprovisioned devices
  # untouched. Shape: [openrouter] / api_key = "...". Never committed --
  # a secrets tool should own writing it once provisioning is designed.
  systemd.tmpfiles.rules = [
    "z /etc/agentic-os/cloud-keys.toml 0600 admin users - -"
  ];

  # Read the release notes before bumping -- do not upgrade blindly.
  system.stateVersion = "26.05";
}
