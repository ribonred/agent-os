{ pkgs, lib, uiShellPackage, ... }:

# The graphical session: Hyprland, assistant on workspace 1, apps on
# their own workspaces. Replaced services.cage, which runs exactly one
# program and so could not host a browser. See design/DESKTOP.md.
#
# Null uiShellPackage (pure build) collapses this to nothing -- headless
# system, all services still running.

let
  hyprlandConfig = pkgs.writeText "hyprland.conf" ''
    general {
      gaps_in = 0
      gaps_out = 0
      border_size = 0
    }

    decoration {
      rounding = 0
    }

    animations {
      enabled = true
      bezier = ease, 0.25, 0.1, 0.25, 1.0
      animation = workspaces, 1, 3, ease, slide
      animation = windows, 1, 3, ease
      animation = fade, 1, 3, ease
    }

    misc {
      # The OS never addresses the owner directly (constitution.md).
      disable_hyprland_logo = true
      disable_splash_rendering = true
      background_color = 0x0b0e14
    }

    input {
      kb_layout = us
      touchpad {
        natural_scroll = true
      }
    }

    # Scale 1 explicitly: panels that misreport their physical size get
    # 2x, which doubles everything on top of the webview's own zoom pin.
    monitor = , preferred, auto, 1

    exec-once = ${uiShellPackage}/bin/agentic-ui
    exec-once = ${pkgs.hyprpaper}/bin/hyprpaper --config /etc/xdg/hypr/hyprpaper.conf
    exec-once = ${pkgs.waybar}/bin/waybar
    exec-once = ${pkgs.dunst}/bin/dunst -config /etc/xdg/dunst/dunstrc

    $mod = SUPER
    bind = $mod, 1, workspace, 1
    bind = $mod, 2, workspace, 2
    bind = $mod, 3, workspace, 3
    bind = $mod, 4, workspace, 4
    bind = $mod, Escape, workspace, 1

    # [workspace empty] assigns at launch -- without it a tiling
    # compositor splits workspace 1 beside the assistant.
    bind = $mod, Space, exec, ${pkgs.fuzzel}/bin/fuzzel
    bind = $mod, B, exec, [workspace empty] google-chrome-stable
    bind = $mod, E, exec, [workspace empty] ${pkgs.nautilus}/bin/nautilus
    bind = $mod SHIFT, Return, exec, [workspace empty] ${pkgs.foot}/bin/foot

    bindel = , XF86AudioRaiseVolume, exec, ${pkgs.wireplumber}/bin/wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
    bindel = , XF86AudioLowerVolume, exec, ${pkgs.wireplumber}/bin/wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
    bindl = , XF86AudioMute, exec, ${pkgs.wireplumber}/bin/wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
    bindel = , XF86MonBrightnessUp, exec, ${pkgs.brightnessctl}/bin/brightnessctl set 5%+
    bindel = , XF86MonBrightnessDown, exec, ${pkgs.brightnessctl}/bin/brightnessctl set 5%-
  '';
in

{
  config = lib.mkIf (uiShellPackage != null) {
    programs.hyprland = {
      enable = true;
      # start-hyprland below is already the watchdog; UWSM would be a
      # second thing owning the same compositor's lifecycle.
      withUWSM = false;
    };

    # Autologin, no greeter. start-hyprland (not Hyprland) is upstream's
    # own session entry: it wraps the compositor in a watchdog, and a
    # device with no second way in cannot afford a dead compositor.
    services.greetd = {
      enable = true;
      settings =
        let
          session = {
            command = "${pkgs.hyprland}/bin/start-hyprland -- --config ${hyprlandConfig}";
            user = "admin";
          };
        in
        {
          initial_session = session;
          default_session = session;
        };
    };

    environment.sessionVariables = {
      # Both diagnosed on a VM: the webview crashed on VMware's virtual
      # GPU without the first, hardware cursors are unreliable there.
      WEBKIT_DISABLE_DMABUF_RENDERER = "1";
      WLR_NO_HARDWARE_CURSORS = "1";
    };

    # wants, not requires: a failed gateway should still show a UI that
    # can report it, not a blank screen.
    systemd.services.greetd = {
      wants = [ "hermes-agent.service" ];
      after = [ "hermes-agent.service" ];
    };

    # Caveat: greetd auto-logs-in without a password, so the keyring only
    # auto-unlocks if its password is blank. Acceptable while disk
    # encryption is undecided, but it must stay a conscious choice.
    security.pam.services.greetd.enableGnomeKeyring = true;
  };
}
