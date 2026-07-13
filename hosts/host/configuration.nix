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
    uv  # every Python invocation on this system goes through uv -- no bare
        # python3/pip in this list, on purpose, so there's nothing to reach
        # for except uv
    fnm # nvm is not packaged in nixpkgs -- its curl-installed, rc-file-
        # mutating model doesn't fit how Nix manages tool versions. fnm is
        # the equivalent that is actually packaged and works declaratively.
    bun
  ];

  # This host has no GPU/NPU (Iris Xe integrated graphics only), so the
  # CPU-only package variant is set explicitly rather than relying on a
  # default. hermes3:3b is the smallest available Hermes cut -- appropriate
  # for this tier, not a production-capable local model. Heavier cuts and
  # GPU/ROCm/CUDA package variants belong on hosts with real acceleration.
  services.ollama = {
    package = pkgs.ollama-cpu;
    loadModels = [ "hermes3:3b" ];
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

  # Bumping this requires reading the NixOS release notes for breaking
  # changes first -- do not upgrade blindly.
  system.stateVersion = "26.05";
}
