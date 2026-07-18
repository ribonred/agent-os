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
