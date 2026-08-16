{ pkgs, lib, uiShellPackage, ... }:

# The desktop layer: bar, launcher, notifications, wallpaper, and the
# agents that make auth prompts and file pickers work. Hyprland alone is
# a compositor, not a desktop.
#
# Written here rather than imported from an existing rice -- those export
# no reusable modules, need home-manager, and track unstable against our
# pinned release. See design/DESKTOP.md.

let
  # design/DESIGN.md tokens. These files are not CSS and cannot read the
  # token list; keep them in step with that document.
  bg = "#0B0E14";
  surface = "#12151C";
  surfaceRaised = "#1A1E27";
  textPrimary = "#E8EAED";
  textSecondary = "#8B93A1";
  accent = "#3DDCFF";
  danger = "#FF6B6B";

  waybarConfig = pkgs.writeText "waybar-config.json" (builtins.toJSON {
    layer = "top";
    position = "top";
    height = 34;
    modules-left = [ "hyprland/workspaces" ];
    modules-center = [ "clock" ];
    modules-right = [ "pulseaudio" "battery" ];
    "hyprland/workspaces" = {
      format = "{name}";
      format-icons = { "1" = "assistant"; };
      on-click = "activate";
    };
    clock = {
      format = "{:%H:%M}";
      tooltip-format = "{:%A %d %B}";
    };
    battery = {
      format = "{capacity}%";
      format-full = ""; # hidden on a mains-powered mini-PC
      states = { warning = 25; critical = 10; };
    };
    pulseaudio = {
      format = "{volume}%";
      format-muted = "muted";
      on-click = "${pkgs.pavucontrol}/bin/pavucontrol";
    };
  });

  # Furniture: never competes with the assistant for attention.
  waybarStyle = pkgs.writeText "waybar-style.css" ''
    * {
      font-family: sans-serif;
      font-size: 13px;
      border: none;
      border-radius: 0;
      min-height: 0;
    }

    window#waybar {
      background: ${bg};
      color: ${textSecondary};
    }

    #workspaces button {
      padding: 0 12px;
      color: ${textSecondary};
      background: transparent;
    }

    #workspaces button.active {
      color: ${textPrimary};
      box-shadow: inset 0 -2px ${accent};
    }

    #clock {
      color: ${textPrimary};
      padding: 0 16px;
    }

    #battery,
    #pulseaudio {
      padding: 0 12px;
    }

    #battery.critical {
      color: ${danger};
    }
  '';

  # fuzzel over rofi: its whole config is a short ini, where rofi needs
  # its own theme language to look like anything but rofi.
  fuzzelConfig = pkgs.writeText "fuzzel.ini" ''
    [main]
    font=sans-serif:size=14
    line-height=32
    horizontal-pad=24
    vertical-pad=20
    inner-pad=8
    width=40

    [colors]
    background=${lib.removePrefix "#" surface}f2
    text=${lib.removePrefix "#" textSecondary}ff
    match=${lib.removePrefix "#" accent}ff
    selection=${lib.removePrefix "#" surfaceRaised}ff
    selection-text=${lib.removePrefix "#" textPrimary}ff
    selection-match=${lib.removePrefix "#" accent}ff
    border=${lib.removePrefix "#" accent}66

    [border]
    width=1
    radius=12
  '';

  dunstConfig = pkgs.writeText "dunstrc" ''
    [global]
      monitor = 0
      follow = mouse
      width = 360
      height = 200
      origin = top-right
      offset = 16x48
      padding = 14
      horizontal_padding = 16
      frame_width = 1
      frame_color = "${accent}"
      separator_color = frame
      font = sans-serif 11
      corner_radius = 12
      idle_threshold = 120

    [urgency_low]
      background = "${surface}"
      foreground = "${textSecondary}"
      timeout = 5

    [urgency_normal]
      background = "${surface}"
      foreground = "${textPrimary}"
      timeout = 8

    [urgency_critical]
      background = "${surface}"
      foreground = "${textPrimary}"
      frame_color = "${danger}"
      timeout = 0
  '';

  # Source is square, so centered rather than stretched. Alpha from
  # luminance -- its baked-in black field would seam against ${bg}.
  wallpaper = pkgs.runCommand "agentic-wallpaper.png"
    { nativeBuildInputs = [ pkgs.imagemagick ]; }
    ''
      magick ${../design/logo.jpg} -resize 1120x1120 -strip \
        \( +clone -colorspace gray \) -alpha off \
        -compose CopyOpacity -composite \
        -channel A -evaluate multiply 0.32 +channel \
        PNG32:mark.png
      magick -size 3840x2160 xc:'${bg}' mark.png \
        -gravity center -compose over -composite $out
    '';

  hyprpaperConfig = pkgs.writeText "hyprpaper.conf" ''
    preload = ${wallpaper}
    wallpaper = , ${wallpaper}
    splash = false
  '';
in

{
  config = lib.mkIf (uiShellPackage != null) {
    environment.systemPackages = with pkgs; [
      waybar
      fuzzel
      dunst
      hyprpaper

      foot
      nautilus

      # Their absence shows up as something silently doing nothing:
      # no clipboard between apps, no volume, no brightness keys.
      wl-clipboard
      pavucontrol
      brightnessctl
    ];

    # Without an agent, anything asking for authorisation fails silently.
    systemd.user.services.hyprpolkitagent = {
      description = "Authentication prompts for the desktop session";
      wantedBy = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.hyprpolkitagent}/libexec/hyprpolkitagent";
        Restart = "on-failure";
        RestartSec = 1;
      };
    };

    # File pickers, screen sharing, "open with".
    xdg.portal = {
      enable = true;
      extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
      xdgOpenUsePortal = true;
    };

    # Paths the session's exec-once lines point at.
    environment.etc = {
      "xdg/waybar/config".source = waybarConfig;
      "xdg/waybar/style.css".source = waybarStyle;
      "xdg/fuzzel/fuzzel.ini".source = fuzzelConfig;
      "xdg/dunst/dunstrc".source = dunstConfig;
      "xdg/hypr/hyprpaper.conf".source = hyprpaperConfig;
    };
  };
}
