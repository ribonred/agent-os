{ pkgs, ... }:

{
  networking.hostName = "agentic-os";
  time.timeZone = "UTC"; # placeholder -- set the real timezone before deploying

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  users.users.admin = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    # No password/SSH key set yet -- decide auth strategy before this box
    # leaves the dev bench. Do not ship a device with an unauthenticated user.
    initialPassword = "changeme";
  };

  services.openssh.enable = true;

  # The agent runtime (modules/hermes-agent.nix carries the full
  # configuration). The GUI is a frontend to this gateway -- without it
  # the device has no agent at all, which a bench install demonstrated
  # the hard way: everything booted, and chat had nothing to reach.
  services.hermes-agent.enable = true;

  # Boot identity (design/DESIGN.md): the brand mark centered on a dark
  # splash from early boot until the kiosk compositor takes over, with
  # kernel/systemd chatter silenced -- the product experience starts at
  # power-on, not at the GUI.
  #
  # A custom theme, not `boot.plymouth.logo`, for two reasons verified
  # live on a device VM: plymouth's two-step plugin composites a
  # no-alpha PNG as fully transparent (the watermark simply never
  # appears -- the design JPEG must become an RGBA glyph with its black
  # field cut to transparency), and the stock spinner theme pins the
  # watermark to the bottom edge with no way to center it from the
  # NixOS module.
  boot.plymouth = {
    enable = true;
    theme = "agentic";
    themePackages = [
      (pkgs.runCommand "plymouth-agentic-theme"
        { nativeBuildInputs = [ pkgs.imagemagick ]; }
        ''
          theme=$out/share/plymouth/themes/agentic
          mkdir -p $theme

          # Dialog/throbber assets reused from the stock spinner theme
          # (disk-password prompts etc. still need them).
          cp ${pkgs.plymouth}/share/plymouth/themes/spinner/throbber-*.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/entry.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/bullet.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/lock.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/capslock.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/keyboard.png \
             ${pkgs.plymouth}/share/plymouth/themes/spinner/keymap-render.png \
             $theme/

          # design/logo.jpg stays the single source of truth: white mark
          # on near-black becomes a transparent-background glyph (alpha
          # from luminance), so it sits seamlessly on the pure-black
          # splash. -strip drops the JPEG's EXIF baggage.
          magick ${../../design/logo.jpg} -resize 256x256 -strip \
            \( +clone -colorspace gray \) -alpha off \
            -compose CopyOpacity -composite PNG32:$theme/watermark.png

          # two-step layout: mark dead-center, spinner below it.
          # Alignments were tuned and screenshot-verified on the VM.
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
  # vt.global_cursor_default=0 kills the blinking console cursor that
  # otherwise shows on the black screen before the splash paints.
  boot.kernelParams = [ "quiet" "splash" "udev.log_level=3" "vt.global_cursor_default=0" ];
  boot.consoleLogLevel = 3;
  boot.initrd.verbose = false;
  # No bootloader menu on a customer device; holding a key at power-on
  # still summons it for bench work.
  boot.loader.timeout = 0;

  environment.systemPackages = with pkgs; [
    git
    vim
    uv  # every Python *project* invocation goes through uv -- no bare pip
    python3 # ...but a real interpreter must exist: uv's own downloaded
            # CPython builds assume an FHS filesystem layout and fail on
            # NixOS, which left an earlier image with `uv` on PATH and no
            # runnable python at all. This is the interpreter uv should
            # target (e.g. `uv venv --python python3`).
    fnm # nvm is not packaged in nixpkgs -- its curl-installed, rc-file-
        # mutating model doesn't fit how Nix manages tool versions. fnm is
        # the equivalent that is actually packaged and works declaratively.
    bun
  ];

  services.ollama = {
    package = pkgs.ollama-cpu;
    # Deliberately NO loadModels here: that would download ~2GB silently
    # at first networked boot, invisible to the person setting up the
    # device. Model acquisition is instead a GUI-driven onboarding step
    # (visible progress, user consent for the download) -- the GUI/agent
    # runtime triggers the pull through Ollama's API when the time
    # comes. Until then the local tier fails loudly and routing leans
    # cloud, which is the honest state of a fresh device.
  };

  # Required for the actual supported-language list (brain/onboarding.md):
  # 5 of the 10 languages need non-Latin scripts, and design/DESIGN.md
  # deliberately relies on the system font stack instead of a bundled
  # webfont -- that choice only works if the system actually has these
  # installed. noto-fonts alone covers Thai/Devanagari/Latin/Vietnamese
  # diacritics; CJK is a separate, much larger package upstream splits
  # out on purpose, hence listed separately here.
  fonts.packages = with pkgs; [
    noto-fonts
    noto-fonts-cjk-sans
  ];

  # The UI shell stores the cloud API key in the OS keyring (Secret
  # Service API) rather than a plain file -- that API is only available
  # if a keyring daemon is actually running. Without this, saving a key
  # in the UI fails loudly instead of silently degrading to insecure
  # storage. Auto-unlock on login (PAM integration) is a follow-up once
  # the device's session/login flow is decided -- until then a first
  # keyring access may prompt to set a keyring password on desktop
  # sessions.
  services.gnome.gnome-keyring.enable = true;

  # A device can also ship with a vendor-provisioned cloud key so the
  # buyer never has to create an API account: the UI shell reads
  # /etc/agentic-os/cloud-keys.toml (root-owned, mode 0600) as a fallback
  # when no user key is in the keyring. The actual key is written at
  # deployment/factory time by the provisioning process, never committed
  # to this repository. Expected shape:
  #
  #   [openrouter]
  #   api_key = "..."
  #
  # A secrets-management tool (e.g. sops-nix/agenix) should own writing
  # that file once the factory provisioning flow is designed.
  #
  # The file is written at factory time as root, before the device user
  # exists on the installed system; this hands it to that user at boot.
  # 'z' only adjusts existing files -- an unprovisioned device is
  # untouched.
  systemd.tmpfiles.rules = [
    "z /etc/agentic-os/cloud-keys.toml 0600 admin users - -"
  ];

  # Bumping this requires reading the NixOS release notes for breaking
  # changes first -- do not upgrade blindly.
  system.stateVersion = "26.05";
}
